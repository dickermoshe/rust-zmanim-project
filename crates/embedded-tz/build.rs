use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=zoneinfo");
    println!("cargo:rerun-if-changed=tzdb/version");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_BUNDLED_TZDB");

    if env::var_os("CARGO_FEATURE_BUNDLED_TZDB").is_none() {
        return;
    }

    if let Err(err) = generate_bundled_tzdb() {
        panic!("failed generating bundled tzdb: {}", err);
    }
}

fn generate_bundled_tzdb() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let zoneinfo_root = manifest_dir.join("zoneinfo");
    if !zoneinfo_root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing {} (run scripts/build-tzdb.sh first)",
                zoneinfo_root.display()
            ),
        ));
    }

    let mut paths = Vec::new();
    collect_regular_files(&zoneinfo_root, &mut paths)?;
    paths.sort();

    let out_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bundled_tzdb.rs");
    let mut out = fs::File::create(out_file)?;
    let tzdb_version = read_tzdb_version(&manifest_dir)?;

    writeln!(out, "pub const TZ_DB_VERSION: &str = {tzdb_version:?};")?;

    writeln!(out, "pub static BUNDLED_TZDB: &[(&str, &[u8])] = &[")?;

    for path in paths {
        let bytes = fs::read(&path)?;
        if !is_tzif(&bytes) {
            continue;
        }

        let rel = path
            .strip_prefix(&zoneinfo_root)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "strip_prefix failed"))?;
        let rel_norm = rel.to_string_lossy().replace('\\', "/");
        let include_path = path.to_string_lossy().replace('\\', "/");
        writeln!(out, "    ({rel_norm:?}, include_bytes!({include_path:?})),")?;
    }

    writeln!(out, "];")?;
    Ok(())
}

fn read_tzdb_version(manifest_dir: &Path) -> io::Result<String> {
    let raw = fs::read_to_string(manifest_dir.join("tzdb").join("version"))?;
    let version = raw.trim();
    let cleaned = version.strip_suffix("-dirty").unwrap_or(version);
    if cleaned.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "tzdb/version is empty",
        ));
    }
    Ok(cleaned.to_string())
}

fn collect_regular_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            collect_regular_files(&path, files)?;
        } else if ft.is_file() {
            if path.file_name() == Some(OsStr::new("posixrules")) {
                continue;
            }
            files.push(path);
        }
    }
    Ok(())
}

fn is_tzif(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && &bytes[..4] == b"TZif"
}
