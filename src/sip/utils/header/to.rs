use rsip::{self, prelude::ToTypedHeader};

use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn to_old(&self, to: &rsip::headers::To) -> rsip::headers::To {
        to.typed()
            .unwrap()
            // .with_tag(self.tag_new(32).into())
            .into()
    }

    pub fn to_new(&self, gb_code: &String) -> rsip::headers::To {
        let from_uri = format!("sip:{}@{}", gb_code, self.domain);
        rsip::typed::To {
            display_name: None,
            uri: rsip::Uri::try_from(from_uri).unwrap(),
            params: Default::default(),
        }
        // .with_tag(self.tag_new(32).into())
        .into()
    }

    pub fn to_new_with_tag(&self, gb_code: &String, tag: &str) -> rsip::headers::To {
        let from_uri = format!("sip:{}@{}", gb_code, self.domain);
        rsip::typed::To {
            display_name: None,
            uri: rsip::Uri::try_from(from_uri).unwrap(),
            params: Default::default(),
        }
        .with_tag(tag.into())
        .into()
    }
}
