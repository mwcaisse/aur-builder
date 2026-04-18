use crate::docker::config::DockerConfig;
use crate::pgp_utils::get_key_id_from_private_key_file;
use std::fs;
use std::io::Write;
use std::process::Command;

/// Updates all the system packages (i.e., runs pacman -Syyu)
pub fn update_system_packages() {
    println!("Updating system packages");
    Command::new("pacman")
        .arg("-Syyu")
        .arg("--noconfirm")
        .status()
        .expect("Failed to update system packages");
}

pub fn trust_additional_keys(gpg_key_ids: &[&str], build_user: &str) {
    println!("Trusting additional keys");
    for key_id in gpg_key_ids {
        println!("Adding gpg key {}", key_id);
        Command::new("sudo")
            .arg("-u")
            .arg(build_user)
            .arg("gpg")
            .arg("--recv-key")
            .arg(key_id)
            .status()
            .expect("Failed to trust additional key");
    }
}

pub fn take_ownership_of_directory(directory: &str, user: &str, group: &str) {
    Command::new("chown")
        .arg("-R")
        .arg(format!("{}:{}", user, group))
        .arg(directory)
        .status()
        .expect(format!("Failed to take ownership of directory: {}", directory,).as_str());
}

pub fn create_directory(directory: &str) {
    fs::create_dir_all(directory)
        .expect(format!("Failed to create directory: {}", directory).as_str());
}

pub fn configure_package_signing(config: &DockerConfig, build_user: &str) {
    println!("Configuring package signing");
    // import the private key
    Command::new("sudo")
        .arg("-u")
        .arg(build_user)
        .arg("gpg")
        .arg("--import")
        .arg(config.signing.key_path.clone().unwrap().as_str())
        .status()
        .expect("failed to import signing key");

    // tell pacman about the signing key
    Command::new("pacman-key")
        .arg("--add")
        .arg(config.signing.public_key_path.clone().unwrap().as_str())
        .status()
        .expect("failed to add signing key to pacman");

    // set the key id in makepkg.conf
    let gpg_key_id =
        get_key_id_from_private_key_file(config.signing.key_path.clone().unwrap().as_str())
            .expect("failed to get key id from private key file");
    let make_file_text = format!("\nGPGKEY=\"{}\"\n", &gpg_key_id);

    write_text_to_end_of_file("/etc/makepkg.conf", &make_file_text);
}

pub fn configure_pacman_conf(config: &DockerConfig) {
    // TODO: Renable this. This was broken in the previous docker file as well
    //   Need to figure out how to avoid the unknown trust issues when this isn't set to "Optional TrustAll"
    // let sig_level = if config.signing.enabled {
    //     "Required DatabaseOptional"
    // } else {
    //     "Optional TrustAll"
    // };

    let pacman_conf_text = format!(
        "
[{}]
SigLevel = {}
Server = file://{}
",
        config.repository.name.as_str(),
        "Optional TrustAll",
        config.repository.path.as_str()
    );

    write_text_to_end_of_file("/etc/pacman.conf", &pacman_conf_text);
}

fn write_text_to_end_of_file(file_path: &str, text: &str) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect(format!("Failed to open file: {}", file_path).as_str());

    file.write_all(text.as_bytes())
        .expect(format!("Failed to write to file: {}", file_path).as_str());
}
