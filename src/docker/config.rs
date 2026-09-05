use crate::config::NonEmptyString;
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DockerConfig {
    pub repository: Repository,
    pub signing: Signing,
    pub additional_trusted_keys: Vec<NonEmptyString>,
}

#[derive(Deserialize, Serialize)]
pub struct Repository {
    pub name: NonEmptyString,
    pub path: NonEmptyString,
}

#[derive(Deserialize, Serialize)]
pub struct Signing {
    pub enabled: bool,
    pub key_path: Option<NonEmptyString>,
    pub public_key_path: Option<NonEmptyString>,
}

pub fn read_docker_config(config_file_path: String) -> anyhow::Result<DockerConfig> {
    let config_text = std::fs::read_to_string(&config_file_path)
        .with_context(|| format!("Failed to read docker config file: {}", config_file_path))?;
    let config: DockerConfig = toml::from_str(&config_text)
        .context("Failed to parse docker config file")?;

    Ok(config)
}

pub fn write_docker_config_to_file(config: &DockerConfig, config_file_path: &str) -> anyhow::Result<()> {
    let config_text = toml::to_string_pretty(config)
        .context("Failed to serialize docker config")?;
    std::fs::write(config_file_path, config_text)
        .with_context(|| format!("Failed to write docker config file: {}", config_file_path))?;

    Ok(())
}
