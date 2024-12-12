use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayStartRequest {
    pub gb_code: String,
    pub setup_type: String,
    pub start_ts: u64,
    pub stop_ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayStartResponse {
    pub locate: String,
    pub code: u32,
    pub msg: String,
    pub gb_code: String,
    pub stream_id: u32,
}

