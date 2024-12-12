use actix_web::{post, web, Responder};

use crate::{
    http::message::replay::keep_alive::{ReplayKeepAliveRequest, ReplayKeepAliveResponse},
    sip::handler::SipHandler,
};

#[post("/replay/keep_alive")]
async fn post_keep_alive(
    data: web::Json<ReplayKeepAliveRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    sip_handler
        .store
        .stream_keep_alive(&data.gb_code, data.stream_id);

    let result = ReplayKeepAliveResponse {
        locate: format!("{}#L{}", file!(), line!()),
        code: 0,
        msg: String::from("OK"),
        gb_code: data.gb_code.clone(),
        stream_id: data.stream_id,
    };
    web::Json(result)
}
