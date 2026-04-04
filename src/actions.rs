use crate::config;
use std::process::Command;

pub fn run_update(config: config::Config) {
    println!("Performing update on all packages!");

    let docker_image = format!("{}:{}", config.image.name, config.image.tag);

    let pull_command = Command::new("docker")
        .arg("pull")
        .arg(&docker_image)
        .status()
        .expect("Failed to update image :(");

    println!("Pulled image! with status: {}", pull_command);

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
    add_env_arg(&mut update_command, "AUR_BUILDER_NEW_PACKAGES", "");
    add_env_arg(&mut update_command, "AUR_BUILDER_GPG_KEYS", &trusted_keys);
    add_env_arg(
        &mut update_command,
        "AUR_BUILDER_SIGN_PACKAGES",
        &config.signing.enabled.to_string(),
    );

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

    let command_status = update_command
        .arg(&docker_image)
        .status()
        .expect("Failed to update packages :(");

    println!(
        "Finished updating all packages! with status: {}",
        command_status
    );
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
