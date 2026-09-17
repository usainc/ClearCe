use crate::{
    core::processing::{
        engine::EngineStatus,
        errors::{err, ErrorCode, Result},
    },
    engines::realesrgan::discovery,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub const PACKAGE_ID: &str = "realesrgan-ncnn-vulkan-20220424-windows";
pub const PACKAGE_VERSION: &str = "0.2.5.0";
pub const PACKAGE_URL: &str = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesrgan-ncnn-vulkan-20220424-windows.zip";
pub const PACKAGE_SHA256: &str = "abc02804e17982a3be33675e4d471e91ea374e65b70167abc09e31acb412802d";
const MAX_DOWNLOAD: u64 = 96 * 1024 * 1024;
const MIN_FREE_SPACE: u64 = 192 * 1024 * 1024;
const PACKAGE_FILES: [&str; 7] = [
    "realesrgan-ncnn-vulkan.exe",
    "models/realesrgan-x4plus.param",
    "models/realesrgan-x4plus.bin",
    "models/realesrgan-x4plus-anime.param",
    "models/realesrgan-x4plus-anime.bin",
    "vcomp140.dll",
    "vcomp140d.dll",
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HardwareTier {
    Unsupported,
    Low,
    Mid,
    High,
    Ultra,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    pub description_key: &'static str,
    pub supported_image_types: &'static [&'static str],
    pub recommended_tier: HardwareTier,
    pub required_files: &'static [&'static str],
    pub source_url: Option<&'static str>,
    pub expected_sha256: Option<&'static str>,
    pub package_type: &'static str,
    pub install_status: &'static str,
    pub engine_compatibility: &'static str,
    pub downloadable: bool,
    pub manual_only: bool,
    pub default_eligible: bool,
    pub availability: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuSummary {
    pub names: Vec<String>,
    pub vendors: Vec<String>,
    pub device_count: usize,
    pub vulkan_available: bool,
    pub dedicated_memory_bytes: Option<u64>,
    pub tier: HardwareTier,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub model_id: Option<String>,
    pub display_name: Option<String>,
    pub profile: &'static str,
    pub reason_key: &'static str,
    pub installed: bool,
    pub action: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedState {
    pub catalog: Vec<CatalogItem>,
    pub gpu: GpuSummary,
    pub recommendation: Recommendation,
    pub active_engine_type: String,
    pub active_model: String,
    pub active_path: Option<String>,
    pub managed_installed: bool,
    pub manual_installed: bool,
    pub package_version: &'static str,
    pub package_source: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub stage: &'static str,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

pub fn managed_root(data: &Path) -> PathBuf {
    data.join("engines").join("managed").join("realesrgan")
}
pub fn manual_root(data: &Path) -> PathBuf {
    data.join("engines").join("manual").join("realesrgan")
}

fn installed_models(root: &Path) -> Vec<String> {
    discovery::discover(&[(root.into(), "managed".into())])
        .map(|i| i.models)
        .unwrap_or_default()
}

pub fn hardware_tier(devices: &[crate::core::processing::engine::GpuDevice]) -> HardwareTier {
    if devices.is_empty() {
        return HardwareTier::Unsupported;
    }
    let device = devices
        .iter()
        .max_by_key(|d| d.dedicated_memory_bytes.unwrap_or(0))
        .unwrap();
    if device.device_type == "integrated" {
        return HardwareTier::Low;
    }
    match device
        .dedicated_memory_bytes
        .map(|v| v / (1024 * 1024 * 1024))
    {
        Some(0..=3) => HardwareTier::Low,
        Some(4..=7) => HardwareTier::Mid,
        Some(8..=15) => HardwareTier::High,
        Some(16..) => HardwareTier::Ultra,
        None => HardwareTier::Mid,
    }
}

pub fn state(
    data: &Path,
    status: &EngineStatus,
    category: &str,
    engine_mode: &str,
    selected_model: &str,
) -> ManagedState {
    let managed = installed_models(&managed_root(data));
    let manual = installed_models(&manual_root(data));
    let tier = hardware_tier(&status.devices);
    let managed_installed = !managed.is_empty();
    let manual_installed = !manual.is_empty()
        || data
            .join("engines")
            .join("realesrgan")
            .join("manifest.json")
            .is_file();
    let wanted = if category == "Anime / Illustration" {
        discovery::MODEL_ANIME
    } else {
        discovery::MODEL
    };
    let installed = if engine_mode == "Manual" {
        manual.iter().any(|m| m == wanted) || (wanted == discovery::MODEL && manual_installed)
    } else {
        managed.iter().any(|m| m == wanted)
            || manual.iter().any(|m| m == wanted)
            || (wanted == discovery::MODEL && manual_installed)
    };
    let reason_key = match tier {
        HardwareTier::Unsupported => "recommendation_reason_unsupported",
        HardwareTier::Low => "recommendation_reason_low",
        HardwareTier::Mid => "recommendation_reason_mid",
        HardwareTier::High | HardwareTier::Ultra => "recommendation_reason_high",
    };
    let recommendation = if tier == HardwareTier::Unsupported {
        Recommendation {
            model_id: None,
            display_name: None,
            profile: "Unavailable",
            reason_key,
            installed: false,
            action: "driver",
        }
    } else {
        Recommendation {
            model_id: Some(wanted.into()),
            display_name: Some(
                if wanted == discovery::MODEL_ANIME {
                    "RealESRGAN Anime 6B"
                } else {
                    "RealESRGAN x4plus"
                }
                .into(),
            ),
            profile: match tier {
                HardwareTier::Low => "Efficient",
                HardwareTier::Mid => "Balanced",
                _ => "Quality",
            },
            reason_key,
            installed,
            action: if installed { "verify" } else { "download" },
        }
    };
    let install_status = |id: &str| {
        if managed.iter().any(|m| m == id) {
            "installed"
        } else {
            "not_installed"
        }
    };
    let catalog = vec![
        CatalogItem {
            id: discovery::MODEL,
            display_name: "RealESRGAN x4plus",
            category: "general",
            description_key: "model_x4plus_description",
            supported_image_types: &["photo"],
            recommended_tier: HardwareTier::Mid,
            required_files: &PACKAGE_FILES[1..3],
            source_url: Some(PACKAGE_URL),
            expected_sha256: Some(PACKAGE_SHA256),
            package_type: "zip",
            install_status: install_status(discovery::MODEL),
            engine_compatibility: "Real-ESRGAN NCNN Vulkan 0.2.5.0",
            downloadable: true,
            manual_only: false,
            default_eligible: true,
            availability: "available",
        },
        CatalogItem {
            id: discovery::MODEL_ANIME,
            display_name: "RealESRGAN Anime 6B",
            category: "anime",
            description_key: "model_anime_description",
            supported_image_types: &["anime", "illustration"],
            recommended_tier: HardwareTier::Low,
            required_files: &PACKAGE_FILES[3..5],
            source_url: Some(PACKAGE_URL),
            expected_sha256: Some(PACKAGE_SHA256),
            package_type: "zip",
            install_status: install_status(discovery::MODEL_ANIME),
            engine_compatibility: "Real-ESRGAN NCNN Vulkan 0.2.5.0",
            downloadable: true,
            manual_only: false,
            default_eligible: true,
            availability: "available",
        },
        CatalogItem {
            id: "realesr-general-x4v3",
            display_name: "realesr-general-x4v3",
            category: "general",
            description_key: "model_general_v3_deferred",
            supported_image_types: &["photo"],
            recommended_tier: HardwareTier::Low,
            required_files: &[],
            source_url: None,
            expected_sha256: None,
            package_type: "none",
            install_status: "not_available",
            engine_compatibility: "Deferred",
            downloadable: false,
            manual_only: false,
            default_eligible: false,
            availability: "deferred",
        },
        CatalogItem {
            id: "gfpgan",
            display_name: "GFPGAN Portrait",
            category: "portrait",
            description_key: "model_gfpgan_deferred",
            supported_image_types: &["portrait"],
            recommended_tier: HardwareTier::Mid,
            required_files: &[],
            source_url: None,
            expected_sha256: None,
            package_type: "none",
            install_status: "not_available",
            engine_compatibility: "Deferred",
            downloadable: false,
            manual_only: false,
            default_eligible: false,
            availability: "deferred",
        },
    ];
    ManagedState {
        catalog,
        gpu: GpuSummary {
            names: status.devices.iter().map(|d| d.name.clone()).collect(),
            vendors: status.devices.iter().map(|d| d.vendor.clone()).collect(),
            device_count: status.devices.len(),
            vulkan_available: !status.devices.is_empty(),
            dedicated_memory_bytes: status
                .devices
                .iter()
                .filter_map(|d| d.dedicated_memory_bytes)
                .max(),
            tier,
        },
        recommendation,
        active_engine_type: if status.source.as_deref() == Some("managed") {
            "managed"
        } else if status.location.is_some() {
            "manual"
        } else {
            "none"
        }
        .into(),
        active_model: if selected_model == "auto" {
            "auto"
        } else {
            selected_model
        }
        .into(),
        active_path: status.location.clone(),
        managed_installed,
        manual_installed,
        package_version: PACKAGE_VERSION,
        package_source: PACKAGE_URL,
    }
}

fn source_allowed(url: &str) -> bool {
    url == PACKAGE_URL
}

fn verify_hash_value(actual: &str, expected: &str) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(err(ErrorCode::IntegrityMismatch))
    }
}

pub fn install(data: &Path, url: &str, mut progress: impl FnMut(InstallProgress)) -> Result<()> {
    if !source_allowed(url) {
        return Err(err(ErrorCode::SourceNotAllowed));
    }
    std::fs::create_dir_all(data).map_err(|_| err(ErrorCode::IoError))?;
    if fs2::available_space(data).map_err(|_| err(ErrorCode::IoError))? < MIN_FREE_SPACE {
        return Err(err(ErrorCode::InsufficientDiskSpace));
    }
    progress(InstallProgress {
        stage: "preparing",
        downloaded_bytes: 0,
        total_bytes: None,
    });
    let work_root = data.join("temp");
    std::fs::create_dir_all(&work_root).map_err(|_| err(ErrorCode::IoError))?;
    let work = tempfile::Builder::new()
        .prefix("managed-engine-")
        .tempdir_in(&work_root)
        .map_err(|_| err(ErrorCode::IoError))?;
    let archive_path = work.path().join("package.zip");
    let policy = reqwest::redirect::Policy::custom(|attempt| {
        let host = attempt.url().host_str().unwrap_or_default();
        if attempt.previous().len() >= 5 {
            return attempt.stop();
        }
        if host == "github.com" || host.ends_with(".githubusercontent.com") {
            attempt.follow()
        } else {
            attempt.stop()
        }
    });
    let client = reqwest::blocking::Client::builder()
        .redirect(policy)
        .build()
        .map_err(|_| err(ErrorCode::NetworkUnavailable))?;
    let mut response = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|_| err(ErrorCode::NetworkUnavailable))?;
    let total = response.content_length();
    if total.is_some_and(|n| n > MAX_DOWNLOAD) {
        return Err(err(ErrorCode::ArchiveInvalid));
    }
    progress(InstallProgress {
        stage: "downloading",
        downloaded_bytes: 0,
        total_bytes: total,
    });
    let mut output = File::create(&archive_path).map_err(|_| err(ErrorCode::IoError))?;
    let mut hash = Sha256::new();
    let mut downloaded = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|_| err(ErrorCode::NetworkUnavailable))?;
        if read == 0 {
            break;
        }
        downloaded = downloaded.saturating_add(read as u64);
        if downloaded > MAX_DOWNLOAD {
            return Err(err(ErrorCode::ArchiveInvalid));
        }
        output
            .write_all(&buffer[..read])
            .map_err(|_| err(ErrorCode::IoError))?;
        hash.update(&buffer[..read]);
        progress(InstallProgress {
            stage: "downloading",
            downloaded_bytes: downloaded,
            total_bytes: total,
        });
    }
    output.sync_all().map_err(|_| err(ErrorCode::IoError))?;
    drop(output);
    progress(InstallProgress {
        stage: "verifying",
        downloaded_bytes: downloaded,
        total_bytes: total,
    });
    verify_hash_value(&format!("{:x}", hash.finalize()), PACKAGE_SHA256)?;
    progress(InstallProgress {
        stage: "extracting",
        downloaded_bytes: downloaded,
        total_bytes: total,
    });
    let extracted = work.path().join("extracted");
    extract_expected(&archive_path, &extracted)?;
    let metadata = serde_json::json!({
        "catalogVersion": 1,
        "packageId": PACKAGE_ID,
        "version": PACKAGE_VERSION,
        "sourceUrl": PACKAGE_URL,
        "sha256": PACKAGE_SHA256,
        "installedAt": crate::core::processing::task::now_ms()
    });
    std::fs::write(
        extracted.join("package.json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .map_err(|_| err(ErrorCode::IoError))?;
    progress(InstallProgress {
        stage: "validating",
        downloaded_bytes: downloaded,
        total_bytes: total,
    });
    let destination = managed_root(data);
    crate::services::installation::install_managed(&extracted, &destination, PACKAGE_VERSION)?;
    progress(InstallProgress {
        stage: "activating",
        downloaded_bytes: downloaded,
        total_bytes: total,
    });
    progress(InstallProgress {
        stage: "completed",
        downloaded_bytes: downloaded,
        total_bytes: total,
    });
    Ok(())
}

fn safe_archive_name(name: &str) -> bool {
    !name.contains('\\')
        && Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

fn extract_expected(archive: &Path, output: &Path) -> Result<()> {
    let file = File::open(archive).map_err(|_| err(ErrorCode::ArchiveInvalid))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_| err(ErrorCode::ArchiveInvalid))?;
    let expected: BTreeSet<&str> = PACKAGE_FILES.into_iter().collect();
    let mut found = BTreeSet::new();
    let mut expanded = 0u64;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|_| err(ErrorCode::ArchiveInvalid))?;
        let name = entry.name().trim_end_matches('/').to_string();
        if name.is_empty() {
            continue;
        }
        if !safe_archive_name(&name) {
            return Err(err(ErrorCode::ArchiveInvalid));
        }
        if !expected.contains(name.as_str()) {
            continue;
        }
        if !found.insert(name.clone()) {
            return Err(err(ErrorCode::ArchiveInvalid));
        }
        expanded = expanded.saturating_add(entry.size());
        if expanded > 128 * 1024 * 1024 || entry.size() > 64 * 1024 * 1024 {
            return Err(err(ErrorCode::ArchiveInvalid));
        }
        let target = output.join(&name);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|_| err(ErrorCode::IoError))?;
        }
        let mut file = File::create(target).map_err(|_| err(ErrorCode::IoError))?;
        let declared = entry.size();
        let copied = std::io::copy(&mut (&mut entry).take(declared + 1), &mut file)
            .map_err(|_| err(ErrorCode::ArchiveInvalid))?;
        if copied != declared {
            return Err(err(ErrorCode::ArchiveInvalid));
        }
    }
    if found.len() != expected.len() {
        return Err(err(ErrorCode::ArchiveInvalid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn device(kind: &str, gib: Option<u64>) -> crate::core::processing::engine::GpuDevice {
        crate::core::processing::engine::GpuDevice {
            id: "x".into(),
            index: 0,
            name: "GPU".into(),
            vendor: "Other".into(),
            device_type: kind.into(),
            dedicated_memory_bytes: gib.map(|v| v * 1024 * 1024 * 1024),
        }
    }
    fn status(devices: Vec<crate::core::processing::engine::GpuDevice>) -> EngineStatus {
        EngineStatus {
            devices,
            id: "realesrgan-ncnn-vulkan".into(),
            name: "Real-ESRGAN".into(),
            model: discovery::MODEL.into(),
            models: vec![discovery::MODEL.into()],
            version: None,
            availability: crate::core::processing::engine::Availability::Available,
            message: String::new(),
            location: None,
            install_dir: String::new(),
            source: None,
            scales: vec![2, 4, 8, 12],
            formats: vec!["PNG".into()],
            gpu_devices: vec![],
            gpu_selection: "Auto".into(),
        }
    }
    fn fixture_engine(root: &Path, anime: bool) -> PathBuf {
        let source = root.join("source");
        std::fs::create_dir_all(source.join("models")).unwrap();
        std::fs::write(
            source.join("realesrgan-ncnn-vulkan.exe"),
            b"fixture executable",
        )
        .unwrap();
        std::fs::write(
            source.join("models/realesrgan-x4plus.param"),
            b"7767517\nfixture",
        )
        .unwrap();
        std::fs::write(source.join("models/realesrgan-x4plus.bin"), b"fixture").unwrap();
        if anime {
            std::fs::write(
                source.join("models/realesrgan-x4plus-anime.param"),
                b"7767517\nfixture",
            )
            .unwrap();
            std::fs::write(
                source.join("models/realesrgan-x4plus-anime.bin"),
                b"fixture",
            )
            .unwrap();
        }
        source
    }
    #[test]
    fn tiers_are_conservative_and_deterministic() {
        assert_eq!(hardware_tier(&[]), HardwareTier::Unsupported);
        assert_eq!(
            hardware_tier(&[device("integrated", None)]),
            HardwareTier::Low
        );
        assert_eq!(
            hardware_tier(&[device("discrete", Some(6))]),
            HardwareTier::Mid
        );
        assert_eq!(
            hardware_tier(&[device("discrete", Some(12))]),
            HardwareTier::High
        );
        assert_eq!(
            hardware_tier(&[device("discrete", Some(24))]),
            HardwareTier::Ultra
        );
    }
    #[test]
    fn recommendation_routes_anime_and_uses_the_only_manual_engine() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("data");
        let source = fixture_engine(root.path(), false);
        crate::services::installation::install(&source, &data.join("engines/manual")).unwrap();
        let gpu = status(vec![device("discrete", Some(8))]);
        let general = state(&data, &gpu, "Photo", "Auto", "auto");
        assert_eq!(
            general.recommendation.model_id.as_deref(),
            Some(discovery::MODEL)
        );
        assert!(general.recommendation.installed);
        assert!(general.manual_installed);
        let anime = state(&data, &gpu, "Anime / Illustration", "Auto", "auto");
        assert_eq!(
            anime.recommendation.model_id.as_deref(),
            Some(discovery::MODEL_ANIME)
        );
        assert!(!anime.recommendation.installed);
        assert_eq!(anime.recommendation.action, "download");
    }

    #[test]
    fn fresh_install_with_vulkan_device_recommends_download() {
        let data = tempfile::tempdir().unwrap();
        let gpu = status(vec![device("discrete", Some(24))]);
        let state = state(data.path(), &gpu, "Photo", "Auto", "auto");
        assert_eq!(
            state.recommendation.model_id.as_deref(),
            Some(discovery::MODEL)
        );
        assert_eq!(state.recommendation.action, "download");
        assert!(!state.recommendation.installed);
        assert_eq!(state.gpu.tier, HardwareTier::Ultra);
    }
    #[test]
    fn failed_managed_request_does_not_touch_manual_engine() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("data");
        let source = fixture_engine(root.path(), false);
        crate::services::installation::install(&source, &data.join("engines/manual")).unwrap();
        let before = std::fs::read(manual_root(&data).join("manifest.json")).unwrap();
        assert!(install(&data, "https://example.invalid/package.zip", |_| {}).is_err());
        assert_eq!(
            std::fs::read(manual_root(&data).join("manifest.json")).unwrap(),
            before
        );
    }
    #[test]
    fn rejects_unapproved_sources_and_unsafe_archive_names() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            install(dir.path(), "https://example.invalid/a.zip", |_| {})
                .unwrap_err()
                .code,
            ErrorCode::SourceNotAllowed
        );
        assert!(!safe_archive_name("../evil.exe"));
        assert!(!safe_archive_name("models\\..\\evil.exe"));
        assert!(safe_archive_name("models/realesrgan-x4plus.bin"));
        assert_eq!(
            verify_hash_value("tampered", PACKAGE_SHA256)
                .unwrap_err()
                .code,
            ErrorCode::IntegrityMismatch
        );
    }
    #[test]
    fn missing_required_files_and_zip_slip_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("fixture.zip");
        {
            let file = File::create(&archive).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file("../evil.exe", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"x").unwrap();
            writer.finish().unwrap();
        }
        assert_eq!(
            extract_expected(&archive, &dir.path().join("out"))
                .unwrap_err()
                .code,
            ErrorCode::ArchiveInvalid
        );
        let archive = dir.path().join("missing.zip");
        let file = File::create(&archive).unwrap();
        zip::ZipWriter::new(file).finish().unwrap();
        assert_eq!(
            extract_expected(&archive, &dir.path().join("out2"))
                .unwrap_err()
                .code,
            ErrorCode::ArchiveInvalid
        );
    }
    #[test]
    #[ignore = "Opt-in real official-package download and Vulkan health check"]
    fn real_official_package_download_install() {
        if std::env::var_os("CLEARCE_TEST_MANAGED_DOWNLOAD").is_none() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        install(root.path(), PACKAGE_URL, |_| {}).unwrap();
        let found = discovery::discover(&[(managed_root(root.path()), "managed".into())]).unwrap();
        assert!(found.models.iter().any(|m| m == discovery::MODEL));
        assert!(found.models.iter().any(|m| m == discovery::MODEL_ANIME));
        use crate::core::processing::engine::{Cancellation, EngineInput, EnhancementEngine};
        let input = root.path().join("input.png");
        let output = root.path().join("output.png");
        image::DynamicImage::new_rgb8(16, 12).save(&input).unwrap();
        let runtime = crate::engines::realesrgan::RealEsrgan {
            locations: vec![(managed_root(root.path()), "managed".into())],
            install_dir: managed_root(root.path()),
        };
        runtime
            .process(
                EngineInput {
                    input: &input,
                    output: &output,
                    gpu_index: None,
                    tile_size: 32,
                    model: "auto",
                    engine_mode: "Auto",
                    category: "Photo",
                },
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(image::image_dimensions(output).unwrap(), (64, 48));
    }
}
