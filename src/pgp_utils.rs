use sequoia_openpgp::cert::CertParser;
use sequoia_openpgp::parse::Parse;
use std::fs::File;
use std::io::BufReader;

pub fn get_key_id_from_private_key_file(key_file_path: &str) -> Result<String, &str> {
    let file = File::open(key_file_path).expect("Unable to open private key file");
    let reader = BufReader::new(file);

    let mut certs = CertParser::from_reader(reader).expect("Unable to parse private key file");
    let all_certs = certs
        .collect::<Result<Vec<_>, _>>()
        .expect("Unable to collect certificates");

    if all_certs.len() != 1 {
        return Err("Expected exactly one certificate in private key file.");
    }

    return Ok(all_certs[0].keyid().to_string());
}
