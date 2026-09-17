use enhancece_lib::{
    core::processing::{
        engine::*,
        errors::{err, ErrorCode, Result},
        task::{OutputFormat, ProcessRequest, Task, TaskStatus},
    },
    engines::realesrgan::RealEsrgan,
    services::processing_service::ProcessingService,
    storage::History,
};
use image::{imageops::FilterType, DynamicImage};
use std::{
    path::Path,
    sync::{mpsc, Arc},
    time::Duration,
};
struct Mock {
    wait: bool,
    bad_dimensions: bool,
    fail: bool,
}
impl EnhancementEngine for Mock {
    fn status(&self) -> EngineStatus {
        EngineStatus {
            id: "test-only".into(),
            name: "Test adapter".into(),
            model: "fixture".into(),
            models: vec!["fixture".into()],
            version: None,
            availability: Availability::Available,
            message: String::new(),
            location: None,
            install_dir: String::new(),
            source: None,
            scales: vec![2, 4],
            formats: vec!["PNG".into()],
            gpu_devices: vec![],
            devices: vec![],
            gpu_selection: "test".into(),
        }
    }
    fn native_scale(&self) -> u32 {
        4
    }
    fn process(&self, input: EngineInput<'_>, cancel: &Cancellation) -> Result<()> {
        if self.wait {
            loop {
                cancel.check()?;
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        if self.fail {
            return Err(err(ErrorCode::InferenceFailed));
        }
        let im = image::open(input.input).unwrap();
        let scale = if self.bad_dimensions { 3 } else { 4 };
        im.resize_exact(im.width() * scale, im.height() * scale, FilterType::Nearest)
            .save(input.output)
            .unwrap();
        Ok(())
    }
}
fn request(path: &Path, output: &Path, scale: u32, format: OutputFormat) -> ProcessRequest {
    ProcessRequest {
        input_path: path.to_string_lossy().into(),
        output_dir: Some(output.to_string_lossy().into()),
        scale,
        format,
        mode: "Photo".into(),
        gpu_index: None,
        gpu_id: None,
        tile_size: 128,
        model_id: "auto".into(),
        engine_mode: "Auto".into(),
    }
}
fn fixture(root: &Path) -> std::path::PathBuf {
    let path = root.join("çağrı & spaced image.png");
    DynamicImage::new_rgb8(12, 8).save(&path).unwrap();
    path
}
fn run(service: &Arc<ProcessingService>, req: ProcessRequest) -> Task {
    let (tx, rx) = mpsc::channel();
    service
        .start(
            req,
            Arc::new(move |t| {
                let _ = tx.send(t);
            }),
        )
        .unwrap();
    loop {
        let t = rx.recv_timeout(Duration::from_secs(120)).unwrap();
        if t.status.terminal() {
            return t;
        }
    }
}
#[test]
fn outputs_are_valid_persisted_and_never_clobber_source_or_previous_result() {
    let d = tempfile::tempdir().unwrap();
    let path = fixture(d.path());
    let original = std::fs::read(&path).unwrap();
    let service = ProcessingService::new(
        Arc::new(Mock {
            wait: false,
            bad_dimensions: false,
            fail: false,
        }),
        d.path().join("data"),
        d.path().into(),
    )
    .unwrap();
    let mut names = vec![];
    for (scale, format) in [
        (2, OutputFormat::PNG),
        (2, OutputFormat::PNG),
        (4, OutputFormat::JPG),
        (2, OutputFormat::WEBP),
    ] {
        let task = run(&service, request(&path, d.path(), scale, format));
        assert_eq!(task.status, TaskStatus::Completed);
        let out = task.output.unwrap();
        let decoded = image::open(&out.path).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (12 * scale, 8 * scale));
        assert!(!names.contains(&out.path));
        names.push(out.path);
    }
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(&service.temp_root).unwrap().count(), 0);
    drop(service);
    let history = History::open(&d.path().join("data/history.sqlite3")).unwrap();
    assert_eq!(history.list().unwrap().len(), 4);
    let id = history.list().unwrap()[0].id.clone();
    history.delete(&id).unwrap();
    assert!(names.iter().all(|p| Path::new(p).is_file()));
}
#[test]
fn failure_and_wrong_dimensions_are_not_success_and_clean_temp() {
    for (bad_dimensions, fail, expected) in [
        (true, false, ErrorCode::DimensionMismatch),
        (false, true, ErrorCode::InferenceFailed),
    ] {
        let d = tempfile::tempdir().unwrap();
        let path = fixture(d.path());
        let service = ProcessingService::new(
            Arc::new(Mock {
                wait: false,
                bad_dimensions,
                fail,
            }),
            d.path().join("data"),
            d.path().into(),
        )
        .unwrap();
        let task = run(&service, request(&path, d.path(), 4, OutputFormat::PNG));
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error.unwrap().code, expected);
        assert!(task.output.is_none());
        assert_eq!(std::fs::read_dir(&service.temp_root).unwrap().count(), 0);
        assert!(!std::fs::read_dir(d.path()).unwrap().any(|p| p
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("enhanced")));
    }
}
#[test]
fn cancel_is_terminal_cleans_files_and_busy_rejects_second_job() {
    let d = tempfile::tempdir().unwrap();
    let path = fixture(d.path());
    let original = std::fs::read(&path).unwrap();
    let service = ProcessingService::new(
        Arc::new(Mock {
            wait: true,
            bad_dimensions: false,
            fail: false,
        }),
        d.path().join("data"),
        d.path().into(),
    )
    .unwrap();
    let (tx, rx) = mpsc::channel();
    let req = request(&path, d.path(), 4, OutputFormat::PNG);
    let first = service
        .start(
            req.clone(),
            Arc::new(move |t| {
                let _ = tx.send(t);
            }),
        )
        .unwrap();
    assert_eq!(
        service.start(req, Arc::new(|_| {})).unwrap_err().code,
        ErrorCode::Busy
    );
    loop {
        let task = rx.recv_timeout(Duration::from_secs(10)).unwrap();
        if task.phase.starts_with("Pass 1") {
            break;
        }
    }
    service.cancel(&first.id).unwrap();
    loop {
        let task = rx.recv_timeout(Duration::from_secs(10)).unwrap();
        if task.status.terminal() {
            assert_eq!(task.status, TaskStatus::Cancelled);
            break;
        }
    }
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(&service.temp_root).unwrap().count(), 0);
}
#[test]
fn missing_engine_and_unwritable_destination_return_typed_errors() {
    let d = tempfile::tempdir().unwrap();
    let path = fixture(d.path());
    let missing = d.path().join("missing");
    let engine = RealEsrgan {
        locations: vec![(missing.clone(), "test".into())],
        install_dir: missing,
    };
    let service =
        ProcessingService::new(Arc::new(engine), d.path().join("data"), d.path().into()).unwrap();
    assert_eq!(
        run(&service, request(&path, d.path(), 2, OutputFormat::PNG))
            .error
            .unwrap()
            .code,
        ErrorCode::EngineNotFound
    );
    let service = ProcessingService::new(
        Arc::new(Mock {
            wait: false,
            bad_dimensions: false,
            fail: false,
        }),
        d.path().join("other"),
        d.path().into(),
    )
    .unwrap();
    assert_eq!(
        run(
            &service,
            request(&path, &d.path().join("not-created"), 2, OutputFormat::PNG)
        )
        .error
        .unwrap()
        .code,
        ErrorCode::OutputNotWritable
    );
}
#[test]
fn unknown_output_format_is_rejected_by_deserialization() {
    assert!(serde_json::from_str::<OutputFormat>("\"EXE\"").is_err());
}
#[test]
#[ignore = "Requires opt-in ENHANCECE_TEST_ENGINE_DIR and a working Vulkan GPU"]
fn real_gpu_inference() {
    let engine_dir = std::path::PathBuf::from(
        std::env::var_os("ENHANCECE_TEST_ENGINE_DIR").expect("engine path required"),
    );
    let temp = tempfile::tempdir().unwrap();
    let root = std::env::var_os("ENHANCECE_TEST_OUTPUT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temp.path().into());
    std::fs::create_dir_all(&root).unwrap();
    let source = root.join("real smoke çağrı.png");
    assert!(!source.exists(), "Use a fresh smoke output directory");
    image::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../public/assets/portrait.png"))
        .unwrap()
        .resize_exact(192, 128, FilterType::Lanczos3)
        .save(&source)
        .unwrap();
    let original = std::fs::read(&source).unwrap();
    let jpeg_source = root.join("real smoke çağrı.jpg");
    let webp_source = root.join("real smoke çağrı.webp");
    let decoded = image::open(&source).unwrap();
    decoded.save(&jpeg_source).unwrap();
    decoded.save(&webp_source).unwrap();
    let jpeg_original = std::fs::read(&jpeg_source).unwrap();
    let webp_original = std::fs::read(&webp_source).unwrap();
    let engine = RealEsrgan {
        locations: vec![(engine_dir.clone(), "opt-in test".into())],
        install_dir: engine_dir,
    };
    let status = engine.status();
    println!("ENGINE {}", serde_json::to_string(&status).unwrap());
    assert!(matches!(status.availability, Availability::Available));
    let service =
        ProcessingService::new(Arc::new(engine), root.join("data"), root.clone()).unwrap();
    for (scale, format) in [
        (2, OutputFormat::PNG),
        (4, OutputFormat::PNG),
        (2, OutputFormat::JPG),
        (4, OutputFormat::WEBP),
    ] {
        let input = match format {
            OutputFormat::JPG => &jpeg_source,
            OutputFormat::WEBP => &webp_source,
            _ => &source,
        };
        let task = run(&service, request(input, &root, scale, format));
        println!("RESULT {}", serde_json::to_string(&task).unwrap());
        assert_eq!(task.status, TaskStatus::Completed);
        let out = task.output.unwrap();
        assert_eq!((out.width, out.height), (192 * scale, 128 * scale));
        assert!(Path::new(&out.path).is_file());
    }
    assert_eq!(std::fs::read(source).unwrap(), original);
    assert_eq!(std::fs::read(jpeg_source).unwrap(), jpeg_original);
    assert_eq!(std::fs::read(webp_source).unwrap(), webp_original);
    assert_eq!(std::fs::read_dir(&service.temp_root).unwrap().count(), 0);
}
