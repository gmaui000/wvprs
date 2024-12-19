use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn is_authorized(&self, user_name: &String, method: &rsip::Method, uri: &rsip::Uri, qop: Option<&rsip::headers::auth::AuthQop>, digest: &String) -> bool {
        let generator = rsip::services::DigestGenerator {
            username: user_name,
            password: &self.password,
            algorithm: self.algorithm,
            nonce: &self.nonce,
            method: method,
            qop: qop,
            uri: &uri,
            realm: &self.realm,
        };

        tracing::debug!("caobing:{:?} {:?} {:#?}",generator.uri.to_string(), generator.method.to_string(), generator.compute());

        return generator.verify(digest);
    }
}
