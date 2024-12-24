use super::SipHandler;
use crate::sip::message::Catalog;
use rsip::{self, prelude::HeadersExt};

impl SipHandler {
    pub async fn on_catalog(
        &self,
        device_addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        request: rsip::Request,
        msg: String,
    ) {
        let data = Catalog::deserialize_from_xml(msg);
        tracing::debug!("on_catalog: {:?}", data);
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

        // 存储 catalog 信息，例如调用 store 中的某个方法
        self.store.save_catalog(&gb_code, data);

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
