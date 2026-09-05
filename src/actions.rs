use crate::config::{Config, NonEmptyString};
use anyhow::Context;
use crate::docker::commands::DEFAULT_DOCKER_CONFIG_PATH;
use crate::docker::config::{DockerConfig, Repository, Signing, write_docker_config_to_file};
use crate::file_utils::sha256_hash_file;
use crate::package_parser;
use crate::package_parser::Package;
use crate::pgp_utils::get_key_id_from_private_key_file;
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn run_clean(config: &Config, to_keep: u32) -> anyhow::Result<()> {
    println!(
        "Cleaning up old versions of packages! Keeping at most {} versions",
        to_keep
    );

    let clean_status = Command::new("paccache")
        .arg("-rv")
        .arg("-c")
        .arg(&config.repository.path.as_str())
        .arg("-k")
        .arg(to_keep.to_string())
        .status()
        .context("Failed to clean up old versions of packages")?;

    println!(
        "Finished cleaning up old versions of packages! with status: {}",
        clean_status
    );

    Ok(())
}

pub fn run_create_repo(config: &Config) -> anyhow::Result<()> {
    println!(
        "Creating repository at path: {}",
        config.repository.path.as_str()
    );

    let repo_path = create_repository_file_path(config);

    let mut command = Command::new("repo-add");

    add_signing_args_to_repo_command(&mut command, config)?;

    command.arg(&repo_path);

    let status = command.status().context("Failed to create repository")?;

    println!(
        "Finished creating repository {}! with status: {}",
        repo_path, status
    );

    Ok(())
}

pub fn run_remove_packages(config: &Config, packages: &[&str]) -> anyhow::Result<()> {
    println!("Removing the following packages: {:?}", packages);

    let status = remove_packages_internal(config, packages)?;

    println!("Finished removing packages! with status: {}", status);

    Ok(())
}

fn remove_packages_internal(config: &Config, packages: &[&str]) -> anyhow::Result<std::process::ExitStatus> {
    let mut command = Command::new("repo-remove");

    command.arg("--remove");

    add_signing_args_to_repo_command(&mut command, config)?;

    let repo_path = create_repository_file_path(config);
    command.arg(&repo_path);
    command.args(packages);

    let status = command.status().context("Failed to remove packages")?;

    Ok(status)
}

fn add_signing_args_to_repo_command(command: &mut Command, config: &Config) -> anyhow::Result<()> {
    if config.signing.enabled {
        let key_path = config
            .signing
            .key_path
            .as_ref()
            .context("Signing is enabled but key_path is not set")?;
        command.arg("-s");
        command.arg("-k");
        command.arg(
            &get_key_id_from_private_key_file(key_path.as_str())
                .context("Failed to get key id from private key file")?,
        );
    }

    Ok(())
}

pub fn run_remove_orphans(config: &Config) -> anyhow::Result<()> {
    let orphaned_packages = get_orphaned_packages(config)?;

    println!(
        "The following packages are orphaned and will be removed: {:?}",
        orphaned_packages
    );
    print!("Proceed with removing them? [Y/n] ");
    io::stdout().flush().ok();
    let mut input = String::new();
    let read_result = io::stdin().read_line(&mut input);

    if !read_result.is_ok() || input.trim().to_lowercase() != "y" {
        return Ok(());
    }

    let status = remove_packages_internal(
        config,
        orphaned_packages
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice(),
    )?;

    println!(
        "Finished removing orphaned packages! with status: {}",
        status
    );

    Ok(())
}

fn get_orphaned_packages(config: &Config) -> anyhow::Result<Vec<String>> {
    let repo_path = create_repository_file_path(config);
    let our_packages = package_parser::get_packages_from_arch_database(&repo_path)
        .context("Failed to get packages from local repository")?;
    let aur_packages = package_parser::get_all_aur_packages()
        .context("Failed to get all AUR packages")?;

    let mut orphaned_packages: Vec<String> = Vec::new();
    for package in our_packages {
        let package_name = &package.name;
        if !aur_packages.contains(package_name) && !package_name.ends_with("-debug") {
            orphaned_packages.push(package.name);
        }
    }

    Ok(orphaned_packages)
}

