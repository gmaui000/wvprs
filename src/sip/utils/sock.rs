use tokio;

use tokio::io::AsyncWriteExt;
use tracing;

use crate::sip::handler::SipHandler;
use crate::utils::color;

impl SipHandler {
    pub async fn socket_send_request(
        &self,
        addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        request: rsip::Request,
    ) -> bool {
        return self
            .socket_send(
                addr,
                tcp_stream,
                request.to_string().as_bytes(),
                request.to_string(),
                "request",
            )
            .await;
    }

    pub async fn socket_send_request_with_body(
        &self,
        addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        request: rsip::Request,
        bin_body: Vec<u8>,
        str_body: String,
    ) -> bool {
        let (content, text) = self.build_content(request.to_string(), str_body, bin_body);
        return self
            .socket_send(addr, tcp_stream, content.as_slice(), text, "request")
            .await;
    }

    pub async fn socket_send_response(
        &self,
        addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        response: rsip::Response,
    ) -> bool {
        return self
            .socket_send(
                addr,
                tcp_stream,
                response.to_string().as_bytes(),
                response.to_string(),
                "response",
            )
            .await;
    }
    
    pub async fn socket_send_response_with_body(
        &self,
        addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        response: rsip::Response,
        bin_body: Vec<u8>,
        str_body: String,
    ) -> bool {
        let (content, text) = self.build_content(response.to_string(), str_body, bin_body);
        return self
            .socket_send(addr, tcp_stream, content.as_slice(), text, "response")
            .await;
    }

    pub async fn socket_send(
        &self,
        addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        data: &[u8],
        text: String,
        data_type: &str,
    ) -> bool {
        if let Some(stream) = tcp_stream {
            return self
                .tcp_socket_send(addr, stream, data, text, data_type)
                .await;
        } else {
            return self.udp_socket_send(addr, data, text, data_type).await;
        }
    }

    pub async fn udp_socket_send(
        &self,
        addr: std::net::SocketAddr,
        data: &[u8],
        text: String,
        data_type: &str,
    ) -> bool {
        match self.sip_udp_socket.send_to(data, addr).await {
            Err(e) => {
                tracing::error!(
                    "{}UdpSocket::send_to({}) error, e: {}, {}data: {}",
                    color::RED,
                    addr,
                    e,
                    color::RESET,
                    text
                );
                return false;
            }
            Ok(amount) => {
                tracing::info!(
                    "{}socket_send {}UdpSocket::send_to({}) ok, amount: {}, {}:{}\n{}",
                    color::GREEN,
                    color::CYAN,
                    addr,
                    amount,
                    data_type,
                    color::RESET,
                    text
                );
                return true;
            }
        }
    }

    pub async fn tcp_socket_send(
        &self,
        addr: std::net::SocketAddr,
        tcp_stream: std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>,
        data: &[u8],
        text: String,
        data_type: &str,
    ) -> bool {
        match tcp_stream.lock().await.write_all(data).await {
            Err(e) => {
                tracing::error!(
                    "{}TcpStream::send_to({}) error, e: {}, {}data: {}",
                    color::RED,
                    addr,
                    e,
                    color::RESET,
                    text
                );
                return false;
            }
            Ok(()) => {
                tracing::info!(
                    "{}socket_send {}TcpStream::send_to({}) ok, amount: {}, {}:{}\n{}",
                    color::GREEN,
                    color::CYAN,
                    addr,
                    data.len(),
                    data_type,
                    color::RESET,
                    text
                );
                return true;
            }
        }
    }

    fn build_content(
        &self,
        headers: String,
        str_body: String,
        bin_body: Vec<u8>,
    ) -> (Vec<u8>, String) {
        let mut content: Vec<u8> = vec![];
        content.extend(headers.as_bytes());
        content.extend(bin_body);

        return (content, format!("{headers}{str_body}"));
    }
}
