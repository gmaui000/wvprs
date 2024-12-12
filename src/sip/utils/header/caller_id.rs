use rsip::{self, prelude::UntypedHeader};

use uuid::Uuid;

use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn caller_id_new(&self) -> rsip::headers::CallId {
        format!(
            "{}@{}:{}",
            Uuid::new_v4().to_string().replace("-", "").to_uppercase(),
            self.ip,
            self.port
        )
        .into()
    }

    pub fn caller_id_str(&self) -> String {
        format!(
            "{}@{}:{}",
            Uuid::new_v4().to_string().replace("-", "").to_uppercase(),
            self.ip,
            self.port
        )
    }

    pub fn caller_id_from_str(&self, s: &String) -> rsip::headers::CallId {
        rsip::headers::CallId::new(s)
    }
}
