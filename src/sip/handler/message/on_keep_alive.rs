use rsip::{self, prelude::HeadersExt};

use super::SipHandler;

use crate::sip::message::KeepAlive;

impl SipHandler {
    pub async fn on_keep_alive(
        &self,
        device_addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        request: rsip::Request,
        msg: String,
    ) {
        let data = KeepAlive::deserialize_from_xml(msg);
        if data.sn > 0 {
            self.store.set_global_sn(data.sn);
        }

        let gb_code = request
            .from_header()
            .unwrap()
            .uri()
            .unwrap()
            .auth
            .unwrap()
            .to_string();
        self.store.register_keep_alive(&gb_code);

        let mut headers: rsip::Headers = Default::default();
        headers.push(request.via_header().unwrap().clone().into());
        headers.push(request.from_header().unwrap().clone().into());
        headers.push(self.to_old(request.to_header().unwrap()).into());
        headers.push(request.call_id_header().unwrap().clone().into());
        headers.push(request.cseq_header().unwrap().clone().into());
        headers.push(rsip::Header::ContentLength(Default::default()));

        let response = rsip::Response {
            status_code: rsip::StatusCode::OK,
            headers,
            version: rsip::Version::V2,
            body: Default::default(),
        };

        let tcp_stream_ref = &tcp_stream;
        self.socket_send_response(device_addr, tcp_stream_ref.clone(), response)
            .await;
    }
}
