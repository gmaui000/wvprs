use actix_web::{post, web, Responder};

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
        None => (code, msg) = (404, "device not found"),
        Some(InviteResult {
            success,
            stream_id,
            branch,
            channel_id,
            socket_addr,
            tcp_stream,
        }) => {
            let server_ip = "192.168.31.164";
            let server_port = 10000;
            id = stream_id;
            sip_handler
                .store
                .update_stream_server_info(stream_id, server_ip, server_port);

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
                    media_server_ip: server_ip.to_string(),
                    media_server_port: server_port,
                    session_type: sip::message::sdp::SdpSessionType::Playback,
                    gb_code: data.gb_code.clone(),
                    setup_type: data.setup_type.clone(),
                    start_ts: data.start_ts,
                    stop_ts: data.stop_ts,
                })
                .await;
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
