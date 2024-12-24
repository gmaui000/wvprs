use crate::sip::handler::SipHandler;
use crate::{sip, version};
use rsip::{self, prelude::UntypedHeader};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

// 封装 send_invite 函数所需的参数
pub struct SendInviteParams {
    pub device_addr: SocketAddr,
    pub tcp_stream: Option<Arc<Mutex<OwnedWriteHalf>>>,
    pub branch: String,
    pub channel_id: String,
    pub caller_id: String,
    pub media_server_ip: String,
    pub media_server_port: u16,
    pub session_type: sip::message::SdpSessionType,
    pub gb_code: String,
    pub setup_type: String,
    pub start_ts: u64,
    pub stop_ts: u64,
}

impl SipHandler {
    pub async fn send_invite(&self, params: SendInviteParams) -> bool {
        // 提取参数
        let SendInviteParams {
            device_addr,
            tcp_stream,
            branch,
            channel_id,
            caller_id,
            media_server_ip,
            media_server_port,
            session_type,
            gb_code,
            setup_type,
            start_ts,
            stop_ts,
        } = params;

        // body
        let str_body = sip::message::generate_media_sdp(
            &media_server_ip,
            media_server_port,
            &gb_code,
            &setup_type,
            session_type,
            start_ts,
            stop_ts,
        );
        let bin_body = str_body.as_bytes().to_vec();

        // headers
        let headers = self.build_headers(
            &branch,
            &channel_id,
            &caller_id,
            &gb_code,
            tcp_stream.as_ref(),
            &str_body,
        );

        // request
        let request = self.build_request(&gb_code, &headers, &bin_body);

        self.socket_send_request_with_body(device_addr, tcp_stream, request, bin_body, str_body)
            .await
    }

    fn build_headers(
        &self,
        branch: &str,
        channel_id: &str,
        caller_id: &str,
        gb_code: &str,
        tcp_stream: Option<&Arc<Mutex<OwnedWriteHalf>>>,
        str_body: &str,
    ) -> rsip::Headers {
        let mut headers: rsip::Headers = Default::default();
        headers.push(
            self.via(
                if tcp_stream.is_some() {
                    rsip::Transport::Tcp
                } else {
                    rsip::Transport::Udp
                },
                &branch.to_string(),
            )
            .into(),
        );
        headers.push(rsip::headers::MaxForwards::default().into());
        headers.push(self.from_new().into());
        headers.push(self.to_new(&gb_code.to_string()).into());
        headers.push(
            rsip::headers::Contact::new(format!("<sip:{}@{}:{}>", self.id, self.ip, self.port))
                .into(),
        );
        headers.push(self.caller_id_from_str(&caller_id.to_string()).into());
        headers.push(
            rsip::typed::CSeq {
                seq: self.store.add_fetch_global_sequence(),
                method: rsip::Method::Invite,
            }
            .into(),
        );
        headers.push(
            rsip::headers::typed::Allow::from(vec![
                rsip::common::Method::Invite,
                rsip::common::Method::Ack,
                rsip::common::Method::Bye,
                rsip::common::Method::Cancel,
                rsip::common::Method::Update,
                rsip::common::Method::PRack,
            ])
            .into(),
        );
        headers.push(rsip::headers::Supported::from(String::from("100rel")).into());
        headers.push(rsip::headers::Subject::from(format!("{channel_id}:0")).into());
        headers.push(
            rsip::headers::UserAgent::from(format!(
                "{} {}",
                version::APP_NAME,
                version::APP_VERSION
            ))
            .into(),
        );
        headers.push(rsip::headers::ContentType::from("APPLICATION/SDP").into());
        headers.push(rsip::headers::ContentLength::from(str_body.len() as u32).into());
        headers
    }

    fn build_request(
        &self,
        gb_code: &str,
        headers: &rsip::Headers,
        _bin_body: &[u8],
    ) -> rsip::Request {
        rsip::Request {
            method: rsip::Method::Invite,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                auth: Some((gb_code.to_string(), Option::<String>::None).into()),
                host_with_port: rsip::Domain::from(self.domain.clone()).into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers.clone(),
            body: Default::default(),
        }
    }
}
