use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(value: String) -> Result<Self, &'static str> {
        if value.is_empty() {
            Err("must not be empty")
        } else {
            Ok(Self(value))
        }
    }
    pub fn from_known_str(value: impl Into<String>) -> Self {
        Self::new(value.into()).unwrap()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        NonEmptyString::new(value).map_err(serde::de::Error::custom)
    }
}

impl Serialize for NonEmptyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub image: Image,
    pub repository: Repository,

    #[serde(default)]
    pub signing: Signing,

    #[serde(default)]
    pub additional_trusted_keys: Vec<NonEmptyString>,
}

#[derive(Deserialize)]
pub struct Image {
    #[serde(default = "default_image_name")]
    pub name: NonEmptyString,
    #[serde(default = "default_image_tag")]
    pub tag: NonEmptyString,
    #[serde(default = "default_image_always_pull")]
    pub always_pull: bool,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            name: default_image_name(),
            tag: default_image_tag(),
            always_pull: default_image_always_pull(),
        }
    }
}

#[derive(Deserialize)]
pub struct Repository {
    pub name: NonEmptyString,
    pub path: NonEmptyString,
}

#[derive(Deserialize, Default)]
pub struct Signing {
    #[serde(default)]
    pub enabled: bool,
    pub key_path: Option<NonEmptyString>,
    pub public_key_path: Option<NonEmptyString>,
}

fn default_image_tag() -> NonEmptyString {
    NonEmptyString::new("latest".to_string()).unwrap()
}

fn default_image_name() -> NonEmptyString {
    NonEmptyString::new("latest".to_string()).unwrap()
}

fn default_image_always_pull() -> bool {
    true
}

pub fn read_config(config_file_path: String) -> Config {
    let config_text =
        std::fs::read_to_string(config_file_path).expect("Failed to read config file");
    let config: Config = toml::from_str(&config_text).expect("Failed to parse config file");

    config
}

#[cfg(test)]
mod tests {
    use crate::config::{default_image_always_pull, default_image_name, default_image_tag, Config};
    use pretty_assertions::assert_eq;

    const CONFIG_EXAMPLE: &str = r#"
# Additional maintainer keys to trust before building any packages
additional_trusted_keys = ["5384CE82BA52C83A", "5384CE82BA52C83B"]

[image]
name = "ghcr.io/mwcaisse/aur-builder-dev"
# name = "aur-builder-rust"
tag = "8013968ba2cdadf3a787bc73eae3935e9350e968"
always_pull = true

[repository]
name = "mitchell-aur"
path = "/etc/aur-builder/tmp-repo/"
#path = "/etc/aur-builder/example-files/"

# TODO: Make this a bit better?
#   Do we need both the public + private key? Or will just private suffice?
#   Do we need to have the key id provided separately, or can we derive it from the key itself?
[signing]
enabled = true
key_path = "etc/aur-builder/resources/tests/FD65E82A5CA3DA76E8ECA4977F4989778F99886F.key"
public_key_path = "etc/aur-builder/resources/tests/FD65E82A5CA3DA76E8ECA4977F4989778F99886F.pub"
"#;

    #[test]
    fn test_can_parse_config() {
        let config: Config = toml::from_str(CONFIG_EXAMPLE).expect("Failed to parse config");

        assert_eq!(
            config.image.name.as_str(),
            "ghcr.io/mwcaisse/aur-builder-dev"
        );
        assert_eq!(
            config.image.tag.as_str(),
            "8013968ba2cdadf3a787bc73eae3935e9350e968"
        );
        assert_eq!(config.image.always_pull, true);

        assert_eq!(config.repository.name.as_str(), "mitchell-aur");
        assert_eq!(
            config.repository.path.as_str(),
            "/etc/aur-builder/tmp-repo/"
        );

        assert_eq!(config.signing.enabled, true);

        assert!(config.signing.key_path.is_some());
        assert!(config.signing.public_key_path.is_some());
        assert_eq!(
            config.signing.key_path.unwrap().as_str(),
            "etc/aur-builder/resources/tests/FD65E82A5CA3DA76E8ECA4977F4989778F99886F.key"
        );
        assert_eq!(
            config.signing.public_key_path.unwrap().as_str(),
            "etc/aur-builder/resources/tests/FD65E82A5CA3DA76E8ECA4977F4989778F99886F.pub"
        );

        assert_eq!(config.additional_trusted_keys.len(), 2);
        assert_eq!(
            config.additional_trusted_keys[0].as_str(),
            "5384CE82BA52C83A"
        );
        assert_eq!(
            config.additional_trusted_keys[1].as_str(),
            "5384CE82BA52C83B"
        );
    }

    const MINIMAL_CONFIG: &str = r#"
    [repository]
    name = "mitchell-aur"
    path = "/etc/aur-builder/tmp-repo/"
    "#;

    #[test]
    fn test_can_parse_minimal_config() {
        let config: Config = toml::from_str(MINIMAL_CONFIG).expect("Failed to parse config");

        assert_eq!(config.repository.name.as_str(), "mitchell-aur");
        assert_eq!(
            config.repository.path.as_str(),
            "/etc/aur-builder/tmp-repo/"
        );

        assert_eq!(config.image.name, default_image_name());
        assert_eq!(config.image.tag, default_image_tag());
        assert_eq!(config.image.always_pull, default_image_always_pull());

        assert_eq!(config.signing.enabled, false);
        assert!(config.signing.key_path.is_none());
        assert!(config.signing.public_key_path.is_none());

        assert_eq!(config.additional_trusted_keys.len(), 0);
    }
}
