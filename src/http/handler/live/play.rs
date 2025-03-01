use actix_web::{post, web, Responder};

use crate::{
    http::message::live::play::{LivePlayRequest, LivePlayResponse},
    sip::{self, handler::SipHandler},
    store::InviteResult,
};

#[post("/live/play")]
async fn post_play(
    data: web::Json<LivePlayRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    let (mut code, mut msg) = (200, "OK");

    let mut id: u32 = 0;
    let call_id = sip_handler.caller_id_str();
    let from_tag = sip_handler.tag_new(32);
    match sip_handler
        .store
        .invite(&data.gb_code, &data.channel_id, &call_id, &from_tag, true)
    {
        None => (code, msg) = (404, "device not found"),
        Some(InviteResult {
            success,
            channel_id,
            stream_id,
            branch,
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
                    session_type: sip::message::sdp::SdpSessionType::Play,
                    gb_code: data.gb_code.clone(),
                    setup_type: data.setup_type.clone(),
                    start_ts: 0,
                    stop_ts: 0,
                })
                .await;
        }
    };

    let result = LivePlayResponse {
        locate: format!("{}#L{}", file!(), line!()),
        code,
        msg: msg.to_string(),
        gb_code: data.gb_code.clone(),
        stream_id: id,
    };
    web::Json(result)
}
