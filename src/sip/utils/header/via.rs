use rsip::{self, prelude::ToTypedHeader};

use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn via(&self, transport: rsip::Transport, branch: &String) -> rsip::headers::Via {
        rsip::typed::Via {
            version: rsip::Version::V2,
            transport,
            uri: rsip::Uri {
                host_with_port: (self.ip.clone(), self.port).into(),
                ..Default::default()
            },
            params: vec![
                rsip::Param::Other(rsip::param::OtherParam::new("rport"), None),
                rsip::param::Branch::new(branch).into(),
            ],
        }
        .into()
    }

    pub fn branch_get(&self, via: &rsip::headers::Via) -> String {
        if let Ok(tv) = via.typed() {
            if let Some(branch) = tv.branch() {
                return branch.to_string();
            }
        }

        String::new()
    }

    pub fn transport_get(&self, via: &rsip::headers::Via) -> rsip::Transport {
        if let Ok(tv) = via.typed() {
            return tv.transport;
        }

        rsip::Transport::Udp
    }
}
