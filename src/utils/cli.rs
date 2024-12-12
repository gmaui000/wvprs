use local_ip_address::local_ip;

use structopt::StructOpt;

use sysinfo;

use crate::version;

#[derive(Clone, StructOpt)]
pub struct CommandLines {
    #[structopt(long, default_value = "memory", help = "memory, postgresql or redis")]
    pub store_engine: String,

    #[structopt(
        long,
        default_value = "memory://",
        help = "connect url for store_engine, like redis://user:pass@host:port/db or postgresql://user:pass@host:port/db"
    )]
    pub store_url: String,

    #[structopt(long, default_value = version::APP_NAME)]
    pub user_agent: String,

    #[structopt(long, default_value = "0.0.0.0")]
    pub host: String,

    #[structopt(long, default_value = "")]
    pub my_ip: String,

    #[structopt(long, default_value = "5060")]
    pub sip_port: u16,

    #[structopt(long, default_value = "6070")]
    pub http_port: u16,

    #[structopt(long, default_value = "3402000000")]
    pub sip_domain: String,

    #[structopt(long, default_value = "34020000002000000001")]
    pub sip_id: String,

    #[structopt(long, default_value = "d383cf85b0e8ce0b")]
    pub sip_password: String,

    #[structopt(long, default_value = "md5")]
    pub sip_algorithm: String,

    #[structopt(long, default_value = "f89d0eaccaf1c90453e2f84688ec800f05")]
    pub sip_nonce: String,

    #[structopt(long, default_value = "gbt@future_oriented.com")]
    pub sip_realm: String,

    #[structopt(long, default_value = "65535")]
    pub socket_recv_buffer_size: usize,

    #[structopt(long, default_value = "180")]
    pub stream_timeout_seconds: u32,

    #[structopt(long, default_value = "300")]
    pub device_timeout_seconds: u32,
}

impl CommandLines {
    pub fn new(app_name: &str, app_version: &str) -> CommandLines {
        let cli_app = CommandLines::clap().name(app_name).version(app_version);

        let mut results = CommandLines::from_clap(&cli_app.get_matches());
        
        if results.my_ip.is_empty() {
            results.my_ip = local_ip().unwrap().to_string();
        }

        if results.store_url.starts_with("memory://") {
            let sys = sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::everything(),
            );
            let mem = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
            results.store_url = format!("memory://main?total={}g", mem.round() as u64);
        }

        return results;
    }
}
