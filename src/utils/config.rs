use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use local_ip_address::local_ip;
use sysinfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_store_engine")]
    pub store_engine: String,
    #[serde(default = "default_store_url")]
    pub store_url: String,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_my_ip")]
    pub my_ip: String,
    #[serde(default = "default_sip_port")]
    pub sip_port: u16,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_sip_domain")]
    pub sip_domain: String,
    #[serde(default = "default_sip_id")]
    pub sip_id: String,
    #[serde(default = "default_sip_password")]
    pub sip_password: String,
    #[serde(default = "default_sip_algorithm")]
    pub sip_algorithm: String,
    #[serde(default = "default_sip_nonce")]
    pub sip_nonce: String,
    #[serde(default = "default_sip_realm")]
    pub sip_realm: String,
    #[serde(default = "default_socket_recv_buffer_size")]
    pub socket_recv_buffer_size: usize,
    #[serde(default = "default_stream_timeout_seconds")]
    pub stream_timeout_seconds: u32,
    #[serde(default = "default_device_timeout_seconds")]
    pub device_timeout_seconds: u32,
}

fn default_store_engine() -> String {
    "memory".to_string()
}

fn default_store_url() -> String {
    "memory://".to_string()
}

fn default_user_agent() -> String {
    "wvprs".to_string()
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_my_ip() -> String {
    "".to_string()
}

fn default_sip_port() -> u16 {
    5060
}

fn default_http_port() -> u16 {
    6070
}

fn default_sip_domain() -> String {
    "3402000000".to_string()
}

fn default_sip_id() -> String {
    "34020000002000000001".to_string()
}

fn default_sip_password() -> String {
    "d383cf85b0e8ce0b".to_string()
}

fn default_sip_algorithm() -> String {
    "md5".to_string()
}

fn default_sip_nonce() -> String {
    "f89d0eaccaf1c90453e2f84688ec800f05".to_string()
}

fn default_sip_realm() -> String {
    "gbt@future_oriented.com".to_string()
}

fn default_socket_recv_buffer_size() -> usize {
    65535
}

fn default_stream_timeout_seconds() -> u32 {
    180
}

fn default_device_timeout_seconds() -> u32 {
    300
}

impl Config {
    // 从YAML文件加载配置
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_content = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&yaml_content)?;
        if config.my_ip.is_empty() {
            if let Ok(ip) = local_ip() {
                config.my_ip = ip.to_string();
            }
        }

        if config.store_url.starts_with("memory://") {
            let sys = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::everything());
            let mem = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
            config.store_url = format!("memory://main?total={}g", mem.round() as u64);
        }
        Ok(config)
    }

    // 保存配置到YAML文件（示例，可按需完善）
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = serde_yaml::to_string(self)?;
        fs::write(path, yaml_content)?;
        Ok(())
    }
}