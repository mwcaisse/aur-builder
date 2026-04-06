use std::collections::HashMap;

/// Captures metadata on a package
///     Currently we are only capturing name, version, and file name.
struct Package {
    name: String,
    version: String,
    file_name: String,
}

/// Parses package metadata information from the contents of a package's `desc` file.
///
fn parse_package_from_desc_contents(contents: &str) -> Result<Package, &str> {
    let fields = parse_fields_from_desc_file(contents).expect("Failed to parse package desc file");

    // TODO: fill this out, but this satisfies the compiler for now
    Ok(Package {
        name: fields
            .get("NAME")
            .unwrap_or(&Vec::new())
            .first()
            .unwrap()
            .clone(),
        version: fields
            .get("VERSION")
            .unwrap_or(&Vec::new())
            .first()
            .unwrap()
            .clone(),
        file_name: fields
            .get("FILENAME")
            .unwrap_or(&Vec::new())
            .first()
            .unwrap()
            .clone(),
    })
}

fn parse_fields_from_desc_file(contents: &str) -> Result<HashMap<String, Vec<String>>, &str> {
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
                "Unable to process package desc file. Found a field value without a field name.",
            );
        }

        // add the value to the field
        fields
            .entry(current_field.clone().unwrap())
            .or_default()
            .push(line.to_string());
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use crate::package_parser::*;

    const SIMPLE_DESC_CONTENTS: &str = "%FILENAME%
bitwarden-bin-2026.3.1-1-x86_64.pkg.tar.zst

%NAME%
bitwarden-bin

%VERSION%
2026.3.1-1

";

    #[test]
    fn test_parse_fields_from_desc_file_simple_fields() {
        let results = parse_fields_from_desc_file(SIMPLE_DESC_CONTENTS);

        assert!(results.is_ok());

        let fields = results.unwrap();

        assert_eq!(fields.len(), 3);

        assert!(fields.contains_key("NAME"));
        assert!(fields.contains_key("FILENAME"));
        assert!(fields.contains_key("VERSION"));

        assert_eq!(fields.get("NAME").unwrap().len(), 1);
        assert_eq!(fields.get("FILENAME").unwrap().len(), 1);
        assert_eq!(fields.get("VERSION").unwrap().len(), 1);

        assert_eq!(fields.get("NAME").unwrap()[0], "bitwarden-bin");
        assert_eq!(
            fields.get("FILENAME").unwrap()[0],
            "bitwarden-bin-2026.3.1-1-x86_64.pkg.tar.zst"
        );
        assert_eq!(fields.get("VERSION").unwrap()[0], "2026.3.1-1");
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
    }
}
