use rsip::{self, prelude::HeadersExt};

use super::SipHandler;

use crate::sip::message::DeviceStatus;

impl SipHandler {
    pub async fn on_device_status(
        &self,
        _device_addr: std::net::SocketAddr,
        _tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        request: rsip::Request,
        msg: String,
    ) {
        let data = DeviceStatus::deserialize_from_xml(msg);
        if data.sn > 0 {
            self.store.set_global_sn(data.sn);
        }

        let mut headers: rsip::Headers = Default::default();
        headers.push(request.via_header().unwrap().clone().into());
        headers.push(request.from_header().unwrap().clone().into());
        headers.push(self.to_old(request.to_header().unwrap()).into());
        headers.push(request.call_id_header().unwrap().clone().into());
        headers.push(request.cseq_header().unwrap().clone().into());
        headers.push(rsip::Header::ContentLength(Default::default()));
    }
}
