use crate::{config, package_parser};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

pub fn run_clean(config: config::Config, to_keep: u32) {
    println!(
        "Cleaning up old versions of packages! Keeping at most {} versions",
        to_keep
    );

    let clean_status = Command::new("paccache")
        .arg("-rv")
        .arg("-c")
        .arg(&config.repository.path)
        .arg("-k")
        .arg(to_keep.to_string())
        .status()
        .expect("Failed to clean up old versions of packages :(");

    println!(
        "Finished cleaning up old versions of packages! with status: {}",
        clean_status
    );
}

pub fn run_create_repo(config: &config::Config) {
    println!("Creating repository at path: {}", config.repository.path);

    // TODO: Should make sure that the parent directories in `repo_path` exist before calling `repo-add`
    let ref repo_path = create_repository_file_path(config);

    let mut command = Command::new("repo-add");

    // if signing is enabled, pass sign flag + key flag
    if config.signing.enabled {
        command.arg("-s");
        command.arg("-k");
        command.arg(config.signing.key_id.clone().unwrap());
    }

    command.arg(repo_path);

    let status = command.status().expect("Failed to create repository :(");

    println!(
        "Finished creating repository {}! with status: {}",
        repo_path, status
    );
}

// TODO: Should probably borrow config in the other functions in here as well
pub fn run_remove_packages(config: &config::Config, packages: &[&str]) {
    println!("Removing the following packages: {:?}", packages);

    let status = remove_packages_internal(config, packages);

    println!("Finished removing packages! with status: {}", status);
}

fn remove_packages_internal(config: &config::Config, packages: &[&str]) -> ExitStatus {
    let mut command = Command::new("repo-remove");

    command.arg("--remove");

    // if signing is enabled, pass sign flag + key flag
    if config.signing.enabled {
        command.arg("-s");
        command.arg("-k");
        command.arg(config.signing.key_id.clone().unwrap());
    }

    let repo_path = create_repository_file_path(config);
    command.arg(repo_path);
    command.args(packages);

    let status = command.status().expect("Failed to remove packages :(");
    status
}
pub fn run_remove_orphans(config: &config::Config) {
    /*

       - We'll need to get a list of all current packages in the AUR
       - We'll need to get a list of all packages in the local repository

        To get all current AUR packages:
            https://wiki.archlinux.org/title/Aurweb_RPC_interface
            This file (https://aur.archlinux.org/packages.gz) contains a list of all packages in the AUR, seperated by line break

        To get all packages in the local repository:
            The local repository file, is an archive file. Inside there is folder for each package.
            The folder for the package has a `desc` file inside, which contains metadata about the package.
            Including the package name.

    */

    let orphaned_packages = get_orphaned_packages(config);

    // Confirm that the user wants to continue and remove the orphaned packages

    println!(
        "The following packages are orphaned and will be removed: {:?}",
        orphaned_packages
    );
    print!("Proceed with removing them? [Y/n] ");
    io::stdout().flush().ok();
    let mut input = String::new();
    let read_result = io::stdin().read_line(&mut input);

    if !read_result.is_ok() || input.trim().to_lowercase() != "y" {
        return;
    }

    //user agreed, remove the orphans
    let status = remove_packages_internal(
        config,
        orphaned_packages
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice(),
    );

    println!(
        "Finished removing orphaned packages! with status: {}",
        status
    );
}

fn get_orphaned_packages(config: &config::Config) -> Vec<String> {
    let repo_path = create_repository_file_path(config);
    let our_packages = package_parser::get_packages_from_arch_database(&repo_path);
    let aur_packages = package_parser::get_all_aur_packages();

    let mut orphaned_packages: Vec<String> = Vec::new();
    for package in our_packages {
        let package_name = package.name.as_str();
        // Ignore package names that end in `-debug` as they are sometimes created as part of the build of the normal package
        // TODO: need to figure out how to differentiate these from actual packages with -debug at the end
        if !aur_packages.contains(package_name) && !package_name.ends_with("-debug") {
            orphaned_packages.push(package.name);
        }
    }

    orphaned_packages
}

fn create_repository_file_path(config: &config::Config) -> String {
    let mut path = PathBuf::from(config.repository.path.clone());
    // TODO: Probably need to handle different database archive extensions (not just assume .db.tar.xz)
    path.push(format!("{}.db.tar.xz", config.repository.name));

    return path.to_string_lossy().to_string();
}

pub fn run_add_packages(config: config::Config, packages: &[&str]) {
    println!("Adding the following packages: {:?}", packages);

    let command_status = run_docker_image(config, Option::from(packages));

    println!("Finished adding packages! with status: {}", command_status);
}

pub fn run_update(config: config::Config) {
    println!("Performing update on all packages!");

    let command_status = run_docker_image(config, None);

    println!(
        "Finished updating all packages! with status: {}",
        command_status
    );
}

pub fn run_rebuild_all(config: config::Config) {
    println!("Performing rebuild on all packages!");

    // Right now to re-build all packages, we pass a package with the name "rebuild" to the docker image
    let command_status = run_docker_image(config, Option::from(&["rebuild"][..]));

    println!(
        "Finished rebuilding all packages! with status: {}",
        command_status
    );
}

fn run_docker_image(config: config::Config, packages: Option<&[&str]>) -> ExitStatus {
    let docker_image = format!("{}:{}", config.image.name, config.image.tag);

    // if we are configured to always pull, pull the image before we run it
    if config.image.always_pull {
        let pull_command = Command::new("docker")
            .arg("pull")
            .arg(&docker_image)
            .status()
            .expect("Failed to update image :(");

        println!("Pulled image! with status: {}", pull_command);
    }

    // Now we shall run the docker image itself

    let trusted_keys = config.additional_trusted_keys.join(" ");
    let mut update_command = Command::new("docker");

    update_command.arg("run");
    add_mount_arg(&mut update_command, &config.repository.path, "/repo");
    add_env_arg(
        &mut update_command,
        "AUR_BUILDER_REPO_NAME",
        &config.repository.name,
    );

    let space_separated_packages = packages.map_or(String::new(), |packages| packages.join(" "));
    add_env_arg(
        &mut update_command,
        "AUR_BUILDER_NEW_PACKAGES",
        &space_separated_packages,
    );
    add_env_arg(&mut update_command, "AUR_BUILDER_GPG_KEYS", &trusted_keys);
    add_env_arg(
        &mut update_command,
        "AUR_BUILDER_SIGN_PACKAGES",
        &config.signing.enabled.to_string(),
    );

    println!("Signing enabled!: {}", &config.signing.enabled.to_string());

    if config.signing.enabled {
        // add the public and private key mounts
        //TODO: Should add some checking that the signing values are set
        add_mount_arg(
            &mut update_command,
            &config.signing.key_path.unwrap(),
            "/aur-builder-keys/signing.key",
        );
        add_mount_arg(
            &mut update_command,
            &config.signing.public_key_path.unwrap(),
            "/aur-builder-keys/signing.pub",
        );
        // add the signing key config
        add_env_arg(
            &mut update_command,
            "AUR_BUILDER_GPG_KEY_ID",
            &config.signing.key_id.unwrap(),
        );
    }

    update_command
        .arg(&docker_image)
        .status()
        .expect("Failed to update packages :(")
}

fn add_env_arg(command: &mut Command, name: &str, value: &str) {
    command.args(["--env", format!("{}={}", name, value).as_str()]);
}

fn add_mount_arg(command: &mut Command, source: &str, destination: &str) {
    command.args([
        "--mount",
        format!("type=bind,source={},destination={}", source, destination).as_str(),
    ]);
}
