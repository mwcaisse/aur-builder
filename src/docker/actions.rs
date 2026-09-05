use crate::docker::common_actions::{
    configure_package_signing, configure_pacman_conf, create_directory,
    take_ownership_of_directory, trust_additional_keys, update_system_packages,
};
use anyhow::Context;
use crate::docker::config::DockerConfig;
use std::process::Command;

const BUILD_USER: &str = "build";

const WORKING_DIR: &str = "/working-dir";

pub fn run_add_packages(config: &DockerConfig, packages: &[&str]) -> anyhow::Result<()> {
    setup_image_for_building_packages(config)?;

    let mut sync_command = create_base_aur_sync_command(config);

    for package in packages {
        sync_command.arg(package);
    }

    let command_status = sync_command.status().context("Failed to sync packages")?;

    println!("Finished syncing packages! with status: {}", command_status);

    Ok(())
}

pub fn run_update_packages(config: &DockerConfig) -> anyhow::Result<()> {
    setup_image_for_building_packages(config)?;

    let mut sync_command = create_base_aur_sync_command(config);

    sync_command.arg("-u");

    let command_status = sync_command.status().context("Failed to sync packages")?;

    println!("Finished syncing packages! with status: {}", command_status);

    Ok(())
}

pub fn run_rebuild_all_packages(config: &DockerConfig) -> anyhow::Result<()> {
    setup_image_for_building_packages(config)?;

    let mut sync_command = create_base_aur_sync_command(config);
    sync_command.arg("--rebuild-all");

    let command_status = sync_command
        .status()
        .context("Failed to rebuild all packages")?;

    println!(
        "Finished rebuilding all packages! with status: {}",
        command_status
    );

    Ok(())
}

pub fn run_rebuild_packages(config: &DockerConfig, packages: &[&str]) -> anyhow::Result<()> {
    setup_image_for_building_packages(config)?;

    let mut sync_command = create_base_aur_sync_command(config);
    sync_command.arg("--rebuild");

    for package in packages {
        sync_command.arg(package);
    }

    let command_status = sync_command
        .status()
        .context("Failed to rebuild packages")?;

    println!(
        "Finished rebuilding packages! with status: {}",
        command_status
    );

    Ok(())
}

fn create_base_aur_sync_command(config: &DockerConfig) -> Command {
    let mut sync_command = Command::new("sudo");
    sync_command
        .arg("-u")
        .arg(BUILD_USER)
        .arg("aur")
        .arg("sync")
        .arg("--noconfirm")
        .arg("--noview")
        .current_dir(WORKING_DIR);

    if config.signing.enabled {
        sync_command.arg("--sign");
    }

    sync_command
}

fn setup_image_for_building_packages(config: &DockerConfig) -> anyhow::Result<()> {
    update_system_packages()?;

    trust_additional_keys(
        &config
            .additional_trusted_keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
        BUILD_USER,
    )?;

    if config.signing.enabled {
        configure_package_signing(&config, BUILD_USER)?;
    }

    configure_pacman_conf(&config)?;

    update_system_packages()?;

    take_ownership_of_directory(config.repository.path.as_str(), BUILD_USER, BUILD_USER)?;

    create_directory(WORKING_DIR)?;
    take_ownership_of_directory(WORKING_DIR, BUILD_USER, BUILD_USER)?;

    Ok(())
}
