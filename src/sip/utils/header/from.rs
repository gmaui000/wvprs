use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn from_new(&self) -> rsip::headers::From {
        let from_uri = format!("sip:{}@{}", self.id, self.domain);
        rsip::typed::From {
            display_name: None,
            uri: rsip::Uri::try_from(from_uri).unwrap(),
            params: Default::default(),
        }
        .with_tag(self.tag_new(32).into())
        .into()
    }

    pub fn from_old(&self, tag: &str) -> rsip::headers::From {
        let from_uri = format!("sip:{}@{}", self.id, self.domain);
        rsip::typed::From {
            display_name: None,
            uri: rsip::Uri::try_from(from_uri).unwrap(),
            params: Default::default(),
        }
        .with_tag(tag.into())
        .into()
    }
}
