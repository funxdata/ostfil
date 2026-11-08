use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ostfil {
    pub uuid: String,
    pub size: u64,
    pub extra: String,
    pub end_time: u64,
    pub md5: String,
    #[serde(default)]
    pub content_type: String,
}

pub const STORAGE_DIR: &str = "./storage";


