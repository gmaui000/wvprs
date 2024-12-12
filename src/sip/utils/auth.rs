use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn is_authorized(&self, user_name: &String, method: &rsip::Method, uri: &rsip::Uri, digest: &String) -> bool {
        let generator = rsip::services::DigestGenerator {
            username: user_name,
            password: &self.password,
            algorithm: self.algorithm,
            nonce: &self.nonce,
            method: method,
            qop: None,
            uri: &uri,
            realm: &self.realm,
        };

        return generator.verify(digest);
    }
}
