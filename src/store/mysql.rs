use tokio;
use uuid::Uuid;

use super::StoreEngine;

use crate::utils::config::Config;

pub struct MySqlStore {
    pub quit_flag: bool,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
    pub service_id: String, // random generated on boot, report to load balance
}

impl MySqlStore {
    pub fn new(_config: &Config) -> Self {
        MySqlStore {
            quit_flag: true,
            task_handle: None,
            service_id: Uuid::new_v4().to_string(),
        }
    }
}

impl StoreEngine for MySqlStore {}

unsafe impl Send for MySqlStore {}

unsafe impl Sync for MySqlStore {}
