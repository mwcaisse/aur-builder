use crate::docker::actions::{
    run_add_packages, run_rebuild_all_packages, run_rebuild_packages, run_update_packages,
};
use crate::docker::config::read_docker_config;
use clap::{Command, arg};
use std::path::PathBuf;

pub fn get_docker_commands() -> Command {
    Command::new("docker")
        .hide(true)
        .subcommand(
            Command::new("add")
                .about("Adds the given package(s) to the AUR repo and builds them")
                .arg(arg!([PACKAGE] ... "Package(s) to add to the repository").num_args(1..)),
        )
        .subcommand(Command::new("update").about("Updates the packages that have new versions"))
        .subcommand(
            Command::new("rebuild")
                .about("Rebuilds all of the packages in the repository")
                .arg(arg!([PACKAGE] ... "Package(s) to rebuild").num_args(1..)),
        )
}

pub const DEFAULT_DOCKER_CONFIG_PATH: &str = "/opt/aur-builder/config.toml";

pub fn handle_matching_commands(matches: &clap::ArgMatches) -> anyhow::Result<bool> {
    if let Some(docker_matches) = matches.subcommand_matches("docker") {
        let config_path = matches
            .get_one::<PathBuf>("config")
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or(DEFAULT_DOCKER_CONFIG_PATH.to_string());

        let config = read_docker_config(config_path)?;

        if let Some(docker_subcommand_matches) = docker_matches.subcommand_matches("add") {
            if let Some(names) = docker_subcommand_matches.get_many::<String>("PACKAGE") {
                let package_names = names.map(String::as_str).collect::<Vec<_>>();
                run_add_packages(&config, &package_names)?;
            } else {
                anyhow::bail!("Must specify at least one package to add!");
            }
        } else if let Some(_docker_subcommand_matches) =
            docker_matches.subcommand_matches("update")
        {
            run_update_packages(&config)?;
        } else if let Some(docker_subcommand_matches) =
            docker_matches.subcommand_matches("rebuild")
        {
            if let Some(names) = docker_subcommand_matches.get_many::<String>("PACKAGE") {
                let package_names = names.map(String::as_str).collect::<Vec<_>>();
                run_rebuild_packages(&config, &package_names)?;
            } else {
                run_rebuild_all_packages(&config)?;
            }
        } else {
            anyhow::bail!("Unknown docker subcommand!");
        }

        return Ok(true);
    }

    Ok(false)
}
