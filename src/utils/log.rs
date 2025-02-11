use super::super::version;
use super::color;
use super::config::Config;
use std::{env, panic};
use time::{macros::format_description, UtcOffset};
use tracing::{self, error};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*};

pub fn init(config: &Config) -> anyhow::Result<Vec<WorkerGuard>> {
    let mut log_dir = env::current_dir().unwrap();
    log_dir.push("logs");
    let file_appender = tracing_appender::rolling::daily(
        &log_dir,
        format!("{app_name}.log", app_name = version::APP_NAME),
    );
    let (filelog, file_log_guard) = tracing_appender::non_blocking(file_appender);
    let (stdoutlog, std_out_guard) = tracing_appender::non_blocking(std::io::stdout());
    let local_time = tracing_subscriber::fmt::time::OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"),
    );

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(stdoutlog.with_max_level(tracing::Level::DEBUG))
                .with_timer(local_time.clone())
                .with_ansi(true)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .pretty(),
        )
        .with(
            fmt::Layer::new()
                .with_writer(filelog.with_max_level(tracing::Level::INFO))
                .with_timer(local_time.clone())
                .with_ansi(false)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true),
        );
    tracing::subscriber::set_global_default(subscriber).map_err(|e| anyhow::anyhow!(e))?;
    tracing::info!(
        "start services{}
╔══════════════════════════════════════════════════════════╗
║               \\\\ || //\\\\    //||===||                ║
║                \\\\||//  \\\\  // ||___||                ║
║                 \\\\//    \\\\//  ||                     ║
║══════════════════════════════════════════════════════════║
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
        color::PURPLE,
        version::APP_VERSION,
        &config.store_engine,
        &config.store_url,
        &config.user_agent,
        &config.host,
        &config.my_ip,
        &config.sip_port,
        &config.http_port,
        &config.sip_domain,
        &config.sip_id,
        &config.sip_password,
        &config.sip_algorithm,
        &config.sip_nonce,
        &config.sip_realm,
        &config.socket_recv_buffer_size,
        color::RESET
    );
    Ok(vec![file_log_guard, std_out_guard])
}

pub fn init_panic() {
    panic::set_hook(Box::new(|panic_info| {
        // 在 panic 发生时记录日志
        error!("Panic occurred: {:?}", panic_info);
    }));
}
