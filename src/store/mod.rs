pub mod memory;
pub mod not_impl;
pub mod pg;
pub mod redis;

use crate::utils::cli::CommandLines;

pub trait StoreEngine: Send + Sync {
    fn is_connected(&self) -> bool {
        return false;
    }

    fn set_global_sn(&self, _v: u32) {}

    fn add_fetch_global_sn(&self) -> u32 {
        return 0;
    }

    fn set_register_sequence(&self, _seq: u32) {}

    fn add_fetch_register_sequence(&self) -> u32 {
        return 0;
    }

    fn set_global_sequence(&self, _seq: u32) {}

    fn add_fetch_global_sequence(&self) -> u32 {
        return 0;
    }

    fn find_device_by_gb_code(
        &self,
        _key: &String,
    ) -> Option<(
        String,
        std::net::SocketAddr,
        Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    )> {
        return None;
    }

    fn find_device_by_stream_id(
        &self,
        _key: u32,
    ) -> Option<(
        String,
        std::net::SocketAddr,
        Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    )> {
        return None;
    }

    fn find_gb_code(&self, _stream_id: u32) -> String {
        return String::new();
    }

    fn register(
        &self,
        _branch: &String,
        _gb_code: &String,
        _socket_addr: std::net::SocketAddr,
        _tcp_stream: &Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    ) -> bool {
        return false;
    }

    fn unregister(&self, _gb_code: &String) -> bool {
        return false;
    }

    fn register_keep_alive(&self, _gb_code: &String) -> bool {
        return false;
    }

    fn invite(
        &self,
        _gb_code: &String,
        _caller_id: &String,
        _is_live: bool,
    ) -> Option<(
        bool,
        u32,
        String,
        std::net::SocketAddr,
        Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
    )> {
        return None;
    }

    fn update_stream_server_info(
        &self,
        _stream_id: u32,
        _stream_server_ip: String,
        _stream_server_port: u16,
    ) {
    }

    fn bye(
        &self,
        _gb_code: &String,
        _stream_id: u32,
    ) -> Option<(
        bool,
        String,
        String,
        std::net::SocketAddr,
        Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        String,
        u16,
    )> {
        return None;
    }

    fn stream_keep_alive(&self, _gb_code: &String, _stream_id: u32) -> bool {
        return false;
    }

    fn start_timeout_check(
        &mut self,
        _timeout_devices_sender: std::sync::mpsc::Sender<Option<String>>,
        _timeout_streams_sender: std::sync::mpsc::Sender<Option<(String, u32)>>,
    ) {
    }

    fn stop_timeout_check(&mut self) {}
}

pub fn create_store(cli_args: &CommandLines) -> Box<dyn StoreEngine> {
    match cli_args.store_engine.as_str() {
        "memory" => {
            return Box::new(memory::MemoryStore::new(cli_args));
        }
        "postgresql" => {
            return Box::new(pg::PostgreSqlStore::new(cli_args));
        }
        "redis" => {
            return Box::new(redis::RedisStore::new(cli_args));
        }
        _ => {
            tracing::error!("not impl store engine: {}", &cli_args.store_engine);
            return Box::new(not_impl::NotImplStore::new(cli_args));
        }
    }
}
