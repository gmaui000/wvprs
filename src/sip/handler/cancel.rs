use super::SipHandler;

impl SipHandler {
    pub async fn on_req_cancel(
        &self,
        _device_addr: std::net::SocketAddr,
        _tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        _request: rsip::Request,
    ) {
    }

    pub async fn on_rsp_cancel(
        &self,
        _device_addr: std::net::SocketAddr,
        _tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        _response: rsip::Response,
    ) {
    }
}
