#[cfg(windows)]
mod containment;
pub mod discovery;
mod gpu;
pub mod runner;
use crate::core::processing::{
    engine::*,
    errors::{err, ErrorCode, Result},
};
use std::{ffi::OsString, path::PathBuf, time::Duration};
pub struct RealEsrgan {
    pub locations: Vec<(PathBuf, String)>,
    pub install_dir: PathBuf,
}
pub fn arguments(input: &EngineInput<'_>, models: PathBuf) -> Vec<OsString> {
    let mut args = vec![
        "-i".into(),
        input.input.as_os_str().into(),
        "-o".into(),
        input.output.as_os_str().into(),
        "-m".into(),
        models.into_os_string(),
        "-n".into(),
        discovery::MODEL.into(),
        "-s".into(),
        "4".into(),
        "-t".into(),
        input.tile_size.to_string().into(),
        "-j".into(),
        "1:1:1".into(),
        "-f".into(),
        "png".into(),
    ];
    if let Some(index) = input.gpu_index {
        args.extend(["-g".into(), index.to_string().into()]);
    }
    args
}
impl EnhancementEngine for RealEsrgan {
    fn native_scale(&self) -> u32 {
        4
    }
    fn status(&self) -> EngineStatus {
        let mut status = EngineStatus {
            devices: vec![],
            id: "realesrgan-ncnn-vulkan".into(),
            name: "Real-ESRGAN NCNN Vulkan".into(),
            model: discovery::MODEL.into(),
            version: None,
            availability: Availability::Missing,
            message: String::new(),
            location: None,
            install_dir: self.install_dir.to_string_lossy().into(),
            source: None,
            scales: vec![2, 4, 8, 12],
            formats: vec!["PNG".into(), "JPG".into(), "WEBP".into()],
            gpu_devices: vec![],
            gpu_selection: "Automatic (NCNN default GPU)".into(),
        };
        let install = match discovery::discover(&self.locations) {
            Ok(v) => v,
            Err(e) => {
                status.availability = if e.code == ErrorCode::EngineNotFound {
                    Availability::Missing
                } else {
                    Availability::Invalid
                };
                status.message = e.message;
                return status;
            }
        };
        status.location = Some(install.root.to_string_lossy().into());
        status.source = Some(install.source);
        status.version = install.version;
        match runner::run(
            &install.root.join(discovery::REQUIRED[0]),
            &install.root,
            &["-h".into()],
            &Cancellation::default(),
            Duration::from_secs(5),
        ) {
            Ok(out)
                if [
                    "-i input-path",
                    "-o output-path",
                    "-m model-path",
                    "-n model-name",
                    "-s scale",
                    "-g gpu-id",
                    "-f format",
                ]
                .iter()
                .all(|v| out.text.contains(v)) => {}
            _ => {
                status.availability = Availability::Invalid;
                status.message = err(ErrorCode::EngineInvalid).message;
                return status;
            }
        }
        status.gpu_devices = gpu::devices();
        // Ask NCNN itself for its indexed device list. A nonexistent input
        // exercises initialization only; it never performs inference or writes output.
        if let Ok(probe) = tempfile::tempdir() {
            let args = arguments(
                &EngineInput {
                    input: &probe.path().join("missing.png"),
                    output: &probe.path().join("unused.png"),
                    gpu_index: None,
                    tile_size: 32,
                },
                PathBuf::from("models"),
            );
            if let Ok(output) = runner::run(
                &install.root.join(discovery::REQUIRED[0]),
                &install.root,
                &args,
                &Cancellation::default(),
                Duration::from_secs(10),
            ) {
                status.devices = gpu::map_ncnn(&output.text, &gpu::identities());
            }
        }
        if status.gpu_devices.is_empty() {
            status.availability = Availability::Unsupported;
            status.message = err(ErrorCode::GpuUnavailable).message;
        } else {
            status.availability = Availability::Available;
            status.message =
                "Engine and Vulkan device detected. Ready for local enhancement.".into();
        }
        status
    }
    fn process(&self, input: EngineInput<'_>, cancel: &Cancellation) -> Result<()> {
        let install = discovery::discover(&self.locations)?;
        cancel.check()?;
        log::info!("event=engine_discovered engine=realesrgan-ncnn-vulkan");
        // This CLI misclassifies Windows verbatim (\\?\) model paths as relative.
        // The child cwd is the verified installation root, so a fixed relative
        // models directory resolves correctly without weakening discovery checks.
        let args = arguments(&input, PathBuf::from("models"));
        let out = runner::run(
            &install.root.join(discovery::REQUIRED[0]),
            &install.root,
            &args,
            cancel,
            Duration::from_secs(1200),
        )?;
        runner::require_success(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn args_preserve_unicode_spaces_and_shell_characters() {
        let input = PathBuf::from("E:/Fotoğraflar/a & b.png");
        let output = PathBuf::from("E:/out put/new.png");
        let args = arguments(
            &EngineInput {
                input: &input,
                output: &output,
                gpu_index: Some(0),
                tile_size: 128,
            },
            PathBuf::from("E:/my models"),
        );
        assert_eq!(args[1], input.as_os_str());
        assert_eq!(args[3], output.as_os_str());
        assert_eq!(args[9], "4");
        assert!(args.iter().any(|a| a == "-g"));
    }
}
