use std::process::exit;
use clap::{Command, command, arg};

fn main() {
    let matches = command!()
        .about("A helper to create local AUR repos and manage the packages inside of them.")
        .arg(
            arg!(-c --config <FILE> "Path to configuration file. Defaults to '/etc/aur-builder/config.yaml'").default_value("/etc/aur-builder/config.yaml")
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
        )
        .subcommand(
            Command::new("remove-orphaned")
                .about("Removes any packages from the local repository that are no longer in the AUR")
        )
        .subcommand(
            Command::new("rebuild")
                .about("Rebuilds all of the packages in the repository")
        )
        .subcommand(
            Command::new("update")
                .about("Updates the packages that have new versions")
        )
        .get_matches();


    if let Some(matches) = matches.subcommand_matches("add") {
        if let Some(names) = matches.get_many::<String>("PACKAGE") {
            let string_names = names.into_iter().map(String::as_str).collect::<Vec<_>>().join(", ");
            println!("Adding the following packages: [{}]", string_names);
        }
        else {
            println!("Must specify at least one package to add!");
            exit(1);
        }
    }
    else if let Some(_matches) = matches.subcommand_matches("update") {
        println!("Performing update on all packages!");
    }
    else {
        println!("Currently not implemented!");
        exit(1);
    }

}
