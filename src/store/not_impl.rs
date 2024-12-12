use tokio;
use uuid::Uuid;

use super::StoreEngine;

use crate::utils::cli::CommandLines;

pub struct NotImplStore {
    pub quit_flag: bool,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
    pub service_id: String, // random generated on boot, report to load balance
}

impl NotImplStore {
    pub fn new(_cli_args: &CommandLines) -> Self {
        NotImplStore {
            quit_flag: true,
            task_handle: None,
            service_id: Uuid::new_v4().to_string(),
        }
    }
}

impl StoreEngine for NotImplStore {}

unsafe impl Send for NotImplStore {}

unsafe impl Sync for NotImplStore {}
