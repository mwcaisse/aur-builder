mod actions;
mod config;
mod docker;
mod package_parser;
mod pgp_utils;

mod file_utils;

#[cfg(test)]
mod test_utils;

use clap::{Command, arg, command, value_parser};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let default_config_path: String = "/etc/aur-builder/config.toml".to_string();

    let matches = command!()
        .about("A helper to create local AUR repos and manage the packages inside of them.")
        .arg(
            arg!(-c --config <FILE> "Path to configuration file. Defaults to '/etc/aur-builder/config.toml'")
                .required(false)
                .value_parser(value_parser!(PathBuf))
        )
        .subcommand(
            Command::new("create")
                .about("Create a new local AUR repo")
        )
        .subcommand(
            Command::new("add")
                .about("Adds the given package(s) to the AUR repo")
                .arg(
                    arg!([PACKAGE] ... "Package(s) to add to the repository").num_args(1..)
                )
        )
        .subcommand(
            Command::new("remove")
                .about("Removes the given package(s) from the AUR repo")
                .arg(
                    arg!([PACKAGE] ... "Package(s) to remove from the repository").num_args(1..)
                )
        )
        .subcommand(
            Command::new("remove-orphaned")
                .about("Removes any packages from the local repository that are no longer in the AUR")
        )
        .subcommand(
            Command::new("rebuild")
                .about("Rebuilds the given package(s) or all packages in the repository")
                .arg(
                    arg!([PACKAGE] ... "Package(s) to rebuild").num_args(1..)
                )
        )
        .subcommand(
            Command::new("update")
                .about("Updates the packages that have new versions")
        )
        .subcommand(
            Command::new("list")
                .about("Lists all of the packages in the repository")
        )
        .subcommand(
            Command::new("search")
                .about("Searches for packages in the repository")
                .arg(
                    arg!([TERM] "Term to search for, will return packages(s) which name contains the term")
                )
        )
        .subcommand(
            Command::new("validate")
                .about("Ensures all of the packages in the repository have a valid checksum. If not prompts to re-built them")
        )
        .subcommand(
            Command::new("clean")
                .about("Removes old versions of the package keeping at most n latest versions")
                .arg(
                    arg!(-n <NUM> "Number of versions of a package to keep.")
                        .value_parser(value_parser!(u32))
                        .default_value("2")
                        .long("number-to-keep")
                )
        )
        .subcommand(
            docker::commands::get_docker_commands()
        )
        .get_matches();

    if docker::commands::handle_matching_commands(&matches)? {
        return Ok(());
    }

    let config_path = matches
        .get_one::<PathBuf>("config")
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or(default_config_path);

    println!("Using config from: {}", config_path);

    let config: config::Config = config::read_config(config_path)?;

    println!("Loaded up config!");

    println!(
        "Using image: {}:{}",
        config.image.name.as_str(),
        config.image.tag.as_str()
    );
    println!(
        "Using repository {} at {}",
        config.repository.name.as_str(),
        config.repository.path.as_str()
    );
    println!("With image signing: {}", config.signing.enabled);

    if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(names) = matches.get_many::<String>("PACKAGE") {
            let package_names = names.map(String::as_str).collect::<Vec<_>>();
            actions::run_add_packages(&config, &package_names)?;
        } else {
            anyhow::bail!("Must specify at least one package to add!");
        }
    } else if let Some(matches) = matches.subcommand_matches("remove") {
        if let Some(names) = matches.get_many::<String>("PACKAGE") {
            let package_names = names.map(String::as_str).collect::<Vec<_>>();
            actions::run_remove_packages(&config, &package_names)?;
        } else {
            anyhow::bail!("Must specify at least one package to remove!");
        }
    } else if let Some(_matches) = matches.subcommand_matches("create") {
        actions::run_create_repo(&config)?;
    } else if let Some(_matches) = matches.subcommand_matches("update") {
        actions::run_update(&config)?;
    } else if let Some(rebuild_matches) = matches.subcommand_matches("rebuild") {
        if let Some(names) = rebuild_matches.get_many::<String>("PACKAGE") {
            let package_names = names.map(String::as_str).collect::<Vec<_>>();
            actions::run_rebuild(&config, &package_names)?;
        } else {
            actions::run_rebuild_all(&config)?;
        }
    } else if let Some(_matches) = matches.subcommand_matches("remove-orphaned") {
        actions::run_remove_orphans(&config)?;
    } else if let Some(clean_matches) = matches.subcommand_matches("clean") {
        let to_keep = clean_matches.get_one::<u32>("NUM").copied().unwrap_or(2);
        actions::run_clean(&config, to_keep)?;
    } else if let Some(_matches) = matches.subcommand_matches("list") {
        actions::list(&config)?;
    } else if let Some(search_matches) = matches.subcommand_matches("search") {
        if let Some(term) = search_matches.get_one::<String>("TERM") {
            actions::search(&config, term)?;
        } else {
            anyhow::bail!("Must specify a search term!");
        }
    } else if let Some(_matches) = matches.subcommand_matches("validate") {
        actions::validate(&config)?;
    } else {
        anyhow::bail!("Currently not implemented!");
    }

    Ok(())
}
