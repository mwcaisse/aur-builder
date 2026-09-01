use crate::actions::create_repository_file_path_from_path_name;
use crate::docker::common_actions::{command_as_build_user, take_ownership_of_directory};
use crate::docker::config::DockerConfig;
use crate::docker::constants::BUILD_USER;
use crate::package_parser;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub fn update_custom_packages(config: &DockerConfig) {
    println!("Updating custom packages");
    for (package_name, package_repo) in &config.custom_packages {
        println!(
            "Updating package: {} from repository: {}",
            package_name,
            package_repo.as_str()
        );
        update_package(package_name.as_str(), package_repo.as_str(), config);
    }
}

pub fn rebuild_all_custom_packages(config: &DockerConfig) {
    // TODO: implement this
}

/// Updates the given function if it needs to be updated
fn update_package(package_name: &str, package_repository: &str, config: &DockerConfig) {
    // TODO: Need to create the directory
    let working_dir = TempDir::new().unwrap();
    let working_dir_path = working_dir.path().to_str().unwrap();
    take_ownership_of_directory(working_dir_path, BUILD_USER, BUILD_USER);

    clone_repository(package_repository, working_dir_path);
    let package_version = get_package_version(working_dir_path).unwrap();

    let repo_path = create_repository_file_path_from_path_name(
        config.repository.path.as_str(),
        config.repository.name.as_str(),
    );
    let repo_package_version = get_version_of_package_in_repo(package_name, repo_path.as_str());

    if !is_package_newer(&package_version, repo_package_version.as_deref()) {
        return;
    }

    let package_files = build_package(working_dir_path, config).unwrap();

    add_package_files_to_repo(
        working_dir_path,
        &package_files,
        config.repository.path.as_str(),
        repo_path.as_str(),
    );
}

/// Clones the given repository into the given directory
fn clone_repository(repository_url: &str, directory: &str) {
    command_as_build_user("git")
        .arg("clone")
        .arg(repository_url)
        .arg(directory)
        .status()
        .expect("Failed to execute git clone command");
}

/// Given a directory that contains the PKGBUILD, return the version of the package
fn get_package_version(repo_directory: &str) -> Result<String, &'static str> {
    let output = command_as_build_user("makepkg")
        .arg("--packagelist")
        .current_dir(repo_directory)
        .output()
        .expect("Failed to execute makepkg to get package version");

    let output_text = String::from_utf8_lossy(&output.stdout);
    let first_line = output_text.lines().next().unwrap();

    parse_version_from_package_name(first_line)
}

/// Parses the version from the package name / the first line out output from makepkg --packagelist
fn parse_version_from_package_name(package_name: &str) -> Result<String, &'static str> {
    let clean_name = package_name.strip_suffix(".pkg.tar.zst").unwrap();
    let parts: Vec<&str> = clean_name.rsplitn(4, "-").collect();
    let version = parts.get(2).copied();
    let pkgrel = parts.get(1).copied();
    Ok(format!("{}-{}", version.unwrap(), pkgrel.unwrap()))
}

fn get_version_of_package_in_repo(package_name: &str, repo_path: &str) -> Option<String> {
    let packages = package_parser::get_packages_from_arch_database(repo_path);
    let our_package = packages.iter().find(|p| p.name == package_name);
    match our_package {
        Some(package) => Some(package.version.clone()),
        None => None,
    }
}

fn is_package_newer(package_version: &str, current_version: Option<&str>) -> bool {
    // If current_version is None, then the package is not in the repository, so it is newer
    if current_version.is_none() {
        return true;
    }

    let ver_cmp_result = run_ver_cmp(package_version, current_version.unwrap());
    match ver_cmp_result {
        Ok(cmp_result) => cmp_result > 0,
        Err(err) => {
            println!("Failed to compare versions: {}", err);
            false
        }
    }
}

/// < 0  if ver1 <  ver2
///   0  if ver1 == ver 2
/// > 0  if ver1 >  ver 2
fn run_ver_cmp(ver1: &str, ver2: &str) -> Result<i32, &'static str> {
    let output = command_as_build_user("vercmp")
        .arg(ver1)
        .arg(ver2)
        .output()
        .expect("Failed to execute vercmp command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let number: i32 = stdout.trim().parse().unwrap();
    Ok(number)
}

/// Builds the package in the given directory
fn build_package(
    package_directory: &str,
    config: &DockerConfig,
) -> Result<Vec<String>, &'static str> {
    let mut command = command_as_build_user("makepkg");

    command.current_dir(package_directory);
    command.arg("--clean");
    command.arg("--syncdeps");
    command.arg("--noconfirm");

    if config.signing.enabled {
        command.arg("--sign");
    }

    command.status().expect("Failed to execute makepkg command");

    let mut package_files_command = command_as_build_user("makepkg");
    package_files_command.current_dir(package_directory);
    package_files_command.arg("--packagelist");

    let output = package_files_command
        .output()
        .expect("Failed to execute makepkg command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let package_files: Vec<String> = stdout
        .lines()
        .filter(|line| !line.contains("-debug"))
        .map(|line| line.to_string())
        .collect();
    Ok(package_files)
}

fn add_package_files_to_repo(
    package_directory: &str,
    package_files: &Vec<String>,
    repo_directory: &str,
    repo_db_path: &str,
) {
    for package_file in package_files {
        let mut package_source_path = PathBuf::from(package_directory);
        package_source_path.push(package_file);

        let package_path = format!("{}/{}", repo_directory, package_file);
        let result = std::fs::copy(
            package_source_path.to_string_lossy().to_string(),
            package_path.as_str(),
        );
        if let Err(e) = result {
            eprintln!("Failed to copy package file {}: {}", package_file, e);
        }

        // now we need to add the package to the database
        let mut add_package_command = Command::new("repo-add");
        add_package_command.arg(repo_db_path);
        add_package_command.arg(package_path.as_str());
        add_package_command
            .status()
            .expect("Failed to add package to repository");
    }
}

//  /tmp/.tmpAdBEBj/yubikey-full-disk-encryption-git-r157.996d52e-1-any.pkg.tar.zst
//                  yubikey-full-disk-encryption-git-r157.996d52e-1-any.pkg.tar.zst
