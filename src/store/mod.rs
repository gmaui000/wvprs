pub mod memory;
pub mod mysql;
pub mod not_impl;
pub mod redis;

use crate::utils::config::Config;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

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

    fn invite(&self, _gb_code: &str, _caller_id: &str, _is_live: bool) -> Option<InviteResult> {
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
