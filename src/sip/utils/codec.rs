use encoding_rs;

use textcode;

use tracing;

use crate::sip::handler::SipHandler;

impl SipHandler {
    pub fn decode_body(&self, data: &Vec<u8>) -> String {
        let (body, _encoding, has_error) = encoding_rs::GB18030.decode(data);
        if has_error {
            tracing::error!("encoding_rs::GB18030.decode error");
            return String::new();
        }

        if body.find(r#"encoding="GB2312""#).is_some() {
            return body.replace(r#"encoding="GB2312""#, r#"encoding="UTF-8""#);
        } else if body.find(r#"encoding="GB18030""#).is_some() {
            return body.replace(r#"encoding="GB18030""#, r#"encoding="UTF-8""#);
        }
        return body.to_string();
    }

    pub fn encode_body(&self, data: &String) -> Vec<u8> {
        let s = data.replace(r#"encoding="UTF-8""#, r#"encoding="GB2312""#);
        return textcode::gb2312::encode_to_vec(s.as_str());

        // let (msg, _encoding, has_error) = encoding_rs::GB18030.encode(&s);
        // if has_error {
        //     tracing::error!("encoding_rs::GB18030.encode error");
        //     return vec![];
        // }

        // return msg.to_vec();
    }
}
