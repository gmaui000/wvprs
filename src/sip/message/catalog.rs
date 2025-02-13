use serde::{Deserialize, Serialize};
use serde_xml_rs::{from_str, to_string};
use tracing;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "Response")]
pub struct Catalog {
    #[serde(rename = "CmdType")]
    pub cmd_type: String,
    #[serde(rename = "SN")]
    pub sn: u32,
    #[serde(rename = "DeviceID")]
    pub device_id: String,
    #[serde(rename = "SumNum")]
    pub sum_num: u32,
    #[serde(rename = "DeviceList")]
    pub device_list: DeviceList,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "DeviceList")]
pub struct DeviceList {
    #[serde(rename = "Num", default)]
    pub num: u32,
    #[serde(rename = "Item")]
    pub items: Vec<Device>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "Item")]
pub struct Device {
    #[serde(rename = "DeviceID")]
    pub device_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Manufacturer")]
    pub manufacturer: String,
    #[serde(rename = "Model")]
    pub model: String,
    #[serde(rename = "Owner")]
    pub owner: String,
    #[serde(rename = "CivilCode")]
    pub civil_code: String,
    #[serde(rename = "Block")]
    pub block: Option<String>,
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Parental")]
    pub parental: u32,
    #[serde(rename = "ParentID")]
    pub parent_id: String,
    #[serde(rename = "RegisterWay")]
    pub register_way: u32,
    #[serde(rename = "Secrecy")]
    pub secrecy: u32,
    #[serde(rename = "IPAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "Port")]
    pub port: Option<u16>,
    #[serde(rename = "Password")]
    pub password: Option<String>,
    #[serde(rename = "Status")]
    pub status: Option<String>,
    #[serde(rename = "Longitude")]
    pub longitude: Option<f64>,
    #[serde(rename = "Latitude")]
    pub latitude: Option<f64>,
    #[serde(rename = "PTZType")]
    pub ptz_type: Option<u32>,
}

impl Catalog {
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
            Ok(c) => c,
            Err(e) => {
                tracing::error!("serde_xml_rs::from_str({}) error, e: {:?}", s, e);
                Catalog::default()
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "Query")]
pub struct CatalogQuery {
    #[serde(rename = "CmdType")]
    pub cmd_type: String,
    #[serde(rename = "SN")]
    pub sn: u32,
    #[serde(rename = "DeviceID")]
    pub device_id: String,
}

impl CatalogQuery {
    pub fn new(sn: u32, gb_code: &str) -> Self {
        CatalogQuery {
            cmd_type: String::from("Catalog"),
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
                CatalogQuery::default()
            }
        }
    }
}
