use anyhow::Context;
use sequoia_openpgp::fmt::hex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn sha256_hash_file(file_path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let hash_result = hasher.finalize();

    Ok(hex::encode(hash_result))
}

#[cfg(test)]
mod tests {
    use super::sha256_hash_file;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn resources_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/tests");
        path
    }

    #[test]
    fn test_hash_small_file_matches_expected() {
        let path = resources_dir().join("hello_world.txt");
        let hash = sha256_hash_file(&path).unwrap();
        assert_eq!(hash, "7F83B1657FF1FC53B92DC18148A1D65DFC2D4B1FA3D677284ADDD200126D9069");
    }

    #[test]
    fn test_hash_binary_file_matches_expected() {
        let path = resources_dir().join("test-aur.db.tar.xz");
        let hash = sha256_hash_file(&path).unwrap();
        assert_eq!(
            hash,
            "91284907FD22528050E2966C43706F650E2B16E435FF6E20D149A7A994BF7D20"
        );
    }

    #[test]
    fn test_hash_empty_file() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let hash = sha256_hash_file(file.path()).unwrap();
        assert_eq!(
            hash,
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        );
    }

    #[test]
    fn test_hash_large_file_spans_multiple_buffers() {
        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        {
            let mut file = File::create(tmp_file.path()).unwrap();
            file.write_all(&vec![b'a'; 20_000]).unwrap();
        }
        let hash = sha256_hash_file(tmp_file.path()).unwrap();
        assert_eq!(
            hash,
            "CC17FAAAD36649C4603DDA4D8FF97CB149722AF0BCAC0746305A2134AD2D0B97"
        );
    }

    #[test]
    fn test_hash_nonexistent_file_errors() {
        let path = resources_dir().join("does-not-exist.txt");
        assert!(sha256_hash_file(&path).is_err());
    }
}
