use enhancece_lib::{
    core::processing::{
        engine::*,
        errors::{err, ErrorCode, Result},
        pipeline,
        safety::SafetyLimits,
        task::{OutputFormat, ProcessRequest, Task, TaskStatus},
    },
    services::{
        batch_queue::{BatchQueueService, QueueItem, QueueSnapshot},
        processing_service::ProcessingService,
    },
};
use image::{imageops::FilterType, DynamicImage};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};
#[derive(Default)]
struct Control {
    calls: AtomicUsize,
    active: AtomicUsize,
    maximum: AtomicUsize,
    hold: AtomicUsize,
    release: AtomicBool,
    fail: AtomicUsize,
}
struct Adapter(Arc<Control>);
#[test]
fn readiness_session_lock_and_startup_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    std::fs::create_dir_all(data.join("temp/job-stale")).unwrap();
    std::fs::write(data.join("temp/job-stale/input.png"), b"derived").unwrap();
    let service = ProcessingService::new(
        Arc::new(Adapter(Arc::new(Control::default()))),
        data.clone(),
        root.path().join("outputs"),
    )
    .unwrap();
    assert!(!data.join("temp/job-stale").exists());
    assert_eq!(service.health(None, None).state, "ready");
    assert_eq!(
        service.health(None, Some("stale-device")).state,
        "requires attention"
    );
    assert_eq!(
        service
            .health(root.path().join("missing").to_str(), None)
            .state,
        "requires attention"
    );
    assert!(ProcessingService::new(
        Arc::new(Adapter(Arc::new(Control::default()))),
        data,
        root.path().join("outputs")
    )
    .is_err());
}
struct Active(Arc<Control>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
    }
}
impl EnhancementEngine for Adapter {
    fn native_scale(&self) -> u32 {
        4
    }
    fn status(&self) -> EngineStatus {
        EngineStatus {
            id: "test".into(),
            name: "Test".into(),
            model: "fixture".into(),
            version: None,
            availability: Availability::Available,
            message: String::new(),
            location: None,
            install_dir: String::new(),
            source: None,
            scales: vec![2, 4, 8, 12],
            formats: vec!["PNG".into()],
            gpu_devices: vec![],
            devices: vec![],
            gpu_selection: "Auto".into(),
        }
    }
    fn process(&self, input: EngineInput<'_>, cancel: &Cancellation) -> Result<()> {
        let c = &self.0;
        let call = c.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let active = c.active.fetch_add(1, Ordering::SeqCst) + 1;
        c.maximum.fetch_max(active, Ordering::SeqCst);
        let _guard = Active(c.clone());
        while c.hold.load(Ordering::SeqCst) == call && !c.release.load(Ordering::SeqCst) {
            cancel.check()?;
            std::thread::sleep(Duration::from_millis(5));
        }
        cancel.check()?;
        if c.fail.load(Ordering::SeqCst) == call {
            return Err(err(ErrorCode::InferenceFailed));
        }
        let image = image::open(input.input).unwrap();
        image
            .resize_exact(image.width() * 4, image.height() * 4, FilterType::Nearest)
            .save(input.output)
            .unwrap();
        Ok(())
    }
}
fn fixture(root: &Path, name: &str) -> PathBuf {
    let p = root.join(name);
    DynamicImage::new_rgb8(16, 12).save(&p).unwrap();
    p
}
fn request(path: &Path, output: &Path, scale: u32) -> ProcessRequest {
    ProcessRequest {
        input_path: path.to_string_lossy().into(),
        output_dir: Some(output.to_string_lossy().into()),
        scale,
        format: OutputFormat::PNG,
        mode: "Photo".into(),
        gpu_index: None,
        gpu_id: None,
        tile_size: 32,
    }
}
fn service(root: &Path, c: &Arc<Control>) -> Arc<ProcessingService> {
    ProcessingService::new(Arc::new(Adapter(c.clone())), root.join("data"), root.into()).unwrap()
}
fn queue(p: Arc<ProcessingService>) -> Arc<BatchQueueService> {
    BatchQueueService::new(p, Arc::new(|_| {}), Arc::new(|_| {})).unwrap()
}
fn until(f: impl Fn() -> bool) {
    let start = Instant::now();
    while !f() {
        assert!(
            start.elapsed() < Duration::from_secs(15),
            "Timed out waiting for state"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn run(p: &Arc<ProcessingService>, r: ProcessRequest) -> Task {
    let (tx, rx) = mpsc::channel();
    p.start(
        r,
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
fn plans_and_target_safety_include_native_intermediates() {
    assert_eq!(
        pipeline::plan(8)
            .unwrap()
            .iter()
            .map(|p| p.scale)
            .collect::<Vec<_>>(),
        vec![4, 2]
    );
    assert_eq!(
        pipeline::plan(12)
            .unwrap()
            .iter()
            .map(|p| p.scale)
            .collect::<Vec<_>>(),
        vec![4, 3]
    );
    let limits = SafetyLimits::default();
    let t = limits.estimate(100, 75, 12, 4).unwrap();
    assert_eq!((t.width, t.height), (1200, 900));
    assert_eq!(t.intermediate_pixels, 100 * 75 * 256);
    assert_eq!(
        limits.estimate(3000, 2000, 12, 4).unwrap_err().code,
        ErrorCode::UnsafeTargetResolution
    );
    assert!(limits.estimate(u32::MAX, 2, 12, 4).is_err());
    assert!(limits.estimate(3000, 2000, 4, 4).is_ok());
}
#[test]
fn multipass_validates_real_dimensions_and_cleans() {
    for scale in [8, 12] {
        let d = tempfile::tempdir().unwrap();
        let path = fixture(d.path(), "source.png");
        let bytes = std::fs::read(&path).unwrap();
        let c = Arc::new(Control::default());
        let p = service(d.path(), &c);
        let task = run(&p, request(&path, d.path(), scale));
        assert_eq!(task.status, TaskStatus::Completed);
        let out = task.output.unwrap();
        assert_eq!((out.width, out.height), (16 * scale, 12 * scale));
        assert_eq!(c.calls.load(Ordering::SeqCst), 2);
        assert!(task
            .stages
            .iter()
            .all(|s| s.status == TaskStatus::Completed));
        assert_eq!(std::fs::read(path).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(&p.temp_root).unwrap().count(), 0);
    }
}
#[test]
fn unsafe_target_is_rejected_before_any_inference() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("large.png");
    DynamicImage::new_rgb8(1500, 1000).save(&path).unwrap();
    let c = Arc::new(Control::default());
    let p = service(d.path(), &c);
    assert_eq!(
        run(&p, request(&path, d.path(), 12)).error.unwrap().code,
        ErrorCode::UnsafeTargetResolution
    );
    assert_eq!(c.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn failures_in_either_pass_propagate_and_clean() {
    for pass in [1, 2] {
        let d = tempfile::tempdir().unwrap();
        let path = fixture(d.path(), "source.png");
        let c = Arc::new(Control::default());
        c.fail.store(pass, Ordering::SeqCst);
        let p = service(d.path(), &c);
        let task = run(&p, request(&path, d.path(), 8));
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error.unwrap().code, ErrorCode::PipelinePassFailed);
        assert!(task.output.is_none());
        assert_eq!(std::fs::read_dir(&p.temp_root).unwrap().count(), 0);
    }
}
#[test]
fn cancelling_either_pass_cleans_without_output() {
    for pass in [1, 2] {
        let d = tempfile::tempdir().unwrap();
        let path = fixture(d.path(), "source.png");
        let c = Arc::new(Control::default());
        c.hold.store(pass, Ordering::SeqCst);
        let p = service(d.path(), &c);
        let task = p
            .start(request(&path, d.path(), 12), Arc::new(|_| {}))
            .unwrap();
        until(|| c.calls.load(Ordering::SeqCst) == pass);
        p.cancel(&task.id).unwrap();
        until(|| p.active().unwrap().unwrap().status.terminal());
        assert_eq!(p.active().unwrap().unwrap().status, TaskStatus::Cancelled);
        assert_eq!(std::fs::read_dir(&p.temp_root).unwrap().count(), 0);
    }
}
#[test]
fn queue_validates_duplicates_folder_and_waiting_controls() {
    let d = tempfile::tempdir().unwrap();
    let a = fixture(d.path(), "a.png");
    let _b = fixture(d.path(), "b.jpg");
    std::fs::write(d.path().join("invalid.png"), b"bad").unwrap();
    let nested = d.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    fixture(&nested, "hidden.png");
    let c = Arc::new(Control::default());
    let q = queue(service(d.path(), &c));
    let report = q
        .enqueue_folder(d.path().to_string_lossy().into(), request(&a, d.path(), 2))
        .unwrap();
    assert_eq!(report.accepted, 2);
    assert_eq!(report.rejected.len(), 1);
    assert_eq!(
        q.enqueue(vec![a.to_string_lossy().into()], request(&a, d.path(), 4))
            .unwrap()
            .rejected[0]
            .error
            .code,
        ErrorCode::DuplicateQueueItem
    );
    let id = q.snapshot().unwrap().items[0].id.clone();
    q.control("cancel", Some(&id)).unwrap();
    assert_eq!(
        q.snapshot().unwrap().items[0].task.status,
        TaskStatus::Cancelled
    );
    q.control("remove", Some(&id)).unwrap();
    assert_eq!(q.snapshot().unwrap().items.len(), 1);
    assert_eq!(c.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn scheduler_serializes_pauses_and_inserts_history() {
    let d = tempfile::tempdir().unwrap();
    let c = Arc::new(Control::default());
    c.hold.store(1, Ordering::SeqCst);
    let p = service(d.path(), &c);
    let q = queue(p.clone());
    for (name, scale) in [("a.png", 2), ("b.png", 4), ("c.png", 8)] {
        let path = fixture(d.path(), name);
        q.enqueue(
            vec![path.to_string_lossy().into()],
            request(&path, d.path(), scale),
        )
        .unwrap();
    }
    q.start().unwrap();
    q.start().unwrap();
    until(|| c.calls.load(Ordering::SeqCst) == 1);
    q.control("pause", None).unwrap();
    c.release.store(true, Ordering::SeqCst);
    until(|| q.snapshot().unwrap().active_id.is_none());
    let state = q.snapshot().unwrap();
    assert!(!state.running);
    assert_eq!(
        state
            .items
            .iter()
            .filter(|i| i.task.status == TaskStatus::Queued)
            .count(),
        2
    );
    q.start().unwrap();
    until(|| {
        let s = q.snapshot().unwrap();
        !s.running && s.active_id.is_none()
    });
    assert!(q
        .snapshot()
        .unwrap()
        .items
        .iter()
        .all(|i| i.task.status == TaskStatus::Completed));
    assert_eq!(p.history.list().unwrap().len(), 3);
    assert_eq!(c.maximum.load(Ordering::SeqCst), 1);
}
#[test]
fn active_queue_cancel_and_failed_retry_make_clean_attempts() {
    let d = tempfile::tempdir().unwrap();
    let a = fixture(d.path(), "a.png");
    let c = Arc::new(Control::default());
    c.hold.store(1, Ordering::SeqCst);
    let p = service(d.path(), &c);
    let q = queue(p.clone());
    q.enqueue(vec![a.to_string_lossy().into()], request(&a, d.path(), 4))
        .unwrap();
    let id = q.snapshot().unwrap().items[0].id.clone();
    q.start().unwrap();
    until(|| c.calls.load(Ordering::SeqCst) == 1);
    q.control("cancel", Some(&id)).unwrap();
    until(|| !q.snapshot().unwrap().running);
    assert_eq!(
        q.snapshot().unwrap().items[0].task.status,
        TaskStatus::Cancelled
    );
    c.fail.store(2, Ordering::SeqCst);
    q.control("retry", Some(&id)).unwrap();
    let attempt = q.snapshot().unwrap().items[0].task.id.clone();
    q.start().unwrap();
    until(|| !q.snapshot().unwrap().running);
    assert_eq!(
        q.snapshot().unwrap().items[0].task.status,
        TaskStatus::Failed
    );
    q.control("retry", Some(&id)).unwrap();
    assert_ne!(attempt, q.snapshot().unwrap().items[0].task.id);
    q.start().unwrap();
    until(|| !q.snapshot().unwrap().running);
    let state = q.snapshot().unwrap();
    assert_eq!(state.items[0].task.status, TaskStatus::Completed);
    assert_eq!(state.items[0].retry_count, 2);
    assert_eq!(p.history.list().unwrap().len(), 3);
}
#[test]
fn restart_preserves_waiting_and_marks_interrupted_without_resuming() {
    let d = tempfile::tempdir().unwrap();
    let a = fixture(d.path(), "a.png");
    let c = Arc::new(Control::default());
    let p = service(d.path(), &c);
    let mut task = Task::queued(request(&a, d.path(), 8));
    task.transition(TaskStatus::Preparing, "Preparing").unwrap();
    task.transition(TaskStatus::Processing, "Pass 1").unwrap();
    p.history.save(&task).unwrap();
    let waiting = Task::queued(request(&a, d.path(), 2));
    let s = QueueSnapshot {
        items: vec![
            QueueItem {
                id: "active".into(),
                task,
                retry_count: 0,
                cancel_pending: false,
            },
            QueueItem {
                id: "waiting".into(),
                task: waiting,
                retry_count: 0,
                cancel_pending: false,
            },
        ],
        running: true,
        active_id: Some("active".into()),
        revision: 1,
        error: None,
    };
    p.history.save_queue(&s).unwrap();
    drop(p);
    let q = queue(service(d.path(), &c));
    let s = q.snapshot().unwrap();
    assert!(!s.running);
    assert!(s.active_id.is_none());
    assert_eq!(s.items[0].task.status, TaskStatus::Failed);
    assert_eq!(
        s.items[0].task.error.as_ref().unwrap().code,
        ErrorCode::InterruptedJob
    );
    assert_eq!(s.items[1].task.status, TaskStatus::Queued);
    assert_eq!(c.calls.load(Ordering::SeqCst), 0);
}
#[test]
#[ignore = "Requires ENHANCECE_TEST_ENGINE_DIR and a Vulkan GPU"]
fn real_multipass_and_batch() {
    use enhancece_lib::engines::realesrgan::RealEsrgan;
    let engine_dir =
        PathBuf::from(std::env::var_os("ENHANCECE_TEST_ENGINE_DIR").expect("engine required"));
    let temporary = tempfile::tempdir().unwrap();
    let root = std::env::var_os("ENHANCECE_PHASE3_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| temporary.path().into());
    std::fs::create_dir_all(&root).unwrap();
    let source = root.join("real source.png");
    assert!(!source.exists(), "Choose a fresh output directory");
    let image =
        image::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../public/assets/portrait.png"))
            .unwrap()
            .resize_exact(64, 48, FilterType::Lanczos3);
    image.save(&source).unwrap();
    let original = std::fs::read(&source).unwrap();
    let engine = RealEsrgan {
        locations: vec![(engine_dir.clone(), "real test".into())],
        install_dir: engine_dir,
    };
    let status = engine.status();
    println!(
        "DEVICES {}",
        serde_json::to_string(&status.devices).unwrap()
    );
    assert!(matches!(status.availability, Availability::Available));
    assert!(
        !status.devices.is_empty(),
        "NCNN device mapping must be verified"
    );
    let p = ProcessingService::new(Arc::new(engine), root.join("data"), root.clone()).unwrap();
    for scale in [2, 4, 8, 12] {
        let mut r = request(&source, &root, scale);
        r.tile_size = 128;
        r.gpu_id = Some(status.devices[0].id.clone());
        let task = run(&p, r);
        println!("SCALE {scale}: {}", serde_json::to_string(&task).unwrap());
        assert_eq!(task.status, TaskStatus::Completed);
        let out = task.output.unwrap();
        assert_eq!((out.width, out.height), (64 * scale, 48 * scale));
        assert_eq!(task.stages.len(), if scale > 4 { 2 } else { 1 });
    }
    let inputs = root.join("batch-inputs");
    std::fs::create_dir(&inputs).unwrap();
    for name in ["a.png", "b.jpg", "c.webp"] {
        image.save(inputs.join(name)).unwrap();
    }
    let q = queue(p.clone());
    let report = q
        .enqueue_folder(inputs.to_string_lossy().into(), request(&source, &root, 4))
        .unwrap();
    assert_eq!(report.accepted, 3);
    q.start().unwrap();
    let deadline = Instant::now();
    loop {
        let state = q.snapshot().unwrap();
        if !state.running && state.active_id.is_none() {
            assert!(
                state
                    .items
                    .iter()
                    .all(|i| i.task.status == TaskStatus::Completed),
                "{state:?}"
            );
            println!("BATCH {}", serde_json::to_string(&state).unwrap());
            break;
        }
        assert!(deadline.elapsed() < Duration::from_secs(180));
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(p.history.list().unwrap().len(), 7);
    let (tx, rx) = mpsc::channel();
    let task = p
        .start(
            request(&source, &root, 12),
            Arc::new(move |t| {
                let _ = tx.send(t);
            }),
        )
        .unwrap();
    loop {
        let t = rx.recv_timeout(Duration::from_secs(120)).unwrap();
        if t.stages
            .get(1)
            .is_some_and(|s| s.status == TaskStatus::Processing)
        {
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(700));
    p.cancel(&task.id).unwrap();
    loop {
        let t = rx.recv_timeout(Duration::from_secs(30)).unwrap();
        if t.status.terminal() {
            assert_eq!(t.status, TaskStatus::Cancelled);
            println!("REAL CANCEL {}", t.id);
            break;
        }
    }
    assert_eq!(std::fs::read(&source).unwrap(), original);
    assert_eq!(std::fs::read_dir(&p.temp_root).unwrap().count(), 0);
}
