use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayStopRequest {
    pub gb_code: String,
    pub stream_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayStopResponse {
    pub locate: String,
    pub code: u32,
    pub msg: String,
    pub gb_code: String,
    pub stream_id: u32,
}
