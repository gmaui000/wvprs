use actix_web::{post, web, Responder};

use crate::{
    http::message::live::stop::{LiveStopRequest, LiveStopResponse},
    sip::handler::SipHandler,
};

use crate::gss;

#[post("/live/stop")]
async fn post_stop(
    data: web::Json<LiveStopRequest>,
    sip_handler: web::Data<std::sync::Arc<SipHandler>>,
) -> impl Responder {
    if let Some((
        by_to_device,
        call_id,
        branch,
        device_addr,
        tcp_stream,
        stream_server_ip,
        stream_server_port,
    )) = sip_handler.store.bye(&data.gb_code, data.stream_id)
    {
        if by_to_device {
            sip_handler
                .send_bye(device_addr, tcp_stream, &branch, &call_id, &data.gb_code)
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

                let mut req = gss::FreeStreamPortRequest::default();
                req.gb_code = data.gb_code.clone();
                req.stream_id = data.stream_id;
                req.media_server_ip = stream_server_ip;
                req.media_server_port = stream_server_port as u32;
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
