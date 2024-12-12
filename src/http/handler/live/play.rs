use actix_web::{post, web, Responder};

use tonic;

use crate::{
    http::message::live::play::{LivePlayRequest, LivePlayResponse},
    sip::{self, handler::SipHandler},
};

use crate::gss;

#[post("/live/play")]
async fn post_play(
    data: web::Json<LivePlayRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    let (mut code, mut msg) = (200, "OK");

    let mut stream_id = 0;
    let call_id = sip_handler.caller_id_str();
    match sip_handler.store.invite(&data.gb_code, &call_id, true) {
        None => (code, msg) = (404, "ipc device not found"),
        Some((is_playing, id, branch, device_addr, tcp_stream)) => {
            stream_id = id;

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

                    let mut req = gss::BindStreamPortRequest::default();
                    req.gb_code = data.gb_code.clone();
                    req.stream_id = stream_id;
                    req.setup_type = gss::StreamSetupType::from_str_name(&data.setup_type)
                        .unwrap_or(gss::StreamSetupType::Udp)
                        as i32;
                    match client.bind_stream_port(req).await {
                        Err(e) => {
                            tracing::error!("grpc bind_stream_port error, e: {:?}", e);
                        }
                        Ok(response) => {
                            let resp = response.into_inner();
                            if resp.code == gss::ResponseCode::Ok as i32 && resp.message.is_empty()
                            {
                                sip_handler.store.update_stream_server_info(
                                    stream_id,
                                    resp.media_server_ip.clone(),
                                    resp.media_server_port as u16,
                                );

                                if is_playing {
                                    // dispatch
                                }
                                sip_handler
                                    .send_invite(
                                        device_addr,
                                        tcp_stream,
                                        &branch,
                                        &call_id,
                                        &String::from(resp.media_server_ip),
                                        resp.media_server_port as u16,
                                        sip::message::sdp::SdpSessionType::Play,
                                        &data.gb_code,
                                        &data.setup_type,
                                        0,
                                        0,
                                    )
                                    .await;
                            }
                        }
                    }
                }
            };
        }
    };

    let result = LivePlayResponse {
        locate: format!("{}#L{}", file!(), line!()),
        code: code,
        msg: msg.to_string(),
        gb_code: data.gb_code.clone(),
        stream_id: stream_id,
    };
    web::Json(result)
}
