use crate::core::processing::errors::{err, ErrorCode, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

fn linked(meta: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
fn scan(path: &Path, root: &Path, entries: &mut usize) -> Result<u64> {
    *entries += 1;
    if *entries > 10000 {
        return Err(err(ErrorCode::IoError));
    }
    let meta = std::fs::symlink_metadata(path).map_err(|_| err(ErrorCode::IoError))?;
    if linked(&meta)
        || !path
            .canonicalize()
            .map_err(|_| err(ErrorCode::IoError))?
            .starts_with(root)
    {
        log::warn!("event=cleanup_skipped_unsafe_path");
        return Err(err(ErrorCode::IoError));
    }
    if meta.is_file() {
        return Ok(meta.len());
    }
    let mut bytes = 0;
    for entry in std::fs::read_dir(path).map_err(|_| err(ErrorCode::IoError))? {
        bytes += scan(
            &entry.map_err(|_| err(ErrorCode::IoError))?.path(),
            root,
            entries,
        )?;
    }
    Ok(bytes)
}
// Call only while the application's exclusive session lock is held and no job is active.
// Never follow reparse points. Validate the entire subtree before removing anything.
pub fn validate_target(root: &Path, target: &Path) -> Result<()> {
    use std::path::Component;
    if target
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(err(ErrorCode::IoError));
    }
    if linked(&std::fs::symlink_metadata(root).map_err(|_| err(ErrorCode::IoError))?)
        || linked(&std::fs::symlink_metadata(target).map_err(|_| err(ErrorCode::IoError))?)
    {
        return Err(err(ErrorCode::IoError));
    }
    let root = root.canonicalize().map_err(|_| err(ErrorCode::IoError))?;
    let target = target.canonicalize().map_err(|_| err(ErrorCode::IoError))?;
    if target == root
        || target.parent() != Some(root.as_path())
        || !target
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("job-"))
    {
        return Err(err(ErrorCode::IoError));
    }
    Ok(())
}
pub fn cache(root: &Path, clear: bool) -> Result<u64> {
    if !root.exists() {
        return Ok(0);
    }
    if linked(&std::fs::symlink_metadata(root).map_err(|_| err(ErrorCode::IoError))?) {
        return Err(err(ErrorCode::IoError));
    }
    let canonical = root.canonicalize().map_err(|_| err(ErrorCode::IoError))?;
    let mut bytes = 0;
    let mut entries = 0;
    for entry in std::fs::read_dir(&canonical).map_err(|_| err(ErrorCode::IoError))? {
        let entry = entry.map_err(|_| err(ErrorCode::IoError))?;
        if !entry.file_name().to_string_lossy().starts_with("job-") {
            continue;
        }
        let path = entry.path();
        validate_target(&canonical, &path)?;
        let size = scan(&path, &canonical, &mut entries)?;
        bytes += size;
        if clear {
            log::info!("event=stale_job_discovered");
            if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            }
            .map_err(|_| err(ErrorCode::IoError))?;
            log::info!("event=stale_temp_removed");
        }
    }
    Ok(bytes)
}
pub fn output(path: Option<&str>, default: &Path) -> Result<PathBuf> {
    let path = path
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| default.into());
    if !path.is_absolute() {
        return Err(err(ErrorCode::OutputNotWritable));
    }
    if path == default {
        std::fs::create_dir_all(&path).map_err(|_| err(ErrorCode::OutputNotWritable))?;
    }
    let path = path
        .canonicalize()
        .map_err(|_| err(ErrorCode::OutputNotWritable))?;
    let _probe = tempfile::Builder::new()
        .prefix(".enhancece-check-")
        .tempfile_in(&path)
        .map_err(|_| err(ErrorCode::OutputNotWritable))?;
    Ok(path)
}
#[derive(Serialize)]
pub struct Check {
    pub code: String,
    pub name: String,
    pub ok: bool,
    pub message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub engine_state: crate::core::processing::engine::Availability,
    pub state: String,
    pub checks: Vec<Check>,
    pub files: Vec<(String, u64)>,
    pub cache_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_root_outside_unowned_and_parent_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("cache");
        std::fs::create_dir_all(root.join("job-safe")).unwrap();
        std::fs::create_dir(temp.path().join("job-outside")).unwrap();
        std::fs::write(root.join("source.png"), b"keep").unwrap();
        assert!(validate_target(&root, &root.join("job-safe")).is_ok());
        for path in [
            root.clone(),
            temp.path().join("job-outside"),
            root.join("../job-outside"),
            root.join("source.png"),
        ] {
            assert!(validate_target(&root, &path).is_err());
        }
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "Opt-in real Windows symlink test; requires OS symlink permission"]
    fn real_symlink_escape_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("cache");
        let outside = temp.path().join("outside");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("sentinel"), b"preserve").unwrap();
        let link = root.join("job-link");
        std::os::windows::fs::symlink_dir(&outside, &link)
            .expect("OS permission is required; if denied document as a manual release test");
        assert!(cache(&root, true).is_err());
        assert_eq!(
            std::fs::read(outside.join("sentinel")).unwrap(),
            b"preserve"
        );
        std::fs::remove_dir(link).unwrap();
        let job = root.join("job-parent");
        std::fs::create_dir(&job).unwrap();
        let nested = job.join("nested-link");
        std::os::windows::fs::symlink_dir(&outside, &nested).unwrap();
        assert!(cache(&root, true).is_err());
        assert_eq!(
            std::fs::read(outside.join("sentinel")).unwrap(),
            b"preserve"
        );
        std::fs::remove_dir(nested).unwrap();
    }
    #[test]
    fn cleanup_only_removes_managed_jobs() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().join("temp");
        std::fs::create_dir_all(root.join("job-old")).unwrap();
        std::fs::write(root.join("job-old/intermediate.png"), b"1234").unwrap();
        for file in [
            "source.png",
            "output.png",
            "history.sqlite3",
            "settings.json",
        ] {
            std::fs::write(d.path().join(file), b"keep").unwrap();
        }
        std::fs::write(root.join("unowned"), b"keep").unwrap();
        assert_eq!(cache(&root, false).unwrap(), 4);
        assert_eq!(cache(&root, true).unwrap(), 4);
        assert_eq!(cache(&root, false).unwrap(), 0);
        assert!(root.join("unowned").exists());
        for file in [
            "source.png",
            "output.png",
            "history.sqlite3",
            "settings.json",
        ] {
            assert!(d.path().join(file).exists());
        }
    }
    #[test]
    fn output_validation_handles_unicode_and_missing_paths() {
        let d = tempfile::tempdir().unwrap();
        let folder = d.path().join("çıktı images");
        std::fs::create_dir(&folder).unwrap();
        assert!(output(folder.to_str(), d.path()).is_ok());
        assert!(output(Some("relative"), d.path()).is_err());
        assert!(output(d.path().join("absent").to_str(), d.path()).is_err());
    }
}
