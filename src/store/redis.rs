use tokio;
use uuid::Uuid;

use super::StoreEngine;

use crate::utils::cli::CommandLines;

pub struct RedisStore {
    pub quit_flag: bool,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
    pub service_id: String, // random generated on boot, report to load balance
}

impl RedisStore {
    pub fn new(_cli_args: &CommandLines) -> Self {
        RedisStore {
            quit_flag: true,
            task_handle: None,
            service_id: Uuid::new_v4().to_string(),
        }
    }
}

impl StoreEngine for RedisStore {}

unsafe impl Send for RedisStore {}

unsafe impl Sync for RedisStore {}
