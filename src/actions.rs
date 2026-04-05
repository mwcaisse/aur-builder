use crate::config;
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
