use serde::{Deserialize, Serialize};
use serde_xml_rs::{from_str, to_string};

use tracing;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlarmStatus {
    #[serde(rename = "Num")]
    num: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeviceInfo {
    #[serde(rename = "CmdType")]
    pub cmd_type: String,
    #[serde(rename = "SN")]
    pub sn: u32,
    #[serde(rename = "DeviceID")]
    pub device_id: String,
    #[serde(rename = "DeviceName")]
    pub device_name: String,
    #[serde(rename = "Manufacturer")]
    pub manufacturer: String,
    #[serde(rename = "Model")]
    pub model: String,
    #[serde(rename = "Firmware")]
    pub firmware: String,
    #[serde(rename = "Result")]
    pub result: String,
}

impl DeviceInfo {
    pub fn serialize_to_xml(&self) -> String {
        match to_string(self) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("serde_xml_rs::to_string({:?}) error, e: {:?}", self, e);
                String::new()
            }
        }
    }

    pub fn deserialize_from_xml(s: String) -> Self {
        match from_str(s.as_str()) {
            Ok(k) => k,
            Err(e) => {
                tracing::error!("serde_xml_rs::from_str({}) error, e: {:?}", s, e);
                DeviceInfo::default()
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "Query")]
pub struct DeviceInfoQuery {
    #[serde(rename = "CmdType")]
    pub cmd_type: String,
    #[serde(rename = "SN")]
    pub sn: u32,
    #[serde(rename = "DeviceID")]
    pub device_id: String,
}

impl DeviceInfoQuery {
    pub fn new(sn: u32, gb_code: &str) -> Self {
        DeviceInfoQuery {
            cmd_type: String::from("DeviceInfo"),
            sn,
            device_id: gb_code.to_string(),
        }
    }

    pub fn serialize_to_xml(&self) -> String {
        match to_string(self) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("serde_xml_rs::to_string({:?}) error, e: {:?}", self, e);
                String::new()
            }
        }
    }

    pub fn deserialize_from_xml(s: String) -> Self {
        match from_str(s.as_str()) {
            Ok(k) => k,
            Err(e) => {
                tracing::error!("serde_xml_rs::from_str({}) error, e: {:?}", s, e);
                DeviceInfoQuery::default()
            }
        }
    }
}
