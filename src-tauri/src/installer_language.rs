//! One-shot installer handoff. No caller-controlled paths or arbitrary settings.
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::Path};
use tauri::Manager;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Tr,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bootstrap {
    version: u8,
    language: Language,
}

fn read(root: &Path) -> Option<Language> {
    let path = root.join("installer-language.json");
    let meta = fs::symlink_metadata(&path).ok()?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 128 {
        return None;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return None;
        }
    }
    let mut data = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(129)
        .read_to_end(&mut data)
        .ok()?;
    if data.len() > 128 {
        return None;
    }
    let value: Bootstrap = serde_json::from_slice(&data).ok()?;
    (value.version == 1).then_some(value.language)
}

#[tauri::command]
pub fn installer_language(app: tauri::AppHandle) -> Option<Language> {
    read(&app.path().app_data_dir().ok()?)
}

#[tauri::command]
pub fn acknowledge_installer_language(app: tauri::AppHandle) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|_| "App data unavailable")?
        .join("installer-language.json");
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Installer language cleanup failed".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_only_versioned_languages_and_preserves_other_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("installer-language.json");
        fs::write(dir.path().join("settings.json"), "unchanged").unwrap();
        for (data, expected) in [
            (r#"{"version":1,"language":"en"}"#, Some(Language::En)),
            (r#"{"version":1,"language":"tr"}"#, Some(Language::Tr)),
            (r#"{"version":2,"language":"tr"}"#, None),
            (r#"{"version":1,"language":"de"}"#, None),
            (r#"{"version":1,"language":"en","path":"x"}"#, None),
            ("not json", None),
        ] {
            fs::write(&path, data).unwrap();
            assert_eq!(read(dir.path()), expected);
        }
        fs::write(&path, vec![b' '; 129]).unwrap();
        assert_eq!(read(dir.path()), None);
        assert_eq!(
            fs::read_to_string(dir.path().join("settings.json")).unwrap(),
            "unchanged"
        );
    }
}
