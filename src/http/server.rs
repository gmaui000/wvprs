use actix_web::{web, App, HttpServer};

use tracing;

use crate::http;
use crate::sip::handler::SipHandler;
use crate::utils::config::Config;

pub async fn run_forever(
    config: &Config,
    sip_handler: std::sync::Arc<SipHandler>,
) -> Result<(), std::io::Error> {
    match HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(sip_handler.clone()))
            .service(http::handler::live::post_play)
            .service(http::handler::live::post_stop)
            .service(http::handler::live::post_keep_alive)
            .service(http::handler::replay::post_start)
            .service(http::handler::replay::post_stop)
            .service(http::handler::replay::post_keep_alive)
    })
    .bind((config.host.clone(), config.http_port))
    {
        Ok(h) => {
            tracing::info!(
                "HttpServer::bind({}:{}) ok",
                &config.host,
                config.http_port
            );
            h.run().await
        }
        Err(e) => {
            tracing::error!(
                "HttpServer::bind({}:{}) error, e: {:?}",
                &config.host,
                config.http_port,
                e
            );
            return Err(e);
        }
    }
}
