use assert_cmd::cargo_bin_cmd;
use assert_fs::prelude::{PathAssert, PathChild};
use predicates::prelude::{PredicateBooleanExt, predicate};
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

// A repository database archive containing two packages:
//   test-pkg-alpha 1.0.0-1 "First test package"
//   test-pkg-beta 2.0.0-1 "Second test package"
const TEST_DATABASE_PATH: &str = "resources/tests/test-aur.db.tar.xz";

fn setup_repository_with_test_packages(repo_dir: &assert_fs::TempDir) {
    let db_contents =
        fs::read(TEST_DATABASE_PATH).expect("Failed to read test fixture database");
    let db_path = repo_dir.path().join("test-aur.db.tar.xz");
    fs::write(&db_path, db_contents).expect("Failed to write test database to repo dir");
}

#[test]
fn test_list_shows_all_packages_in_repository() {
    let repo_dir = assert_fs::TempDir::new().unwrap();
    setup_repository_with_test_packages(&repo_dir);
    let config_file = create_test_config_file_no_signing(
        "unused",
        "unused",
        repo_dir.path().to_str().unwrap(),
    );
    let config_path = config_file.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-pkg-alpha"))
        .stdout(predicate::str::contains("1.0.0-1"))
        .stdout(predicate::str::contains("First test package"))
        .stdout(predicate::str::contains("test-pkg-beta"))
        .stdout(predicate::str::contains("2.0.0-1"))
        .stdout(predicate::str::contains("Second test package"));
}

#[test]
fn test_search_shows_packages_matching_term() {
    let repo_dir = assert_fs::TempDir::new().unwrap();
    setup_repository_with_test_packages(&repo_dir);
    let config_file = create_test_config_file_no_signing(
        "unused",
        "unused",
        repo_dir.path().to_str().unwrap(),
    );
    let config_path = config_file.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);
    cmd.arg("search");
    cmd.arg("test");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-pkg-alpha"))
        .stdout(predicate::str::contains("1.0.0-1"))
        .stdout(predicate::str::contains("First test package"))
        .stdout(predicate::str::contains("test-pkg-beta"))
        .stdout(predicate::str::contains("2.0.0-1"))
        .stdout(predicate::str::contains("Second test package"));
}

#[test]
fn test_search_only_shows_packages_matching_term() {
    let repo_dir = assert_fs::TempDir::new().unwrap();
    setup_repository_with_test_packages(&repo_dir);
    let config_file = create_test_config_file_no_signing(
        "unused",
        "unused",
        repo_dir.path().to_str().unwrap(),
    );
    let config_path = config_file.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);
    cmd.arg("search");
    cmd.arg("beta");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-pkg-beta"))
        .stdout(predicate::str::contains("Second test package"))
        .stdout(predicate::str::contains("test-pkg-alpha").not())
        .stdout(predicate::str::contains("First test package").not());
}

#[test]
fn test_search_shows_no_packages_when_nothing_matches() {
    let repo_dir = assert_fs::TempDir::new().unwrap();
    setup_repository_with_test_packages(&repo_dir);
    let config_file = create_test_config_file_no_signing(
        "unused",
        "unused",
        repo_dir.path().to_str().unwrap(),
    );
    let config_path = config_file.path().to_str().unwrap();

    let mut cmd = cargo_bin_cmd!("aur-builder");
    cmd.arg("--config");
    cmd.arg(config_path);
    cmd.arg("search");
    cmd.arg("nonexistent-package");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-pkg-alpha").not())
        .stdout(predicate::str::contains("test-pkg-beta").not());
}
