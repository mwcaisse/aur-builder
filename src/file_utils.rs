use sequoia_openpgp::fmt::hex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn sha256_hash_file(file_path: &Path) -> String {
    println!("Hashing file: {}", file_path.display());
    let mut file = File::open(file_path).unwrap();
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = file.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let hash_result = hasher.finalize();

    hex::encode(hash_result)
}