fn create_repository_file_path(config: &Config) -> String {
    let mut path = PathBuf::from(config.repository.path.as_str());
    path.push(format!("{}.db.tar.xz", config.repository.name.as_str()));

    path.to_string_lossy().to_string()
}

pub fn run_add_packages(config: &Config, packages: &[&str]) -> anyhow::Result<()> {
    println!("Adding the following packages: {:?}", packages);

    let mut aur_builder_command = vec!["docker", "add"];
    aur_builder_command.extend_from_slice(packages);
    let command_status = run_docker_image(config, &aur_builder_command)?;

    println!("Finished adding packages! with status: {}", command_status);

    Ok(())
}

pub fn run_update(config: &Config) -> anyhow::Result<()> {
    println!("Performing update on all packages!");

    let command_status = run_docker_image(config, &["docker", "update"][..])?;

    println!(
        "Finished updating all packages! with status: {}",
        command_status
    );

    Ok(())
}

pub fn run_rebuild_all(config: &Config) -> anyhow::Result<()> {
    println!("Performing rebuild on all packages!");

    let command_status = run_docker_image(config, &["docker", "rebuild"][..])?;

    println!(
        "Finished rebuilding all packages! with status: {}",
        command_status
    );

    Ok(())
}

pub fn run_rebuild(config: &Config, packages: &[&str]) -> anyhow::Result<()> {
    let mut aur_builder_command = vec!["docker", "rebuild"];
    aur_builder_command.extend_from_slice(packages);
    let command_status = run_docker_image(config, &aur_builder_command)?;

    println!(
        "Finished rebuilding given packages! with status: {}",
        command_status
    );

    Ok(())
}

fn create_docker_image_config(
    config: &Config,
    repository_mount_path: &str,
    signing_key_mount_path: Option<&str>,
    signing_public_key_mount_path: Option<&str>,
) -> DockerConfig {
    DockerConfig {
        repository: Repository {
            name: config.repository.name.clone(),
            path: NonEmptyString::from_known_str(repository_mount_path),
        },
        signing: Signing {
            enabled: config.signing.enabled,
            key_path: signing_key_mount_path.map(NonEmptyString::from_known_str),
            public_key_path: signing_public_key_mount_path.map(NonEmptyString::from_known_str),
        },
        additional_trusted_keys: config.additional_trusted_keys.clone(),
    }
}

fn run_docker_image(config: &Config, aur_builder_command: &[&str]) -> anyhow::Result<std::process::ExitStatus> {
    let docker_image = format!(
        "{}:{}",
        config.image.name.as_str(),
        config.image.tag.as_str()
    );

    if config.image.always_pull {
        let pull_command = Command::new("docker")
            .arg("pull")
            .arg(&docker_image)
            .status()
            .context("Failed to pull docker image")?;

        println!("Pulled image! with status: {}", pull_command);
    }

    let mut update_command = Command::new("docker");

    const REPO_MOUNT_PATH: &str = "/repo";
    const SIGNING_KEY_MOUNT_PATH: &str = "/aur-builder-keys/signing.key";
    const SIGNING_PUBLIC_KEY_MOUNT_PATH: &str = "/aur-builder-keys/signing.pub";

    update_command.arg("run");
    add_mount_arg(
        &mut update_command,
        &config.repository.path.as_str(),
        REPO_MOUNT_PATH,
    );

    println!("Signing enabled!: {}", &config.signing.enabled.to_string());

    if config.signing.enabled {
        let key_path = config
            .signing
            .key_path
            .as_ref()
            .context("Signing is enabled but key_path is not set")?;
        let public_key_path = config
            .signing
            .public_key_path
            .as_ref()
            .context("Signing is enabled but public_key_path is not set")?;

        add_mount_arg(
            &mut update_command,
            &key_path.as_str(),
            SIGNING_KEY_MOUNT_PATH,
        );
        add_mount_arg(
            &mut update_command,
            &public_key_path.as_str(),
            SIGNING_PUBLIC_KEY_MOUNT_PATH,
        );
    }

    let docker_config = create_docker_image_config(
        config,
        REPO_MOUNT_PATH,
        if config.signing.enabled {
            Some(SIGNING_KEY_MOUNT_PATH)
        } else {
            None
        },
        if config.signing.enabled {
            Some(SIGNING_PUBLIC_KEY_MOUNT_PATH)
        } else {
            None
        },
    );

    let docker_config_file =
        NamedTempFile::new().context("Failed to create temporary docker config file")?;

    write_docker_config_to_file(&docker_config, docker_config_file.path().to_str().unwrap())?;

    add_mount_arg(
        &mut update_command,
        docker_config_file.path().to_str().unwrap(),
        DEFAULT_DOCKER_CONFIG_PATH,
    );

    let status = update_command
        .arg(&docker_image)
        .args(aur_builder_command)
        .status()
        .context("Failed to run docker image")?;

    Ok(status)
}

