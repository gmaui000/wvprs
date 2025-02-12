use actix_web::{post, web, Responder};

use crate::gss;
use crate::{
    http::message::replay::start::{ReplayStartRequest, ReplayStartResponse},
    sip::{self, handler::SipHandler},
    store::InviteResult,
};

#[post("/replay/start")]
async fn post_start(
    data: web::Json<ReplayStartRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    let (mut code, mut msg) = (200, "OK");

    let mut id = 0;
    let call_id = sip_handler.caller_id_str();
    let from_tag = sip_handler.tag_new(32);
    match sip_handler
        .store
        .invite(&data.gb_code, &data.channel_id, &call_id, &from_tag, true)
    {
        None => (code, msg) = (404, "ipc device not found"),
        Some(InviteResult {
            success,
            stream_id,
            branch,
            channel_id,
            socket_addr,
            tcp_stream,
        }) => {
            id = stream_id;

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

                    let req = gss::BindStreamPortRequest {
                        gb_code: data.gb_code.clone(),
                        stream_id,
                        setup_type: gss::StreamSetupType::from_str_name(&data.setup_type)
                            .unwrap_or(gss::StreamSetupType::Udp)
                            as i32,
                    };
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
                                    &resp.media_server_ip,
                                    resp.media_server_port as u16,
                                );

                                if success {
                                    // dispatch
                                }
                                sip_handler
                                    .send_invite(sip::request::invite::SendInviteParams {
                                        device_addr: socket_addr,
                                        tcp_stream,
                                        branch,
                                        channel_id,
                                        caller_id: call_id,
                                        from_tag,
                                        media_server_ip: resp.media_server_ip,
                                        media_server_port: resp.media_server_port as u16,
                                        session_type: sip::message::sdp::SdpSessionType::Playback,
                                        gb_code: data.gb_code.clone(),
                                        setup_type: data.setup_type.clone(),
                                        start_ts: data.start_ts,
                                        stop_ts: data.stop_ts,
                                    })
                                    .await;
                            }
                        }
                    }
                }
            };
        }
    };

    let result = ReplayStartResponse {
        locate: format!("{}#L{}", file!(), line!()),
        code,
        msg: msg.to_string(),
        gb_code: data.gb_code.clone(),
        stream_id: id,
    };
    web::Json(result)
}
