use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DockerConfig {
    pub repository: Repository,
    pub signing: Signing,
    pub additional_trusted_keys: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Repository {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize, Serialize)]
pub struct Signing {
    pub enabled: bool,
    pub key_path: Option<String>,
    pub public_key_path: Option<String>,
}

pub fn read_docker_config(config_file_path: String) -> DockerConfig {
    let config_text =
        std::fs::read_to_string(config_file_path).expect("Failed to read config file");
    let config: DockerConfig = toml::from_str(&config_text).expect("Failed to parse config file");

    config
}

pub fn write_docker_config_to_file(config: &DockerConfig, config_file_path: &str) {
    let config_text = toml::to_string_pretty(&config).expect("Failed to serialize config");
    std::fs::write(config_file_path, config_text).expect("Failed to write config file");
}
