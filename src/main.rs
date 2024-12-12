pub mod http;
pub mod sip;
pub mod store;
pub mod utils;
pub mod version;

pub mod gss {
    tonic::include_proto!("gss");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let cli_args = utils::cli::CommandLines::new(&version::APP_NAME, &version::APP_VERSION);

    // open daily log
    utils::log::open_daily_file_log(&version::APP_NAME, &version::APP_VERSION, &cli_args);

    // prepare sip server
    let (sip_udp_socket, sip_tcp_listener) = sip::server::bind(&cli_args).await?;

    // connect store
    let store_engine = store::create_store(&cli_args);
    if !store_engine.is_connected() {
        tracing::error!("create_store error");
        return Ok(());
    }

    // run sip server
    let sip_handler = sip::handler::SipHandler::new(&cli_args, store_engine, sip_udp_socket, sip_tcp_listener);
    let sip_handler_arc = std::sync::Arc::new(sip_handler);
    let sip_service = sip::server::run_forever(cli_args.clone(), sip_handler_arc.clone());

    // run http server
    let http_service = http::server::run_forever(&cli_args, sip_handler_arc);

    // wait
    let _ = tokio::join!(sip_service, http_service);

    Ok(())
}
