use anyhow::Context;
use sequoia_openpgp::cert::CertParser;
use sequoia_openpgp::parse::Parse;
use std::fs::File;
use std::io::BufReader;

pub fn get_key_id_from_private_key_file(key_file_path: &str) -> anyhow::Result<String> {
    let file = File::open(key_file_path)
        .with_context(|| format!("Unable to open key file {}", key_file_path))?;

    let reader = BufReader::new(file);

    let all_certs = CertParser::from_reader(reader)
        .with_context(|| format!("Unable to parse key file {}", key_file_path))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Unable to collect certificates from key file {}", key_file_path))?;

    if all_certs.len() != 1 {
        anyhow::bail!("Expected exactly one certificate in private key file.");
    }

    Ok(all_certs[0].fingerprint().to_string())
}

#[cfg(test)]
mod tests {
    use crate::pgp_utils::get_key_id_from_private_key_file;
    use crate::test_utils::assert_string_starts_with;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn test_can_parse_key_id_from_file() {
        let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        key_path.push("resources/tests/FD65E82A5CA3DA76E8ECA4977F4989778F99886F.key");

        let key_id = get_key_id_from_private_key_file(key_path.to_str().unwrap()).unwrap();
        assert_eq!(key_id, "FD65E82A5CA3DA76E8ECA4977F4989778F99886F");
    }

    #[test]
    fn test_can_parse_key_id_from_public_key_file() {
        let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        key_path.push("resources/tests/FD65E82A5CA3DA76E8ECA4977F4989778F99886F.pub");

        let key_id = get_key_id_from_private_key_file(key_path.to_str().unwrap()).unwrap();
        assert_eq!(key_id, "FD65E82A5CA3DA76E8ECA4977F4989778F99886F");
    }

    #[test]
    fn test_non_existent_key_file_errors() {
        let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        key_path.push("resources/tests/file_does_not_exist.key");

        let res = get_key_id_from_private_key_file(key_path.to_str().unwrap());
        assert!(res.is_err());

        let error = res.unwrap_err();
        assert_string_starts_with(
            &format!("Unable to open key file {}", key_path.to_str().unwrap()),
            &format!("{}", error),
        );
    }

    #[test]
    fn test_non_key_file_errors() {
        let mut key_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        key_path.push("resources/tests/hello_world.txt");

        let res = get_key_id_from_private_key_file(key_path.to_str().unwrap());
        assert!(res.is_err());

        let error = res.unwrap_err();
        assert_string_starts_with(
            &format!("Unable to parse key file {}", key_path.to_str().unwrap()),
            &format!("{}", error),
        );
    }
}
