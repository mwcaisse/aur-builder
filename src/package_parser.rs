use anyhow::Context;
use flate2::read::GzDecoder;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use tar::Archive;
use xz::read::XzDecoder;

/// Captures metadata on a package
///     Currently we are only capturing name, version, and file name.
pub struct Package {
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub sha_256_checksum: String,
    pub description: Option<String>,
}

/// Gets a list of packages from the Arch database in the given file
/// Assumes that the database in in tar.xz format
/// TODO: Add support for other archive formats / determining format of the database file
///     Arch supports any archive format that is supported by `libarchive`
pub fn get_packages_from_arch_database(path_to_database: &str) -> anyhow::Result<Vec<Package>> {
    let file = File::open(path_to_database)
        .with_context(|| format!("Failed to open database file: {}", path_to_database))?;
    let tar_archive = XzDecoder::new(file);
    let mut archive = Archive::new(tar_archive);

    let mut packages: Vec<Package> = Vec::new();

    let entries = archive
        .entries()
        .context("Failed to read archive entries")?;

    for file in entries {
        let mut file = file.context("Failed to read archive entry")?;

        let path = file
            .header()
            .path()
            .context("Failed to get entry path")?;
        let file_name = path
            .file_name()
            .context("Failed to get file name from entry path")?;

        if file.header().entry_type().is_file() && file_name == "desc" {
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents)
                .context("Failed to read desc file contents")?;
            let package = parse_package_from_desc_contents(&file_contents)?;
            packages.push(package);
        }
    }

    Ok(packages)
}

pub fn get_all_aur_packages() -> anyhow::Result<HashSet<String>> {
    let resp = reqwest::blocking::get("https://aur.archlinux.org/packages.gz")
        .context("Failed to fetch AUR package list")?;

    let mut gz = GzDecoder::new(resp);
    let mut contents = String::new();
    gz.read_to_string(&mut contents)
        .context("Failed to read AUR package list")?;

    let aur_packages: HashSet<String> = contents.lines().map(|l| l.to_string()).collect();

    Ok(aur_packages)
}

