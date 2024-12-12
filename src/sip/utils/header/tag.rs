use rand::Rng;

use crate::sip::handler::SipHandler;

static CHARSET: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

impl SipHandler {
    pub fn tag_new(&self, length: usize) -> String {
        let mut rng = rand::thread_rng();
        std::iter::repeat(())
            .take(length)
            .map(|_| {
                let index = rng.gen_range(0..CHARSET.len());
                CHARSET[index]
            })
            .collect()
    }

    pub fn tag_get(&self, from: &rsip::headers::From) -> String {
        if let Ok(t) = from.tag() {
            if let Some(tag) = t {
                return tag.to_string();
            }
        }

        return String::new();
    }
}
