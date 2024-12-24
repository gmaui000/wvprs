pub mod memory;
pub mod mysql;
pub mod not_impl;
pub mod redis;

use crate::sip::message::Catalog;
use crate::utils::config::Config;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

// 定义存储在 sip_devices 中的设备信息结构体
#[derive(Debug)]
pub struct SipDeviceInfo {
    // 分支信息
    pub branch: String,
    // UDP 客户端地址
    pub udp_client_addr: SocketAddr,
    // TCP 流写入器的可选引用
    pub tcp_stream_writer: Option<Arc<Mutex<OwnedWriteHalf>>>,
    // 时间戳
    pub ts: u32,
    // 子设备
    pub sub_devices: Option<Vec<SipSubDeviceInfo>>,
}

// 定义存储在 sip_devices 中的设备信息结构体
#[derive(Debug)]
pub struct SipSubDeviceInfo {
    pub sub_device_id: String,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub owner: String,
    pub civil_code: String,
    pub block: String,
    pub address: String,
    pub parental: u32,
    pub parent_id: String,
    pub register_way: u32,
    pub secrecy: u32,
    pub ip_address: String,
    pub port: u16,
    pub password: String,
    pub status: String,
    pub longitude: f64,
    pub latitude: f64,
    pub ptz_type: u32,
}

// 定义存储在 gb_streams 中的流信息结构体
#[derive(Debug, Default)]
pub struct GbStreamInfo {
    // gb_code, _caller_id, _stream_server_ip, _stream_server_port, ts
    // 设备的 gb_code
    pub gb_code: String,
    // 调用者 ID
    pub caller_id: String,
    // 流媒体 IP
    pub stream_server_ip: String,
    // 流媒体 PORT
    pub stream_server_port: u16,
    // 时间戳
    pub ts: u32,
}

// 用于表示设备信息的结构体
pub struct DeviceInfo {
    pub branch: String,
    pub socket_addr: SocketAddr,
    pub tcp_stream: Option<Arc<Mutex<OwnedWriteHalf>>>,
}

// 用于表示流信息的结构体
pub struct StreamInfo {
    pub gb_code: String,
    pub stream_server_ip: String,
    pub stream_server_port: u16,
}

// 用于表示 invite 操作的返回结果的结构体
pub struct InviteResult {
    pub success: bool,
    pub channel_id: String,
    pub stream_id: u32,
    pub branch: String,
    pub socket_addr: SocketAddr,
    pub tcp_stream: Option<Arc<Mutex<OwnedWriteHalf>>>,
}

// 用于表示 bye 操作的返回结果的结构体
pub struct ByeResult {
    pub success: bool,
    pub call_id: String,
    pub branch: String,
    pub socket_addr: SocketAddr,
    pub tcp_stream: Option<Arc<Mutex<OwnedWriteHalf>>>,
    pub stream_server_ip: String,
    pub stream_server_port: u16,
}

pub trait StoreEngine: Send + Sync {
    fn is_connected(&self) -> bool {
        false
    }

    fn set_global_sn(&self, _v: u32) {}

    fn add_fetch_global_sn(&self) -> u32 {
        0
    }

    fn set_register_sequence(&self, _seq: u32) {}

    fn add_fetch_register_sequence(&self) -> u32 {
        0
    }

    fn set_global_sequence(&self, _seq: u32) {}

    fn add_fetch_global_sequence(&self) -> u32 {
        0
    }

    fn find_device_by_gb_code(&self, _key: &str) -> Option<DeviceInfo> {
        None
    }

    fn find_device_by_stream_id(&self, _key: u32) -> Option<DeviceInfo> {
        None
    }

    fn find_gb_code_by_caller_id(&self, _key: &str) -> Option<String> {
        None
    }

    fn find_gb_code(&self, _stream_id: u32) -> String {
        String::new()
    }

    fn register(
        &self,
        _branch: &str,
        _gb_code: &str,
        _socket_addr: std::net::SocketAddr,
        _tcp_stream: &Option<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    ) -> bool {
        false
    }

    fn unregister(&self, _gb_code: &str) -> bool {
        false
    }

    fn register_keep_alive(&self, _gb_code: &str) -> bool {
        false
    }

    fn save_catalog(&self, _gb_code: &str, _data: Catalog) -> bool {
        false
    }

    fn invite(
        &self,
        _gb_code: &str,
        _channel_id: &str,
        _caller_id: &str,
        _is_live: bool,
    ) -> Option<InviteResult> {
        None
    }

    fn update_stream_server_info(
        &self,
        _stream_id: u32,
        _stream_server_ip: String,
        _stream_server_port: u16,
    ) {
    }

    fn bye(&self, _gb_code: &str, _stream_id: u32) -> Option<ByeResult> {
        None
    }

    fn stream_keep_alive(&self, _gb_code: &str, _stream_id: u32) -> bool {
        false
    }

    fn start_timeout_check(
        &mut self,
        _timeout_devices_sender: mpsc::Sender<Option<String>>,
        _timeout_streams_sender: mpsc::Sender<Option<(String, u32)>>,
    ) {
    }

    fn stop_timeout_check(&mut self) {}
}

pub fn create_store(config: &Config) -> Box<dyn StoreEngine> {
    match config.store_engine.as_str() {
        "memory" => Box::new(memory::MemoryStore::new(config)),
        "mysql" => Box::new(mysql::MySqlStore::new(config)),
        "redis" => Box::new(redis::RedisStore::new(config)),
        _ => {
            tracing::error!("not impl store engine: {}", &config.store_engine);
            Box::new(not_impl::NotImplStore::new(config))
        }
    }
}
