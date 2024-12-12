pub mod http;
pub mod sip;
pub mod store;
pub mod utils;
pub mod version;
use clap::Parser;
use std::{path::PathBuf, process::exit};
use tracing::{self, error};
use utils::config::Config;

pub mod gss {
    tonic::include_proto!("gss");
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_name = "CONFIG_FILE_PATH")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if !args.config.is_file() {
        error!("config file is not existed: {}", args.config.display());
        exit(1);
    }

    let config = Config::load_from_file(&args.config).unwrap();

    // open daily log
    let _ = utils::log::init(&config);

    // prepare sip server
    let (sip_udp_socket, sip_tcp_listener) = sip::server::bind(&config).await?;

    // connect store
    let store_engine = store::create_store(&config);
    if !store_engine.is_connected() {
        tracing::error!("create_store error");
        return Ok(());
    }

    // run sip server
    let sip_handler = sip::handler::SipHandler::new(&config, store_engine, sip_udp_socket, sip_tcp_listener);
    let sip_handler_arc = std::sync::Arc::new(sip_handler);
    let sip_service = sip::server::run_forever(config.clone(), sip_handler_arc.clone());

    // run http server
    let http_service = http::server::run_forever(&config, sip_handler_arc);

    // wait
    let _ = tokio::join!(sip_service, http_service);

    Ok(())
}
