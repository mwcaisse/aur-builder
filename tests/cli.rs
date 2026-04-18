use assert_cmd::cargo_bin_cmd;
use assert_fs::prelude::{PathAssert, PathChild};
use predicates::prelude::predicate;
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_errors_when_no_config_file() {
    let mut cmd = cargo_bin_cmd!("aur-builder");

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(
            "Using config from: /etc/aur-builder/config.toml",
        ))
        .stderr(predicate::str::contains("Failed to read config file"))
        .stderr(predicate::str::contains("No such file or directory"));
}

#[test]
fn test_errors_when_config_file_passed_doesnt_exist() {
    let config_path = "/tmp/doesnt-exist.toml";

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(format!(
            "Using config from: {}",
            config_path
        )))
        .stderr(predicate::str::contains("Failed to read config file"))
        .stderr(predicate::str::contains("No such file or directory"));
}

fn create_test_config_file_no_signing(
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

#[test]
fn test_can_create_repo() {
    // TODO: Be able to pull image + tag from env variables, so we can use the latest built image
    //  but for now just pulling the latest image will work

    let repo_dir = assert_fs::TempDir::new().unwrap();
    let config_file = create_test_config_file_no_signing(
        "ghcr.io/mwcaisse/aur-builder",
        "latest",
        repo_dir.path().to_str().unwrap(),
    );
    let config_path = config_file.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);
    cmd.arg("create");

    println!(
        "stdout:\n{}",
        String::from_utf8_lossy(&cmd.output().unwrap().stdout)
    );
    println!(
        "stderr:\n{}",
        String::from_utf8_lossy(&cmd.output().unwrap().stderr)
    );

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Finished creating repository"))
        .stdout(predicate::str::contains(" with status: exit status: 0"));

    repo_dir
        .child("test-aur.db.tar.xz")
        .assert(predicate::path::exists());
}

fn has_package(dir: &Path, package_name: &str) -> bool {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .any(|entry| {
            let binding = entry.file_name();
            let name = binding.to_string_lossy();
            name.starts_with(package_name) && name.ends_with(".pkg.tar.zst")
        })
}

#[test]
fn test_can_add_packages() {
    // TODO: Be able to pull image + tag from env variables, so we can use the latest built image
    //  but for now just pulling the latest image will work

    let repo_dir = assert_fs::TempDir::new().unwrap();
    let config_file = create_test_config_file_no_signing(
        "ghcr.io/mwcaisse/aur-builder",
        "latest",
        repo_dir.path().to_str().unwrap(),
    );
    let config_path = config_file.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);
    cmd.arg("create");

    println!(
        "stdout:\n{}",
        String::from_utf8_lossy(&cmd.output().unwrap().stdout)
    );
    println!(
        "stderr:\n{}",
        String::from_utf8_lossy(&cmd.output().unwrap().stderr)
    );

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Finished creating repository"))
        .stdout(predicate::str::contains(" with status: exit status: 0"));

    let mut add_packages_cmd = cargo_bin_cmd!("aur-builder");
    add_packages_cmd.arg("--config");
    add_packages_cmd.arg(config_path);
    add_packages_cmd.arg("add");
    add_packages_cmd.arg("yay-bin");
    add_packages_cmd.arg("freetube-bin");

    println!(
        "stdout:\n{}",
        String::from_utf8_lossy(&add_packages_cmd.output().unwrap().stdout)
    );
    println!(
        "stderr:\n{}",
        String::from_utf8_lossy(&add_packages_cmd.output().unwrap().stderr)
    );

    add_packages_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Finished syncing packages! with status: exit status: 0",
        ))
        .stdout(predicate::str::contains(
            "Finished adding packages! with status: exit status: 0",
        ));

    assert!(has_package(repo_dir.path(), "yay-bin"));
    assert!(has_package(repo_dir.path(), "freetube-bin"));
}
