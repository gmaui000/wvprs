use tokio;

use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::AtomicU32;
use dashmap::DashMap;

use super::StoreEngine;

use crate::utils::config::Config;

pub struct MemoryStore {
    pub quit_flag: bool,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
    pub service_id: String, // random generated on boot, report to load balance
    pub stream_timeout_seconds: u32,
    pub device_timeout_seconds: u32,
    pub live_stream_id: AtomicU32, // auto increment
    pub replay_stream_id: AtomicU32, // auto increment
    pub global_sn: AtomicU32,      // SN
    pub register_sequence: AtomicU32, // CSeq
    pub global_sequence: AtomicU32, // CSeq
    pub sip_devices: Arc<
        DashMap<
            String,
            (
                String,                                                                      // branch
                std::net::SocketAddr, // udp client addr
                Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>, // tcp stream writer
                u32, // ts
            ),
        >,
    >, // device gb_code -> (branch, net addr, stream server ip, stream server port, ts)
    pub gb_streams: Arc<
        DashMap<u32, (String, String, String, u16, u32)>,
    >, // stream_id -> (device gb_code, caller_id, ts)
    pub gb_streams_rev:
        Arc<DashMap<String, Vec<u32>>>, // device gb_code -> [stream_id]
}

impl MemoryStore {
    pub fn new(config: &Config) -> Self {
        MemoryStore {
            quit_flag: true,
            task_handle: None,
            service_id: Uuid::new_v4().to_string(),
            stream_timeout_seconds: config.stream_timeout_seconds,
            device_timeout_seconds: config.stream_timeout_seconds,
            live_stream_id: AtomicU32::new(1),
            replay_stream_id: AtomicU32::new(1),
            global_sn: AtomicU32::new(0),
            register_sequence: AtomicU32::new(0),
            global_sequence: AtomicU32::new(0),
            sip_devices: Arc::new(DashMap::<
                String,
                (
                    String,
                    std::net::SocketAddr,
                    Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
                    u32,
                ),
            >::default()),
            gb_streams: Arc::new(DashMap::<
                u32,
                (String, String, String, u16, u32),
            >::default()),
            gb_streams_rev: Arc::new(DashMap::<String, Vec<u32>>::default(),
            ),
        }
    }
}

impl StoreEngine for MemoryStore {
    fn is_connected(&self) -> bool {
        return true;
    }

    fn set_global_sn(&self, v: u32) {
        self.global_sn
            .store(v, std::sync::atomic::Ordering::Relaxed);
    }

