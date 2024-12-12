use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LivePlayRequest {
    pub gb_code: String,
    pub setup_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LivePlayResponse {
    pub locate: String,
    pub code: u32,
    pub msg: String,
    pub gb_code: String,
    pub stream_id: u32,
}

