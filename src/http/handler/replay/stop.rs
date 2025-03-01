use actix_web::{post, web, Responder};

use crate::{
    http::message::replay::stop::{ReplayStopRequest, ReplayStopResponse},
    sip::handler::{SipHandler, SipTransaction},
    store::ByeResult,
};

#[post("/replay/stop")]
async fn post_stop(
    data: web::Json<ReplayStopRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    if let Some(ByeResult {
        success,
        caller_id,
        from_tag,
        to_tag,
        branch,
        socket_addr,
        tcp_stream,
    }) = sip_handler.store.bye(&data.gb_code, data.stream_id)
    {
        if success {
            sip_handler
                .send_bye(
                    socket_addr,
                    tcp_stream,
                    SipTransaction {
                        caller_id,
                        from_tag,
                        to_tag,
                        branch,
                    },
                    &data.gb_code,
                )
                .await;
        }
    }

    let result = ReplayStopResponse {
        locate: format!("{}#L{}", file!(), line!()),
        code: 0,
        msg: String::from("OK"),
        gb_code: data.gb_code.clone(),
        stream_id: data.stream_id,
    };
    web::Json(result)
}
