use chrono;

use time;

use tracing_appender;
use tracing_subscriber;

use crate::utils::ansi_color as Color;

use super::cli::CommandLines;

pub fn open_daily_file_log(app_name: &str, app_version: &str, cli_args: &CommandLines) {
    let mut log_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    log_dir.push("log");

    tracing_subscriber::fmt()
        // .json()
        .with_writer(tracing_appender::rolling::daily(
            &log_dir,
            format!("{app_name}.{sip_port}.log", sip_port = cli_args.sip_port),
        ))
        .with_max_level(tracing::Level::INFO)
        .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
            time::UtcOffset::from_hms(8, 0, 0).unwrap(),
            time::macros::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"
            ),
        ))
        .with_line_number(true)
        .with_thread_ids(true)
        .with_ansi(true)
        .init();

    log_dir.push(format!(
        "{app_name}.{sip_port}.log.{date}",
        sip_port = cli_args.sip_port,
        date = chrono::Local::now().format("%Y-%m-%d")
    ));
    println!(
        "{}logging to: {}{}",
        Color::PURPLE,
        log_dir.to_str().unwrap(),
        Color::RESET
    );

    tracing::info!(
        "start services{}
╔══════════════════════════════════════════════════════════╗
║                         WVP-RS                           ║
║══════════════════════════════════════════════════════════║
║                                                          ║
║                                                          ║
║ version: {:<47} ║
║                                                          ║
║ store_engine: {:<42} ║
║ store_url: {:<45} ║
║ user_agent: {:<44} ║
║ host: {:<50} ║
║ my_ip: {:<49} ║
║ sip_port: {:<46} ║
║ http_port: {:<45} ║
║ sip_domain: {:<44} ║
║ sip_id: {:<48} ║
║ sip_password: {:<42} ║
║ sip_algorithm: {:<41} ║
║ sip_nonce: {:<45} ║
║ sip_realm: {:<45} ║
║ socket_recv_buffer_size: {:<31} ║
╚══════════════════════════════════════════════════════════╝{}",
        Color::PURPLE,
        app_version,
        &cli_args.store_engine,
        &cli_args.store_url,
        &cli_args.user_agent,
        &cli_args.host,
        &cli_args.my_ip,
        &cli_args.sip_port,
        &cli_args.http_port,
        &cli_args.sip_domain,
        &cli_args.sip_id,
        &cli_args.sip_password,
        &cli_args.sip_algorithm,
        &cli_args.sip_nonce,
        &cli_args.sip_realm,
        &cli_args.socket_recv_buffer_size,
        Color::RESET
    );
}
