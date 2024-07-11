use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use crate::error::HVMError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub zk_params_path: String,
    pub state_db_path: String,
    // Add other configuration parameters as needed
}

impl Config {
    pub fn load() -> Result<Self, HVMError> {
        let mut file = File::open("config.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    }
}