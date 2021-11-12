use crate::target::Target;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub listen_on: Vec<u16>,
    pub targets: Vec<Target>,
}

impl AppConfig {
    pub fn ports(&mut self) -> &[u16] {
        &self.listen_on
    }
}
