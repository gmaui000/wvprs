pub mod ack;
pub mod bye;
pub mod cancel;
pub mod info;
pub mod invite;
pub mod message;
pub mod notify;
pub mod options;
pub mod prack;
pub mod publish;
pub mod refer;
pub mod register;
pub mod subscribe;
pub mod update;

use bytes::Bytes;
use std::net::SocketAddr;
use std::str::FromStr;

use rsip::{
    self,
    prelude::{HasHeaders, HeadersExt, ToTypedHeader},
};

use super::server::{DOUBLE_CR_LF_BYTES, SIP_BYTES};
use crate::store::StoreEngine;
use crate::utils::{color, config::Config};
pub struct SipTransaction {
    pub caller_id: String,
    pub from_tag: String,
    pub to_tag: String,
    pub branch: String,
}

pub struct SipHandler {
    pub ip: String,
    pub port: u16,
    pub domain: String,
    pub id: String,
    pub password: String,
    pub algorithm: rsip::headers::auth::Algorithm,
    pub nonce: String,
    pub realm: String,
    pub store: Box<dyn StoreEngine>,
    pub sip_udp_socket: tokio::net::UdpSocket,
    pub sip_tcp_listener: tokio::net::TcpListener,
}

impl SipHandler {
    pub fn new(
        config: &Config,
        store: Box<dyn StoreEngine>,
        sip_udp_socket: tokio::net::UdpSocket,
        sip_tcp_listener: tokio::net::TcpListener,
    ) -> Self {
        SipHandler {
            ip: config.my_ip.clone(),
            port: config.sip_port,
            domain: config.sip_domain.clone(),
            id: config.sip_id.clone(),
            password: config.sip_password.clone(),
            algorithm: rsip::headers::auth::Algorithm::from_str(&config.sip_algorithm).unwrap(),
            nonce: config.sip_nonce.clone(),
            realm: config.sip_realm.clone(),
            store,
            sip_udp_socket,
            sip_tcp_listener,
        }
    }
}

impl SipHandler {
    pub async fn dispatch(
        &self,
        device_addr: SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        sip_data: &[u8],
    ) {
        // 将 sip_data 转换为 Bytes 类型并修剪前面的空格和制表符
        let bytes = Bytes::copy_from_slice(sip_data);
        let trimmed_sip_data = bytes.trim_ascii_start();

        if trimmed_sip_data.len() == DOUBLE_CR_LF_BYTES.len() && sip_data == DOUBLE_CR_LF_BYTES {
            return;
        }

        if trimmed_sip_data.len() > SIP_BYTES.len()
            && trimmed_sip_data[..SIP_BYTES.len()] == SIP_BYTES
        {
            self.dispatch_response(device_addr, tcp_stream, trimmed_sip_data)
                .await;
        } else {
            self.dispatch_request(device_addr, tcp_stream, trimmed_sip_data)
                .await;
        }
    }

    pub async fn dispatch_request(
        &self,
        device_addr: SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        sip_data: &[u8],
    ) {
        match rsip::Request::try_from(sip_data) {
            Err(e) => {
                tracing::error!(
                    "{}rsip::Request::try_from error, e: {}, {}request: {}",
                    color::RED,
                    e,
                    color::RESET,
                    String::from_utf8_lossy(sip_data)
                );
            }
            Ok(request) => {
                tracing::info!(
                    "{}dispatch {}rsip::Request::try_from({}) ok, amount: {:?}, request:{}\n{}",
                    color::PURPLE,
                    color::CYAN,
                    device_addr,
                    sip_data.len(),
                    color::RESET,
                    format!(
                        "{}{} {} {}{}\n{}{}",
                        color::YELLOW,
                        request.method().to_string(),
                        request.version().to_string(),
                        request.uri().to_string(),
                        color::RESET,
                        request.headers().to_string(),
                        self.decode_body(request.body())
                    )
                );

                let seq = request.cseq_header().unwrap().typed().unwrap().seq;
                if seq > 0 {
                    if request.method() == &rsip::Method::Register {
                        self.store.set_register_sequence(seq);
                    } else {
                        self.store.set_global_sequence(seq);
                    }
                }

                match request.method() {
                    rsip::Method::Register => {
                        self.on_req_register(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Ack => self.on_req_ack(device_addr, tcp_stream, request).await,
                    rsip::Method::Bye => self.on_req_bye(device_addr, tcp_stream, request).await,
                    rsip::Method::Cancel => {
                        self.on_req_cancel(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Info => self.on_req_info(device_addr, tcp_stream, request).await,
                    rsip::Method::Invite => {
                        self.on_req_invite(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Message => {
                        self.on_req_message(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Notify => {
                        self.on_req_notify(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Options => {
                        self.on_req_options(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::PRack => {
                        self.on_req_prack(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Publish => {
                        self.on_req_publish(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Refer => {
                        self.on_req_refer(device_addr, tcp_stream, request).await
                    }
                    rsip::Method::Subscribe => {
                        self.on_req_subscribe(device_addr, tcp_stream, request)
                            .await
                    }
                    rsip::Method::Update => {
                        self.on_req_update(device_addr, tcp_stream, request).await
                    }
                };
            }
        };
    }

    pub async fn dispatch_response(
        &self,
        device_addr: SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        sip_data: &[u8],
    ) {
        match rsip::Response::try_from(sip_data) {
            Err(e) => {
                tracing::error!(
                    "{}rsip::Response::try_from error, e: {}, {}response: {}",
                    color::RED,
                    e,
                    color::RESET,
                    String::from_utf8_lossy(sip_data)
                );
            }
            Ok(response) => {
                tracing::info!(
                    "{}dispatch {}rsip::Response::try_from({}) ok, amount: {:?}, response:{}\n{}",
                    color::PURPLE,
                    color::CYAN,
                    device_addr,
                    sip_data.len(),
                    color::RESET,
                    format!(
                        "{} {}\n{}\n{}",
                        response.version().to_string(),
                        response.status_code().to_string(),
                        response.headers().to_string(),
                        self.decode_body(response.body())
                    )
                );

                if let Ok(seq) = response.cseq_header() {
                    if let Ok(method) = seq.method() {
                        match method {
                            rsip::Method::Register => {
                                self.on_rsp_register(device_addr, response).await
                            }
                            rsip::Method::Ack => {
                                self.on_rsp_ack(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Bye => {
                                self.on_rsp_bye(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Cancel => {
                                self.on_rsp_cancel(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Info => {
                                self.on_rsp_info(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Invite => {
                                self.on_rsp_invite(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Message => {
                                self.on_rsp_message(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Notify => {
                                self.on_rsp_notify(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Options => {
                                self.on_rsp_options(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::PRack => {
                                self.on_rsp_prack(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Publish => {
                                self.on_rsp_publish(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Refer => {
                                self.on_rsp_refer(device_addr, tcp_stream, response).await
                            }
                            rsip::Method::Subscribe => {
                                self.on_rsp_subscribe(device_addr, tcp_stream, response)
                                    .await
                            }
                            rsip::Method::Update => {
                                self.on_rsp_update(device_addr, tcp_stream, response).await
                            }
                        }
                    }
                }
            }
        }
    }
}
