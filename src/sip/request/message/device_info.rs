use crate::sip::handler::SipHandler;
use crate::{sip, version};

impl SipHandler {
    pub async fn send_device_info_query(
        &self,
        device_addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        transport: rsip::Transport,
        branch: &String,
        gb_code: &String,
    ) -> bool {
        // body
        let text_body =
            sip::message::DeviceInfoQuery::new(self.store.add_fetch_global_sn(), gb_code)
                .serialize_to_xml();
        let bin_body = self.encode_body(&text_body);

        // headers
        let mut headers: rsip::Headers = Default::default();
        headers.push(self.via(transport, branch).into());
        headers.push(rsip::headers::MaxForwards::default().into());
        headers.push(self.from_new().into());
        headers.push(self.to_new(gb_code).into());
        headers.push(self.caller_id_new().into());
        headers.push(
            rsip::typed::CSeq {
                seq: self.store.add_fetch_global_sequence(),
                method: rsip::Method::Message,
            }
            .into(),
        );
        headers.push(
            rsip::headers::UserAgent::from(format!(
                "{} {}",
                version::APP_NAME,
                version::APP_VERSION
            ))
            .into(),
        );
        headers.push(
            rsip::typed::ContentType(rsip::headers::typed::MediaType::Other(
                "Application/MANSCDP+xml".into(),
                vec![]
            ))
            .into()
        );
        headers.push(rsip::headers::ContentLength::from(bin_body.len() as u32).into());

        // request
        let request = rsip::Request {
            method: rsip::Method::Message,
            uri: rsip::Uri {
                scheme: Some(rsip::Scheme::Sip),
                auth: Some((gb_code.clone(), Option::<String>::None).into()),
                host_with_port: rsip::Domain::from(self.domain.clone()).into(),
                ..Default::default()
            },
            version: rsip::Version::V2,
            headers: headers,
            body: Default::default(),
        };

        return self
            .socket_send_request_with_body(device_addr, tcp_stream, request, bin_body, text_body)
            .await;
    }
}
