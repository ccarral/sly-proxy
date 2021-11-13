use crate::error::SlyError;
use crate::target::Target;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub listen_on: Vec<u16>,
    pub targets: Vec<Target>,
}

impl AppConfig {
    pub fn ports(&self) -> &[u16] {
        &self.listen_on
    }

    pub fn from_file<P: AsRef<Path>>() -> Result<AppConfig, SlyError> {
        unimplemented!();
    }
}
