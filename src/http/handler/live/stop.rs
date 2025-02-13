use actix_web::{post, web, Responder};

use crate::{
    http::message::live::stop::{LiveStopRequest, LiveStopResponse},
    sip::handler::SipHandler,
    store::ByeResult,
};

use crate::gss;

#[post("/live/stop")]
async fn post_stop(
    data: web::Json<LiveStopRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    if let Some(ByeResult {
        success,
        call_id,
        from_tag,
        to_tag,
        branch,
        socket_addr,
        tcp_stream,
        stream_server_ip,
        stream_server_port,
    }) = sip_handler.store.bye(&data.gb_code, data.stream_id)
    {
        if success {
            sip_handler
                .send_bye(
                    socket_addr,
                    tcp_stream,
                    &branch,
                    &call_id,
                    &from_tag,
                    &to_tag,
                    &data.gb_code,
                )
                .await;
        }

        match tonic::transport::Channel::builder("tcp://127.0.0.1:7080".parse().unwrap())
            .connect()
            .await
        {
            Err(e) => {
                tracing::error!("grpc connect error, e: {:?}", e);
            }
            Ok(channel) => {
                let mut client =
                    gss::gbt_stream_service_client::GbtStreamServiceClient::new(channel);

                let req = gss::FreeStreamPortRequest {
                    gb_code: data.gb_code.clone(),
                    stream_id: data.stream_id,
                    media_server_ip: stream_server_ip,
                    media_server_port: stream_server_port as u32,
                };
                match client.free_stream_port(req).await {
                    Err(e) => {
                        tracing::error!("grpc free_stream_port error, e: {:?}", e);
                    }
                    Ok(response) => {
                        let resp = response.into_inner();
                        if resp.code != gss::ResponseCode::Ok as i32 {
                            tracing::error!("grpc free_stream_port error, resp: {:?}", resp);
                        }
                    }
                }
            }
        };
    }

    let result = LiveStopResponse {
        locate: format!("{}#L{}", file!(), line!()),
        code: 0,
        msg: String::from("OK"),
        gb_code: data.gb_code.clone(),
        stream_id: data.stream_id,
    };
    web::Json(result)
}
