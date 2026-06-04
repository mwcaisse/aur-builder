use crate::actions::create_repository_file_path_from_path_name;
use crate::docker::config::DockerConfig;
use crate::package_parser;
use std::process::Command;
use tempfile::TempDir;

pub fn update_custom_packages(config: &DockerConfig) {
    // TODO: Implement this
    // This will be used to update the custom packages listed in config

    // to build a custom package, we need to:
    //  clone the repo
    //  run makepkg -s
    // add the resulting package to the repo
}

pub fn rebuild_all_custom_packages(config: &DockerConfig) {
    // TODO: implement this
}

/// Updates the given function if it needs to be updated
fn update_package(package_name: &str, package_repository: &str, config: &DockerConfig) {
    // TODO: Need to create the directory
    let working_dir = TempDir::new().unwrap();
    let working_dir_path = working_dir.path().to_str().unwrap();

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

    // build the package
    // add the package to the repository
}

/// Clones the given repository into the given directory
fn clone_repository(repository_url: &str, directory: &str) {
    let command = Command::new("git")
        .arg("clone")
        .arg(repository_url)
        .arg(directory)
        .status()
        .expect("Failed to execute git clone command");
}

/// Given a directory that contains the PKGBUILD, return the version of the package
fn get_package_version(repo_directory: &str) -> Result<String, &'static str> {
    let output = Command::new("makepkg")
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
    let output = Command::new("vercmp")
        .arg(ver1)
        .arg(ver2)
        .output()
        .expect("Failed to execute vercmp command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let number: i32 = stdout.trim().parse().unwrap();
    Ok(number)
}