/// Parses package metadata information from the contents of a package's `desc` file.
fn parse_package_from_desc_contents(contents: &str) -> anyhow::Result<Package> {
    let fields = parse_fields_from_desc_file(contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse package desc file: {}", e))?;

    Ok(Package {
        name: get_field_value(&fields, "NAME")?,
        version: get_field_value(&fields, "VERSION")?,
        file_name: get_field_value(&fields, "FILENAME")?,
        sha_256_checksum: get_field_value(&fields, "SHA256SUM")?,
        description: fields.get("DESC").and_then(|v: &Vec<String>| v.first().cloned()),
    })
}

fn get_field_value(fields: &HashMap<String, Vec<String>>, field_name: &str) -> anyhow::Result<String> {
    fields
        .get(field_name)
        .and_then(|v| v.first())
        .cloned()
        .with_context(|| format!("Missing required field '{}' in package desc file", field_name))
}

fn parse_fields_from_desc_file(contents: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let mut fields: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_field: Option<String> = None;

    for line in contents.lines() {
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() {
            continue;
        }

        if trimmed_line.starts_with("%") && trimmed_line.ends_with("%") {
            current_field = Some(trimmed_line[1..trimmed_line.len() - 1].to_uppercase());
            continue;
        }

        // we have a value line, but no field, return an error
        if current_field.is_none() {
            return Err(
                "Unable to process package desc file. Found a field value without a field name.".to_string(),
            );
        }

        // add the value to the field
        let field_name = current_field.as_ref().unwrap();
        fields
            .entry(field_name.clone())
            .or_default()
            .push(line.to_string());
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use crate::package_parser::*;
    use pretty_assertions::assert_eq;

    const SIMPLE_DESC_CONTENTS: &str = "%FILENAME%
bitwarden-bin-2026.3.1-1-x86_64.pkg.tar.zst

%NAME%
bitwarden-bin

%VERSION%
2026.3.1-1

%SHA256SUM%
8c2085e1d306d423d34cb10ec968f52555874a4af5e5eabd2b3ab1bf9e0d4cfd

%DESC%
Yet another yogurt. Pacman wrapper and AUR helper written in go. Pre-compiled.


";

    #[test]
    fn test_parse_fields_from_desc_file_simple_fields() {
        let results = parse_fields_from_desc_file(SIMPLE_DESC_CONTENTS);

        assert!(results.is_ok());

        let fields = results.unwrap();

        assert_eq!(fields.len(), 5);

        assert!(fields.contains_key("NAME"));
        assert!(fields.contains_key("FILENAME"));
        assert!(fields.contains_key("VERSION"));
        assert!(fields.contains_key("SHA256SUM"));
        assert!(fields.contains_key("DESC"));

        assert_eq!(fields.get("NAME").unwrap().len(), 1);
        assert_eq!(fields.get("FILENAME").unwrap().len(), 1);
        assert_eq!(fields.get("VERSION").unwrap().len(), 1);
        assert_eq!(fields.get("SHA256SUM").unwrap().len(), 1);
        assert_eq!(fields.get("DESC").unwrap().len(), 1);

        assert_eq!(fields.get("NAME").unwrap()[0], "bitwarden-bin");
        assert_eq!(
            fields.get("FILENAME").unwrap()[0],
            "bitwarden-bin-2026.3.1-1-x86_64.pkg.tar.zst"
        );
        assert_eq!(fields.get("VERSION").unwrap()[0], "2026.3.1-1");
        assert_eq!(
            fields.get("SHA256SUM").unwrap()[0],
            "8c2085e1d306d423d34cb10ec968f52555874a4af5e5eabd2b3ab1bf9e0d4cfd"
        );
        assert_eq!(fields.get("DESC").unwrap()[0], "Yet another yogurt. Pacman wrapper and AUR helper written in go. Pre-compiled.");
    }

    const MULTILINE_DESC_CONTENTS: &str = "%DEPENDS%
aspnet-runtime-6.0
gcc-libs
glibc
sqlite

%MAKEDEPENDS%
dotnet-sdk-6.0
yarn";

    #[test]
    fn test_parse_fields_from_desc_files_multiline_values() {
        let results = parse_fields_from_desc_file(MULTILINE_DESC_CONTENTS);
        assert!(results.is_ok());

        let fields = results.unwrap();
        assert_eq!(fields.len(), 2);

        assert!(fields.contains_key("DEPENDS"));
        assert!(fields.contains_key("MAKEDEPENDS"));

        assert_eq!(fields.get("DEPENDS").unwrap().len(), 4);
        assert_eq!(fields.get("MAKEDEPENDS").unwrap().len(), 2);

        assert_eq!(
            fields.get("DEPENDS").unwrap(),
            &vec!["aspnet-runtime-6.0", "gcc-libs", "glibc", "sqlite"]
        );
        assert_eq!(
            fields.get("MAKEDEPENDS").unwrap(),
            &vec!["dotnet-sdk-6.0", "yarn"]
        );
    }

    const ERROR_VALUE_WITHOUT_FIELD_NAME_CONTENT: &str = "HELLO
%MAKEDEPENDS%";

    #[test]
    fn test_parse_fields_from_desc_files_no_values() {
        let results = parse_fields_from_desc_file(ERROR_VALUE_WITHOUT_FIELD_NAME_CONTENT);
        assert!(!results.is_ok());
    }

    #[test]
    fn test_parse_package_from_desc_contents() {
        let results = parse_package_from_desc_contents(SIMPLE_DESC_CONTENTS);
        assert!(results.is_ok());

        let package = results.unwrap();
        assert_eq!(package.name, "bitwarden-bin");
        assert_eq!(package.version, "2026.3.1-1");
        assert_eq!(
            package.file_name,
            "bitwarden-bin-2026.3.1-1-x86_64.pkg.tar.zst"
        );
        assert_eq!(
            package.sha_256_checksum,
            "8c2085e1d306d423d34cb10ec968f52555874a4af5e5eabd2b3ab1bf9e0d4cfd"
        );
    }
}
