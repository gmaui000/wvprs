use std::str::FromStr;

use rsip::{
    self,
    prelude::{HeadersExt, UntypedHeader},
};

use sdp_rs;

use super::SipHandler;

impl SipHandler {
    pub async fn on_req_invite(
        &self,
        _device_addr: std::net::SocketAddr,
        _tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        _request: rsip::Request,
    ) {
    }

    pub async fn on_rsp_invite(
        &self,
        device_addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        response: rsip::Response,
    ) {
        if &rsip::StatusCode::Trying == response.status_code() {
            self.on_rsp_invite_100(device_addr, tcp_stream, response)
                .await;
        } else if &rsip::StatusCode::OK == response.status_code() {
            self.on_rsp_invite_200(device_addr, tcp_stream, response)
                .await;
        } else {
            tracing::warn!("unexpected response, method: {}", response.status_code());
        }
    }

    pub async fn on_rsp_invite_100(
        &self,
        _device_addr: std::net::SocketAddr,
        _tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        _response: rsip::Response,
    ) {
    }

    pub async fn on_rsp_invite_200(
        &self,
        device_addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        response: rsip::Response,
    ) {
        // decode body
        let sdp_msg = self.decode_body(response.body());

        match sdp_rs::SessionDescription::from_str(&sdp_msg) {
            Err(e) => {
                tracing::error!("sdp_rs::SessionDescription::from_str error, e: {:?}", e);
            }
            Ok(_) => {
                if let Some(gb_code) = self
                    .store
                    .find_gb_code_by_caller_id(response.call_id_header().unwrap().clone().value())
                {
                    let mut headers: rsip::Headers = Default::default();
                    headers.push(response.via_header().unwrap().clone().into());
                    headers.push(response.from_header().unwrap().clone().into());
                    headers.push(self.to_old(response.to_header().unwrap()).into());
                    headers.push(response.call_id_header().unwrap().clone().into());
                    headers.push(response.cseq_header().unwrap().clone().into());
                    headers.push(rsip::Header::ContentLength(Default::default()));

                    let request = rsip::Request {
                        method: rsip::Method::Ack,
                        uri: rsip::Uri {
                            scheme: Some(rsip::Scheme::Sip),
                            auth: Some((gb_code.clone(), Option::<String>::None).into()),
                            host_with_port: rsip::Domain::from(self.domain.clone()).into(),
                            ..Default::default()
                        },
                        headers,
                        version: rsip::Version::V2,
                        body: Default::default(),
                    };

                    self.socket_send_request(device_addr, tcp_stream, request)
                        .await;
                } else {
                    tracing::error!("device not found: {}", device_addr);
                }
            }
        }
    }
}
