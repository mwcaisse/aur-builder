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
