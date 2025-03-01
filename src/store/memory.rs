use tokio;

use dashmap::DashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::StoreEngine;
use super::{GbStreamInfo, SipDeviceInfo};
use crate::sip::message::Catalog;

use crate::utils::config::Config;

pub struct MemoryStore {
    pub quit_flag: bool,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
    pub service_id: String, // 启动时随机生成，向负载均衡器报告
    pub stream_timeout_seconds: u32,
    pub device_timeout_seconds: u32,
    pub live_stream_id: AtomicU32,    // 自动递增
    pub replay_stream_id: AtomicU32,  // 自动递增
    pub global_sn: AtomicU32,         // SN
    pub register_sequence: AtomicU32, // CSeq
    pub global_sequence: AtomicU32,   // CSeq
    pub sip_devices: Arc<DashMap<String, SipDeviceInfo>>,
    pub gb_streams: Arc<DashMap<u32, GbStreamInfo>>,
    pub gb_streams_rev: Arc<DashMap<String, Vec<u32>>>,
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
            sip_devices: Arc::new(DashMap::<String, SipDeviceInfo>::default()),
            gb_streams: Arc::new(DashMap::<u32, GbStreamInfo>::default()),
            gb_streams_rev: Arc::new(DashMap::<String, Vec<u32>>::default()),
        }
    }
}

