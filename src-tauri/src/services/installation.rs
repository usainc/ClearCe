use crate::{
    core::processing::errors::{err, ErrorCode, Result},
    engines::realesrgan::discovery::{self, REQUIRED},
};
use std::{collections::BTreeMap, path::Path};
// Explicit user-selected, trusted local runtime only. No network or shell.
pub fn install(source: &Path, parent: &Path) -> Result<()> {
    install_inner(source, parent, false)
}
pub fn install_verified(source: &Path, parent: &Path) -> Result<()> {
    install_inner(source, parent, true)
}
fn install_inner(source: &Path, parent: &Path, verify: bool) -> Result<()> {
    let source = source
        .canonicalize()
        .map_err(|_| err(ErrorCode::EngineNotFound))?;
    std::fs::create_dir_all(parent).map_err(|_| err(ErrorCode::IoError))?;
    let parent = parent.canonicalize().map_err(|_| err(ErrorCode::IoError))?;
    let destination = parent.join("realesrgan");
    if destination.exists() && !verify {
        return Err(err(ErrorCode::EngineInvalid));
    }
    let staging = tempfile::Builder::new()
        .prefix("install-")
        .tempdir_in(&parent)
        .map_err(|_| err(ErrorCode::IoError))?;
    std::fs::create_dir(staging.path().join("models")).map_err(|_| err(ErrorCode::IoError))?;
    let mut files = BTreeMap::new();
    for name in REQUIRED
        .into_iter()
        .chain(["vcomp140.dll", "vcomp140d.dll"])
    {
        let path = source.join(name);
        if !path.exists() && !REQUIRED.contains(&name) {
            continue;
        }
        let path = path
            .canonicalize()
            .map_err(|_| err(ErrorCode::EngineInvalid))?;
        if !path.starts_with(&source)
            || !path.is_file()
            || path.metadata().map_err(|_| err(ErrorCode::IoError))?.len() > 128 * 1024 * 1024
        {
            return Err(err(ErrorCode::EngineInvalid));
        }
        let target = staging.path().join(name);
        std::fs::copy(path, &target).map_err(|_| err(ErrorCode::IoError))?;
        files.insert(name, discovery::hash(&target)?);
    }
    let manifest = serde_json::json!({"version":null,"files":files});
    std::fs::write(
        staging.path().join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .map_err(|_| err(ErrorCode::IoError))?;
    discovery::discover(&[(staging.path().into(), "local import".into())])?;
    if verify {
        use crate::core::processing::engine::{Availability, EnhancementEngine};
        let runtime = crate::engines::realesrgan::RealEsrgan {
            locations: vec![(staging.path().into(), "local import".into())],
            install_dir: staging.path().into(),
        };
        if !matches!(
            runtime.status().availability,
            Availability::Available | Availability::Unsupported
        ) {
            return Err(err(ErrorCode::EngineInvalid));
        }
    }
    // Keep an existing installation as a sibling backup; never recursively remove it.
    let backup = parent.join(format!("previous-{}", uuid::Uuid::new_v4()));
    let replaced = destination.exists();
    if replaced {
        std::fs::rename(&destination, &backup).map_err(|_| err(ErrorCode::IoError))?;
    }
    if std::fs::rename(staging.path(), &destination).is_err() {
        if replaced {
            let _ = std::fs::rename(&backup, &destination);
        }
        return Err(err(ErrorCode::IoError));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn managed_import_verifies_files_and_never_overwrites() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let dest = root.path().join("engines");
        std::fs::create_dir_all(source.join("models")).unwrap();
        std::fs::write(
            source.join(REQUIRED[0]),
            b"fixture executable (not launched)",
        )
        .unwrap();
        std::fs::write(source.join(REQUIRED[1]), b"7767517\nfixture").unwrap();
        assert!(install(&source, &dest).is_err());
        std::fs::write(source.join(REQUIRED[2]), b"fixture model").unwrap();
        install(&source, &dest).unwrap();
        assert!(discovery::discover(&[(dest.join("realesrgan"), "installed".into())]).is_ok());
        assert!(install(&source, &dest).is_err());
        std::fs::write(dest.join("realesrgan").join(REQUIRED[2]), b"corrupt").unwrap();
        assert!(discovery::discover(&[(dest.join("realesrgan"), "installed".into())]).is_err());
        assert_eq!(
            std::fs::read(source.join(REQUIRED[2])).unwrap(),
            b"fixture model"
        );
    }
}
