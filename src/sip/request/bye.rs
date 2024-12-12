use rsip::{self, prelude::UntypedHeader};

use crate::sip::handler::SipHandler;
use crate::version;

impl SipHandler {
    pub async fn send_bye(
        &self,
        device_addr: std::net::SocketAddr,
        tcp_stream: Option<std::sync::Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>>,
        branch: &String,
        caller_id: &String,
        gb_code: &String,
    ) -> bool {
        // headers
        let mut headers: rsip::Headers = Default::default();
        headers.push(
            self.via(
                if tcp_stream.is_some() {
                    rsip::Transport::Tcp
                } else {
                    rsip::Transport::Udp
                },
                branch,
            )
            .into(),
        );
        headers.push(rsip::headers::MaxForwards::default().into());
        headers.push(self.from_new().into());
        headers.push(self.to_new(gb_code).into());
        headers.push(
            rsip::headers::Contact::new(format!("<sip:{}@{}:{}>", self.id, self.ip, self.port))
                .into(),
        );
        headers.push(self.caller_id_from_str(caller_id).into());
        headers.push(
            rsip::typed::CSeq {
                seq: self.store.add_fetch_global_sequence(),
                method: rsip::Method::Bye,
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

        // request
        let request = rsip::Request {
            method: rsip::Method::Bye,
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
            .socket_send_request(device_addr, tcp_stream, request)
            .await;
    }
}