impl StoreEngine for MemoryStore {
    fn is_connected(&self) -> bool {
        true
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

    fn find_device_by_gb_code(&self, key: &str) -> Option<super::DeviceInfo> {
        if let Some(device) = self.sip_devices.get(key) {
            let SipDeviceInfo {
                branch,
                udp_client_addr,
                tcp_stream_writer,
                ..
            } = device.value();
            return Some(super::DeviceInfo {
                branch: branch.clone(),
                socket_addr: *udp_client_addr,
                tcp_stream: tcp_stream_writer.clone(),
            });
        }
        None
    }

    fn find_device_by_stream_id(&self, key: u32) -> Option<super::DeviceInfo> {
        let gb_code = self.find_gb_code(key);
        if !gb_code.is_empty() {
            return self.find_device_by_gb_code(&gb_code);
        }
        None
    }

    fn find_gb_code_by_caller_id(&self, key: &str) -> Option<String> {
        for entry in self.gb_streams.iter() {
            let (_, stream) = entry.pair();
            let GbStreamInfo {
                gb_code, caller_id, ..
            } = stream;
            if key == caller_id {
                return Some(gb_code.to_string());
            }
        }
        None
    }

    fn find_gb_code(&self, stream_id: u32) -> String {
        if let Some(stream) = self.gb_streams.get(&stream_id) {
            let GbStreamInfo { gb_code, .. } = stream.value();
            return gb_code.to_string();
        }
        String::new()
    }

    fn register(
        &self,
        branch: &str,
        gb_code: &str,
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
                gb_code.to_string(),
                SipDeviceInfo {
                    branch: branch.to_string(),
                    udp_client_addr: socket_addr,
                    tcp_stream_writer: tcp_stream.as_ref().cloned(),
                    ts,
                    sub_devices: None,
                },
            );
            return true;
        }
        false
    }

    fn unregister(&self, gb_code: &str) -> bool {
        if self.sip_devices.get(gb_code).is_some() {
            self.sip_devices.remove(gb_code);
            return true;
        }
        false
    }

    fn register_keep_alive(&self, gb_code: &str) -> bool {
        let entry = self.sip_devices.entry(gb_code.to_string());
        if let dashmap::Entry::Occupied(mut device) = entry {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as u32;
            device.get_mut().ts = ts;
            return true;
        }
        false
    }

    fn save_catalog(&self, gb_code: &str, data: Catalog) -> bool {
        let entry = self.sip_devices.entry(gb_code.to_string());
        if let dashmap::Entry::Occupied(mut device_entry) = entry {
            let device_info = device_entry.get_mut();
            if device_info.sub_devices.is_none() {
                device_info.sub_devices = Some(Vec::new());
            }
            if let Some(sub_devices) = &mut device_info.sub_devices {
                for device in data.device_list.items {
                    sub_devices.push(device);
                }
                return true;
            }
        }
        false
    }

    fn invite(
        &self,
        gb_code: &str,
        channel_id: &str,
        caller_id: &str,
        from_tag: &str,
        is_live: bool,
    ) -> Option<super::InviteResult> {
        let result = self.find_device_by_gb_code(gb_code);
        result.as_ref()?;
        let super::DeviceInfo {
            branch,
            socket_addr,
            tcp_stream,
        } = result.unwrap();

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
            GbStreamInfo {
                gb_code: gb_code.to_string(),
                caller_id: caller_id.to_string(),
                from_tag: from_tag.to_string(),
                to_tag: String::new(),
                stream_server_ip: String::new(),
                stream_server_port: 0,
                ts,
            },
        );

        let is_playing = self.gb_streams_rev.get(gb_code).is_some();

        if let Some(mut streams) = self.gb_streams_rev.get_mut(gb_code) {
            streams.push(stream_id);
        } else {
            self.gb_streams_rev
                .insert(gb_code.to_string(), vec![stream_id]);
        }

        Some(super::InviteResult {
            success: is_playing,
            channel_id: channel_id.to_string(),
            stream_id,
            branch,
            socket_addr,
            tcp_stream,
        })
    }

    fn update_stream_tag_info(&self, from_tag: &str, to_tag: &str) {
        for mut entry in self.gb_streams.iter_mut() {
            let stream = entry.value_mut();
            if from_tag == stream.from_tag {
                stream.to_tag = to_tag.to_string();
            }
        }
    }

    fn update_stream_server_info(
        &self,
        stream_id: u32,
        stream_server_ip: &str,
        stream_server_port: u16,
    ) {
        let entry = self.gb_streams.entry(stream_id);
        if let dashmap::Entry::Occupied(mut stream) = entry {
            stream.get_mut().stream_server_ip = stream_server_ip.to_string();
            stream.get_mut().stream_server_port = stream_server_port;
        }
    }

    fn bye(&self, gb_code: &str, stream_id: u32) -> Option<super::ByeResult> {
        let caller_id: String;
        let from_tag: String;
        let to_tag: String;
        if let Some(stream) = self.gb_streams.get(&stream_id) {
            let GbStreamInfo {
                caller_id: caller_id_inner,
                from_tag: from_tag_inner,
                to_tag: to_tag_inner,
                ..
            } = stream.value();
            caller_id = caller_id_inner.clone();
            from_tag = from_tag_inner.clone();
            to_tag = to_tag_inner.clone();
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

        if let Some(super::DeviceInfo {
            branch,
            socket_addr,
            tcp_stream,
        }) = self.find_device_by_gb_code(gb_code)
        {
            return Some(super::ByeResult {
                success: bye_to_device,
                caller_id,
                from_tag,
                to_tag,
                branch,
                socket_addr,
                tcp_stream,
            });
        }

        None
    }

    fn stream_keep_alive(&self, gb_code: &str, stream_id: u32) -> bool {
        let entry = self.gb_streams.entry(stream_id);
        if let dashmap::Entry::Occupied(mut stream) = entry {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as u32;
            stream.get_mut().gb_code = gb_code.to_string();
            stream.get_mut().ts = ts;
            return true;
        }
        false
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

            // Check condition before entering the loop
            if !*quit_flag {
                loop {
                    let ts_now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs() as u32;

                    let mut timeout_streams = Vec::<(String, u32)>::default();
                    for entry in gb_streams.iter() {
                        let (stream_id, stream) = entry.pair();
                        let GbStreamInfo { gb_code, ts, .. } = stream;
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
                    for entry in sip_devices.iter() {
                        let (gb_code, device) = entry.pair();
                        let SipDeviceInfo { ts, .. } = device;
                        if *ts - ts_now > device_timeout_seconds {
                            timeout_devices.push(gb_code.clone());
                        }
                    }

                    for gb_code in timeout_devices {
                        if let Err(e) = timeout_devices_sender.send(Some(gb_code)) {
                            tracing::error!("timeout_devices_sender.send error, e: {:?}", e);
                        }
                    }

                    // Explicitly check the condition before sleeping
                    if *quit_flag {
                        break;
                    }

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
