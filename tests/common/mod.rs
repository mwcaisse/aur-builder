use tempfile::NamedTempFile;

pub fn create_test_config_file_no_signing(
    image_name: &str,
    image_tag: &str,
    repo_path: &str,
) -> NamedTempFile {
    let config_content = format!(
        r#"
additional_trusted_keys = []

[image]
name = "{}"
tag = "{}"
always_pull = true

[repository]
name = "test-aur"
path = "{}"

[signing]
enabled = false
        "#,
        image_name, image_tag, repo_path
    );

    let config_file = NamedTempFile::new().unwrap();
    let config_file_path = config_file.path().to_str().unwrap();
    std::fs::write(config_file_path, config_content).expect("Failed to write config file");

    config_file
}
