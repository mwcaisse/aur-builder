mod common;

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::{PredicateBooleanExt, predicate};
use std::fs;

use common::create_test_config_file_no_signing;

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

// A repository database archive containing two packages:
//   test-pkg-alpha 1.0.0-1 "First test package"
//   test-pkg-beta 2.0.0-1 "Second test package"
const TEST_DATABASE_PATH: &str = "resources/tests/test-aur.db.tar.xz";

fn setup_repository_with_test_packages(repo_dir: &assert_fs::TempDir) {
    let db_contents = fs::read(TEST_DATABASE_PATH).expect("Failed to read test fixture database");
    let db_path = repo_dir.path().join("test-aur.db.tar.xz");
    fs::write(&db_path, db_contents).expect("Failed to write test database to repo dir");
}

#[test]
fn test_list_shows_all_packages_in_repository() {
    let repo_dir = assert_fs::TempDir::new().unwrap();
    setup_repository_with_test_packages(&repo_dir);
    let config_file =
        create_test_config_file_no_signing("unused", "unused", repo_dir.path().to_str().unwrap());
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
    let config_file =
        create_test_config_file_no_signing("unused", "unused", repo_dir.path().to_str().unwrap());
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
    let config_file =
        create_test_config_file_no_signing("unused", "unused", repo_dir.path().to_str().unwrap());
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
    let config_file =
        create_test_config_file_no_signing("unused", "unused", repo_dir.path().to_str().unwrap());
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