fn add_mount_arg(command: &mut Command, source: &str, destination: &str) {
    command.args([
        "--mount",
        format!("type=bind,source={},destination={}", source, destination).as_str(),
    ]);
}

pub fn list(config: &Config) -> anyhow::Result<()> {
    let repo_path = create_repository_file_path(config);
    let repo_packages = package_parser::get_packages_from_arch_database(&repo_path)
        .context("Failed to get packages from repository")?;

    println!("\n\n");
    print_list_of_packages(repo_packages.iter());

    Ok(())
}

pub fn search(config: &Config, search_term: &str) -> anyhow::Result<()> {
    let repo_path = create_repository_file_path(config);
    let repo_packages = package_parser::get_packages_from_arch_database(&repo_path)
        .context("Failed to get packages from repository")?;

    let filtered_packages = repo_packages.iter().filter(|p| p.name.contains(search_term));

    println!("\n\n");
    print_list_of_packages(filtered_packages);

    Ok(())
}

fn print_list_of_packages<'a>(packages: impl Iterator<Item = &'a Package>) {
    for package in packages {
        println!("{} {}", package.name.bold(), package.version.blue());
        if let Some(description) = &package.description {
            println!("\t{}", description);
        }
    }
}

pub fn validate(config: &Config) -> anyhow::Result<()> {
    let repo_path = create_repository_file_path(config);
    let repo_directory = config.repository.path.as_str();
    let repo_packages = package_parser::get_packages_from_arch_database(&repo_path)
        .context("Failed to get packages from repository")?;

    let invalid_packages = repo_packages
        .iter()
        .filter(|p| !validate_package_checksum(p, &repo_directory))
        .collect::<Vec<_>>();

    if invalid_packages.is_empty() {
        println!("All packages have valid checksums");
        return Ok(());
    }

    println!("The following packages have invalid checksums:");
    for package in invalid_packages.iter() {
        println!("{} {}", package.name.bold(), package.version.blue());
    }

    print!("Proceed with rebuilding them? [Y/n] ");
    io::stdout().flush().ok();
    let mut input = String::new();
    let read_result = io::stdin().read_line(&mut input);

    if !read_result.is_ok() || input.trim().to_lowercase() != "y" {
        return Ok(());
    }

    run_rebuild(
        config,
        invalid_packages
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;
    println!("Rebuilt invalid packages!");

    Ok(())
}

fn validate_package_checksum(package: &Package, repo_directory: &str) -> bool {
    let mut package_file_path = PathBuf::from(repo_directory);
    package_file_path.push(&package.file_name);

    let hash = sha256_hash_file(&package_file_path);

    match hash {
        Ok(hash) => hash.eq_ignore_ascii_case(&package.sha_256_checksum),
        Err(_) => false,
    }
}
