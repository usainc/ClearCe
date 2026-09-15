use super::{
    engine::{Cancellation, EngineInput, EnhancementEngine},
    errors::{err, ErrorCode, Result},
    safety::SafetyLimits,
    task::{ImageInfo, ProcessRequest},
    task::{PipelineStage, TaskStatus},
};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageDecoder, ImageReader};
use std::{
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};
pub const MAX_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_PIXELS: u64 = 6_000_000;
pub fn decode(path: &Path, max_pixels: u64) -> Result<DynamicImage> {
    let meta = path.metadata().map_err(|_| err(ErrorCode::InvalidImage))?;
    if !meta.is_file() {
        return Err(err(ErrorCode::InvalidImage));
    }
    if meta.len()
        > max_pixels
            .saturating_mul(8)
            .saturating_add(16 * 1024 * 1024)
    {
        return Err(err(ErrorCode::ImageTooLarge));
    }
    let file = File::open(path).map_err(|_| err(ErrorCode::InvalidImage))?;
    let mut reader = ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .map_err(|_| err(ErrorCode::InvalidImage))?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP)
    ) {
        return Err(err(ErrorCode::UnsupportedFormat));
    }
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(800 * 1024 * 1024);
    reader.limits(limits);
    // Inspect dimensions before allocating a decoded image.
    let dimensions = ImageReader::open(path)
        .map_err(|_| err(ErrorCode::InvalidImage))?
        .with_guessed_format()
        .map_err(|_| err(ErrorCode::InvalidImage))?
        .into_dimensions()
        .map_err(|_| err(ErrorCode::InvalidImage))?;
    if dimensions.0 == 0 || dimensions.1 == 0 {
        return Err(err(ErrorCode::InvalidImage));
    }
    if u64::from(dimensions.0) * u64::from(dimensions.1) > max_pixels {
        return Err(err(ErrorCode::ImageTooLarge));
    }
    let mut decoder = reader
        .into_decoder()
        .map_err(|_| err(ErrorCode::InvalidImage))?;
    let orientation = decoder
        .orientation()
        .map_err(|_| err(ErrorCode::InvalidImage))?;
    let mut decoded =
        DynamicImage::from_decoder(decoder).map_err(|_| err(ErrorCode::InvalidImage))?;
    decoded.apply_orientation(orientation);
    Ok(decoded)
}
pub fn inspect(path: &Path) -> Result<ImageInfo> {
    if !path.is_absolute() {
        return Err(err(ErrorCode::InvalidImage));
    }
    let path = path
        .canonicalize()
        .map_err(|_| err(ErrorCode::InvalidImage))?;
    let bytes = path
        .metadata()
        .map_err(|_| err(ErrorCode::InvalidImage))?
        .len();
    if bytes > MAX_BYTES {
        return Err(err(ErrorCode::ImageTooLarge));
    }
    let img = decode(&path, MAX_PIXELS)?;
    Ok(info(&path, &img, bytes))
}
pub fn info(path: &Path, img: &DynamicImage, bytes: u64) -> ImageInfo {
    ImageInfo {
        path: path.to_string_lossy().into(),
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
        width: img.width(),
        height: img.height(),
        bytes,
    }
}
pub fn output_name(input: &Path, scale: u32, extension: &str, serial: u32) -> String {
    let stem: String = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | ' '))
        .take(100)
        .collect();
    let stem = if stem.trim().is_empty() {
        "image"
    } else {
        stem.trim()
    };
    let suffix = if serial == 0 {
        String::new()
    } else {
        format!("_{serial}")
    };
    format!("{stem}_enhanced_{scale}x{suffix}.{extension}")
}
pub struct PreparedOutput {
    pub file: tempfile::NamedTempFile,
    pub folder: PathBuf,
    pub input: ImageInfo,
    pub width: u32,
    pub height: u32,
}
pub struct PipelineEvent {
    pub phase: String,
    pub pass: Option<usize>,
    pub completed: bool,
}
pub fn plan(scale: u32) -> Result<Vec<PipelineStage>> {
    let factors = match scale {
        2 => vec![2],
        4 => vec![4],
        8 => vec![4, 2],
        12 => vec![4, 3],
        _ => return Err(err(ErrorCode::UnsupportedScale)),
    };
    Ok(factors
        .into_iter()
        .enumerate()
        .map(|(i, scale)| PipelineStage {
            label: format!(
                "Pass {} — native 4x AI{}",
                i + 1,
                if scale == 4 {
                    String::new()
                } else {
                    format!(", resize to effective {scale}x")
                }
            ),
            scale,
            status: TaskStatus::Queued,
        })
        .collect())
}
pub fn process(
    engine: &dyn EnhancementEngine,
    request: &ProcessRequest,
    temp_root: &Path,
    default_output: &Path,
    cancel: &Cancellation,
    phase: &dyn Fn(&str),
) -> Result<PreparedOutput> {
    process_with_limits(
        engine,
        request,
        temp_root,
        default_output,
        cancel,
        &SafetyLimits::default(),
        &|event| phase(&event.phase),
    )
}
pub fn process_with_limits(
    engine: &dyn EnhancementEngine,
    request: &ProcessRequest,
    temp_root: &Path,
    default_output: &Path,
    cancel: &Cancellation,
    limits: &SafetyLimits,
    events: &dyn Fn(PipelineEvent),
) -> Result<PreparedOutput> {
    let phase = |text: &str| {
        events(PipelineEvent {
            phase: text.into(),
            pass: None,
            completed: false,
        })
    };
    request.validate()?;
    cancel.check()?;
    let source = Path::new(&request.input_path);
    let input = inspect(source)?;
    let estimate = limits.estimate(
        input.width,
        input.height,
        request.scale,
        engine.native_scale(),
    )?;
    phase("Validated target resolution and working-memory estimate");
    let folder = match request
        .output_dir
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        Some(s) => PathBuf::from(s),
        None => default_output.to_path_buf(),
    };
    if !folder.is_absolute() {
        return Err(err(ErrorCode::OutputNotWritable));
    }
    if folder == default_output {
        std::fs::create_dir_all(&folder).map_err(|_| err(ErrorCode::OutputNotWritable))?;
    }
    let folder = folder
        .canonicalize()
        .map_err(|_| err(ErrorCode::OutputNotWritable))?;
    let file = tempfile::Builder::new()
        .prefix(".enhancece-")
        .suffix(".tmp")
        .tempfile_in(&folder)
        .map_err(|_| err(ErrorCode::OutputNotWritable))?;
    std::fs::create_dir_all(temp_root).map_err(|_| err(ErrorCode::IoError))?;
    limits.check_disk(&estimate, temp_root, &folder)?;
    let work = tempfile::Builder::new()
        .prefix("job-")
        .tempdir_in(temp_root)
        .map_err(|_| err(ErrorCode::IoError))?;
    let result = (|| {
        phase("Preparing image");
        let decoded = decode(Path::new(&input.path), MAX_PIXELS)?;
        cancel.check()?;
        let prepared = work.path().join("input.png");
        decoded
            .save_with_format(&prepared, image::ImageFormat::Png)
            .map_err(|_| err(ErrorCode::IoError))?;
        drop(decoded);
        let stages = plan(request.scale)?;
        let mut current = prepared;
        let mut size = (input.width, input.height);
        for (index, stage) in stages.iter().enumerate() {
            cancel.check()?;
            limits.check_disk(&estimate, temp_root, &folder)?;
            events(PipelineEvent {
                phase: stage.label.clone(),
                pass: Some(index),
                completed: false,
            });
            let intermediate = work.path().join(format!("pass{}.png", index + 1));
            engine
                .process(
                    EngineInput {
                        input: &current,
                        output: &intermediate,
                        gpu_index: request.gpu_index,
                        tile_size: request.tile_size,
                    },
                    cancel,
                )
                .map_err(|e| {
                    if stages.len() > 1 && e.code == ErrorCode::InferenceFailed {
                        let mut error = err(ErrorCode::PipelinePassFailed);
                        error.message = format!(
                            "AI pass {} failed. No final output was accepted.",
                            index + 1
                        );
                        error
                    } else {
                        e
                    }
                })?;
            cancel.check()?;
            if !intermediate.is_file() {
                return Err(err(ErrorCode::OutputMissing));
            }
            let verified = decode(&intermediate, limits.max_intermediate_pixels)?;
            size = (
                size.0 * engine.native_scale(),
                size.1 * engine.native_scale(),
            );
            if verified.dimensions() != size {
                return Err(err(ErrorCode::DimensionMismatch));
            }
            drop(verified);
            current = intermediate;
            // The last pass's effective 2x/3x result is finalized below after
            // real native 4x inference; do not mark it complete prematurely.
            if index + 1 < stages.len() {
                events(PipelineEvent {
                    phase: format!("{} completed", stage.label),
                    pass: Some(index),
                    completed: true,
                });
            }
        }
        let generated = decode(&current, limits.max_intermediate_pixels)?;
        phase("Writing output");
        let target = (input.width * request.scale, input.height * request.scale);
        // x4plus is intrinsically 4x. 2x is real 4x AI inference followed by Lanczos3 reduction.
        let generated = if generated.dimensions() != target {
            generated.resize_exact(target.0, target.1, FilterType::Lanczos3)
        } else {
            generated
        };
        cancel.check()?;
        events(PipelineEvent {
            phase: format!("{} completed", stages.last().unwrap().label),
            pass: Some(stages.len() - 1),
            completed: true,
        });
        phase("Encoding output");
        let mut file = file;
        if request.format == super::task::OutputFormat::JPG {
            DynamicImage::ImageRgb8(generated.to_rgb8())
                .write_to(&mut file, request.format.image_format())
        } else {
            generated.write_to(&mut file, request.format.image_format())
        }
        .map_err(|_| err(ErrorCode::IoError))?;
        file.flush().map_err(|_| err(ErrorCode::IoError))?;
        file.as_file()
            .sync_all()
            .map_err(|_| err(ErrorCode::IoError))?;
        cancel.check()?;
        phase("Validating final output");
        let verified = decode(file.path(), MAX_PIXELS * 16)?;
        if verified.dimensions() != target {
            return Err(err(ErrorCode::DimensionMismatch));
        }
        Ok(PreparedOutput {
            file,
            folder,
            input,
            width: target.0,
            height: target.1,
        })
    })();
    work.close().map_err(|_| err(ErrorCode::IoError))?;
    log::info!("event=cleanup_completed");
    result
}
pub fn publish(mut output: PreparedOutput, request: &ProcessRequest) -> Result<ImageInfo> {
    let bytes = output
        .file
        .as_file()
        .metadata()
        .map_err(|_| err(ErrorCode::IoError))?
        .len();
    for serial in 0..10000 {
        let path = output.folder.join(output_name(
            Path::new(&output.input.path),
            request.scale,
            request.format.extension(),
            serial,
        ));
        match output.file.persist_noclobber(&path) {
            Ok(_) => {
                return Ok(ImageInfo {
                    path: path.to_string_lossy().into(),
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    width: output.width,
                    height: output.height,
                    bytes,
                })
            }
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
                output.file = e.file;
            }
            Err(_) => return Err(err(ErrorCode::OutputNotWritable)),
        }
    }
    Err(err(ErrorCode::OutputNotWritable))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jpeg_exif_orientation_is_applied_before_dimension_validation() {
        use image::ImageEncoder;
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("rotated.jpg");
        let mut encoder = image::codecs::jpeg::JpegEncoder::new(File::create(&path).unwrap());
        // Little-endian TIFF: one orientation tag with value 6 (90 degrees).
        encoder
            .set_exif_metadata(vec![
                73, 73, 42, 0, 8, 0, 0, 0, 1, 0, 18, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
            ])
            .unwrap();
        encoder
            .encode_image(&DynamicImage::new_rgb8(12, 8))
            .unwrap();
        let info = inspect(&path).unwrap();
        assert_eq!((info.width, info.height), (8, 12));
    }
    #[test]
    fn names_preserve_unicode_and_avoid_original() {
        let path = Path::new("/tmp/çağrı & a.jpg");
        assert_eq!(output_name(path, 2, "png", 1), "çağrı  a_enhanced_2x_1.png");
    }
    #[test]
    fn invalid_content_is_not_trusted_by_extension() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.png");
        std::fs::write(&p, b"not an image").unwrap();
        assert!(inspect(&p).is_err());
    }
    #[test]
    fn signature_accepts_image_with_wrong_extension() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("photo.dat");
        DynamicImage::new_rgb8(3, 2)
            .save_with_format(&p, image::ImageFormat::Png)
            .unwrap();
        assert_eq!(inspect(&p).unwrap().width, 3);
    }
}
