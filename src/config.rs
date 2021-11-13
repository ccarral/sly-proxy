use crate::error::SlyError;
use crate::target::Target;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
pub struct AppConfig {
    pub name: String,
    pub listen_on: Vec<u16>,
    pub target: Vec<Target>,
}

impl AppConfig {
    pub fn ports(&self) -> &[u16] {
        &self.listen_on
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<AppConfig, SlyError> {
        let file_contents =
            read_to_string(path).map_err(|_| SlyError::Config("Couldn't find file".to_string()))?;
        let config = toml::from_str(&file_contents).map_err(|e| SlyError::Config(e.to_string()))?;
        Ok(config)
    }
}

pub fn get_default_config() -> Result<AppConfig, SlyError> {
    AppConfig::from_file("sly.toml")
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::*;
    #[test]
    fn test_from_file() {
        let target = ["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"]
            .into_iter()
            .map(|addr| {
                let addr = addr.parse::<SocketAddr>().unwrap();
                Target::from_sock_addr(&addr)
            })
            .collect::<Vec<Target>>();

        let from_file = AppConfig::from_file("examples/sly.toml").unwrap();
        assert_eq!(
            from_file,
            AppConfig {
                name: "caca-app".to_string(),
                listen_on: vec![8083, 8084],
                target
            }
        )
    }
}
