use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub image: Image,
    pub repository: Repository,
    pub signing: Signing,
    pub additional_trusted_keys: Vec<String>
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    #[serde(default = "default_image_name")]
    pub name: String,
    #[serde(default = "default_image_tag")]
    pub tag: String
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signing {
    pub enabled: bool,
    pub key_path: Option<String>,
    pub public_key_path: Option<String>,
    pub key_id: Option<String>,
}

fn default_image_tag() -> String { "latest".to_string() }
fn default_image_name() -> String { "registry.gitlab.com/mwcaisse/application-images/arch-aur-builder".to_string() }

pub fn read_config(config_file_path: String) -> Config {
    let config_text = std::fs::read_to_string(config_file_path).expect("Failed to read config file");
    let config: Config = toml::from_str(&config_text).expect("Failed to parse config file");

    config
}