    fn add_fetch_global_sn(&self) -> u32 {
        self.global_sn
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn set_register_sequence(&self, seq: u32) {
        self.register_sequence
            .store(seq, std::sync::atomic::Ordering::Relaxed);
    }

    fn add_fetch_register_sequence(&self) -> u32 {
        self.register_sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn set_global_sequence(&self, seq: u32) {
        self.global_sequence
            .store(seq, std::sync::atomic::Ordering::Relaxed);
    }

    fn add_fetch_global_sequence(&self) -> u32 {
        self.global_sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1
    }

    fn find_device_by_gb_code(
        &self,
        key: &String,
    ) -> Option<(
        String,
        std::net::SocketAddr,
        Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    )> {
        if let Some(device) = self.sip_devices.get(key) {
            let (branch, addr, tcp_stream, _ts) = device.value();
            return Some((branch.clone(), addr.clone(), tcp_stream.clone()));
        }
        return None;
    }

    fn find_device_by_stream_id(
        &self,
        key: u32,
    ) -> Option<(
        String,
        std::net::SocketAddr,
        Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    )> {
        let gb_code = self.find_gb_code(key);
        if !gb_code.is_empty() {
            return self.find_device_by_gb_code(&gb_code);
        }
        return None;
    }

    fn find_gb_code(&self, stream_id: u32) -> String {
        if let Some(stream) =
            self.gb_streams.get(&stream_id)
        {
            let (gb_code, _caller_id, _stream_server_ip, _stream_server_port, _ts) = stream.value();
            return gb_code.to_string();
        }
        return String::new();
    }

    fn register(
        &self,
        branch: &String,
        gb_code: &String,
        socket_addr: std::net::SocketAddr,
        tcp_stream: &Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    ) -> bool {
        if self.sip_devices.get(gb_code).is_none() {

            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as u32;

            tracing::debug!("register a sip_devices: {:?}", self.sip_devices);
            self.sip_devices.insert(
                gb_code.clone(),
                (branch.clone(), socket_addr, tcp_stream.as_ref().cloned(), ts),
            );
            return true;
        }
        return false;
    }

    fn unregister(&self, gb_code: &String) -> bool {
        if self.sip_devices.get(gb_code).is_some() {
            self.sip_devices.remove(gb_code);
            return true;
        }
        return false;
    }

    fn register_keep_alive(&self, gb_code: &String) -> bool {
        let entry = self.sip_devices.entry(gb_code.to_string());
        if let dashmap::Entry::Occupied(mut device) =  entry {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as u32;
            device.get_mut().3 = ts;
            return true;
        }
        return false;
    }

    fn invite(
        &self,
        gb_code: &String,
        caller_id: &String,
        is_live: bool,
    ) -> Option<(
        bool,
        u32,
        String,
        std::net::SocketAddr,
        Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    )> {
        let result = self.find_device_by_gb_code(gb_code);
        if result.is_none() {
            return None;
        }
        let (branch, device_addr, tcp_stream) = result.unwrap();

        let stream_id = if is_live {
            self.live_stream_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        } else {
            self.replay_stream_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        };

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as u32;

        self.gb_streams.insert(
            stream_id,
            (gb_code.clone(), caller_id.clone(), String::new(), 0, ts),
        );

        let is_playing = self.gb_streams_rev.get(gb_code).is_some();

        if let Some(mut streams) = self.gb_streams_rev.get_mut(gb_code) {
            streams.push(stream_id);
        } else {
            self.gb_streams_rev.insert(gb_code.clone(), vec![stream_id]);
        }

        return Some((is_playing, stream_id, branch, device_addr, tcp_stream));
    }

    fn update_stream_server_info(
        &self,
        stream_id: u32,
        stream_server_ip: String,
        stream_server_port: u16,
    ) {
        let entry = self.gb_streams.entry(stream_id);
        if let dashmap::Entry::Occupied(mut stream) =  entry {
            stream.get_mut().2 = stream_server_ip;
            stream.get_mut().3 = stream_server_port;
        }
    }

    fn bye(
        &self,
        gb_code: &String,
        stream_id: u32,
    ) -> Option<(
        bool,
        String,
        String,
        std::net::SocketAddr,
        Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        String,
        u16,
    )> {
        let call_id: String;
        let ip: String;
        let port: u16;
        if let Some(stream) =
            self.gb_streams.get(&stream_id)
        {
            let (_gb_code, call_id_, stream_server_ip, stream_server_port, _ts) = stream.value();
            call_id = call_id_.clone();
            ip = stream_server_ip.clone();
            port = *stream_server_port;
        } else {
            return None;
        }

        self.gb_streams.remove(&stream_id);

        let mut bye_to_device = false;
        if let Some(mut streams) = self.gb_streams_rev.get_mut(gb_code) {
            streams.retain(|&id| id != stream_id);
            bye_to_device = streams.is_empty();
        }
        if bye_to_device {
            self.gb_streams_rev.remove(gb_code);
        }

        if let Some((branch, addr, tcp_stream)) = self.find_device_by_gb_code(&gb_code) {
            return Some((bye_to_device, call_id, branch, addr, tcp_stream, ip, port));
        }

        return None;
    }

    fn stream_keep_alive(&self, gb_code: &String, stream_id: u32) -> bool {
        let entry = self.gb_streams.entry(stream_id);
        if let dashmap::Entry::Occupied(mut stream) =  entry {
            let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as u32;
            stream.get_mut().0 = gb_code.clone();
            stream.get_mut().4 = ts;
            return true;
        }
        return false;
    }

    fn start_timeout_check(
        &mut self,
        timeout_devices_sender: std::sync::mpsc::Sender<Option<String>>,
        timeout_streams_sender: std::sync::mpsc::Sender<Option<(String, u32)>>,
    ) {
        self.quit_flag = false;

        let quit_flag = Arc::new(self.quit_flag);
        let sip_devices = self.sip_devices.clone();
        let gb_streams = self.gb_streams.clone();

        let stream_timeout_seconds = self.stream_timeout_seconds;
        let device_timeout_seconds = self.device_timeout_seconds;
        self.task_handle = Some(tokio::spawn(async move {
            tracing::info!("start_timeout_check begin");

            while !*quit_flag {
                let ts_now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as u32;

                let mut timeout_streams = Vec::<(String, u32)>::default();
                for entry in gb_streams.iter()
                {
                    let ( stream_id,stream) = entry.pair();
                    let (gb_code, _caller_id, _stream_server_ip, _stream_server_port, ts)  = stream;
                    if *ts - ts_now > stream_timeout_seconds {
                        timeout_streams.push((gb_code.clone(), *stream_id));
                    }
                }

                for (gb_code, stream_id) in timeout_streams {
                    if let Err(e) = timeout_streams_sender.send(Some((gb_code, stream_id))) {
                        tracing::error!("timeout_streams_sender.send error, e: {:?}", e);
                    }
                }

                let mut timeout_devices = Vec::<String>::default();
                for entry in
                    sip_devices.iter()
                {
                    let (gb_code, device) = entry.pair();
                    let (_branch, _sock, _tcp_stream, ts) = device;
                    if *ts - ts_now > device_timeout_seconds {
                        timeout_devices.push(gb_code.clone());
                    }
                }

                for gb_code in timeout_devices {
                    if let Err(e) = timeout_devices_sender.send(Some(gb_code)) {
                        tracing::error!("timeout_devices_sender.send error, e: {:?}", e);
                    }
                }

                if !*quit_flag {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }

            tracing::info!("start_timeout_check end");
        }));
    }

    fn stop_timeout_check(&mut self) {
        self.quit_flag = true;
    }
}

unsafe impl Send for MemoryStore {}

unsafe impl Sync for MemoryStore {}
