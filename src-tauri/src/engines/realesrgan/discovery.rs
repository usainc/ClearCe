use crate::core::processing::errors::{err, ErrorCode, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
pub const MODEL: &str = "realesrgan-x4plus";
pub const MODEL_ANIME: &str = "realesrgan-x4plus-anime";
pub const REQUIRED: [&str; 3] = [
    "realesrgan-ncnn-vulkan.exe",
    "models/realesrgan-x4plus.param",
    "models/realesrgan-x4plus.bin",
];
#[derive(Deserialize)]
pub struct Manifest {
    pub version: Option<String>,
    pub files: BTreeMap<String, String>,
}
pub struct Installation {
    pub root: PathBuf,
    pub source: String,
    pub version: Option<String>,
    pub models: Vec<String>,
}
pub fn discover(locations: &[(PathBuf, String)]) -> Result<Installation> {
    for (root, source) in locations {
        if !root.exists() {
            continue;
        }
        let root = root
            .canonicalize()
            .map_err(|_| err(ErrorCode::EngineInvalid))?;
        let manifest: Manifest = serde_json::from_reader(
            File::open(root.join("manifest.json")).map_err(|_| err(ErrorCode::EngineInvalid))?,
        )
        .map_err(|_| err(ErrorCode::EngineInvalid))?;
        for name in REQUIRED {
            let path = root
                .join(name)
                .canonicalize()
                .map_err(|_| err(ErrorCode::EngineInvalid))?;
            if !path.starts_with(&root) || !path.is_file() {
                return Err(err(ErrorCode::EngineInvalid));
            }
            let expected = manifest
                .files
                .get(name)
                .ok_or_else(|| err(ErrorCode::EngineInvalid))?;
            if &hash(&path)? != expected {
                return Err(err(ErrorCode::EngineInvalid));
            }
        }
        for (name, expected) in &manifest.files {
            if !matches!(
                name.as_str(),
                "vcomp140.dll"
                    | "vcomp140d.dll"
                    | "models/realesrgan-x4plus-anime.param"
                    | "models/realesrgan-x4plus-anime.bin"
                    | "package.json"
            ) {
                continue;
            }
            if hash(&root.join(name))? != *expected {
                return Err(err(ErrorCode::EngineInvalid));
            }
        }
        let param = std::fs::read_to_string(root.join(REQUIRED[1]))
            .map_err(|_| err(ErrorCode::EngineInvalid))?;
        if !param.starts_with("7767517") {
            return Err(err(ErrorCode::EngineInvalid));
        }
        let anime_param = root.join("models/realesrgan-x4plus-anime.param");
        let anime_bin = root.join("models/realesrgan-x4plus-anime.bin");
        let mut models = vec![MODEL.into()];
        if anime_param.is_file()
            && anime_bin.is_file()
            && std::fs::read_to_string(&anime_param).is_ok_and(|value| value.starts_with("7767517"))
        {
            models.push(MODEL_ANIME.into());
        }
        return Ok(Installation {
            root,
            source: source.clone(),
            version: manifest.version,
            models,
        });
    }
    Err(err(ErrorCode::EngineNotFound))
}
pub fn hash(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|_| err(ErrorCode::EngineInvalid))?;
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|_| err(ErrorCode::EngineInvalid))?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_engine_is_typed() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            discover(&[(dir.path().join("missing"), "test".into())])
                .err()
                .unwrap()
                .code,
            ErrorCode::EngineNotFound
        );
    }
    #[test]
    fn incomplete_installation_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            discover(&[(dir.path().into(), "test".into())])
                .err()
                .unwrap()
                .code,
            ErrorCode::EngineInvalid
        );
    }
}
