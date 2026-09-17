use crate::{
    core::processing::{
        engine::{Cancellation, EngineStatus, EnhancementEngine},
        errors::{err, ErrorCode, Result},
        pipeline,
        safety::SafetyLimits,
        task::{ProcessRequest, Task, TaskStatus},
    },
    storage::History,
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
pub type TaskSink = Arc<dyn Fn(Task) + Send + Sync>;
struct Active {
    task: Task,
    cancel: Cancellation,
}
pub struct ProcessingService {
    _session: std::fs::File,
    stopping: AtomicBool,
    pub engine: Arc<dyn EnhancementEngine>,
    pub history: History,
    active: Mutex<Option<Active>>,
    pub temp_root: PathBuf,
    pub default_output: PathBuf,
    pub limits: SafetyLimits,
}
impl ProcessingService {
    pub fn new(
        engine: Arc<dyn EnhancementEngine>,
        data: PathBuf,
        default_output: PathBuf,
    ) -> Result<Arc<Self>> {
        std::fs::create_dir_all(&data).map_err(|_| err(ErrorCode::StorageError))?;
        let session = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(data.join("session.lock"))
            .map_err(|_| err(ErrorCode::StorageError))?;
        session.try_lock().map_err(|_| err(ErrorCode::Busy))?;
        log::info!("event=application_startup");
        if super::maintenance::cache(&data.join("temp"), true).is_err() {
            log::warn!("event=cleanup_skipped_unsafe_or_unreadable_path");
        }
        Ok(Arc::new(Self {
            _session: session,
            stopping: AtomicBool::new(false),
            limits: SafetyLimits::load(&data.join("processing-limits.json"))?,
            engine,
            history: History::open(&data.join("history.sqlite3"))?,
            active: Mutex::new(None),
            temp_root: data.join("temp"),
            default_output,
        }))
    }
    pub fn status(&self) -> EngineStatus {
        self.engine.status()
    }
    pub fn status_mode(&self, mode: &str) -> EngineStatus {
        self.engine.status_mode(mode)
    }
    pub fn cache(&self, clear: bool) -> Result<u64> {
        let active = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
        if clear && active.as_ref().is_some_and(|a| !a.task.status.terminal()) {
            return Err(err(ErrorCode::Busy));
        }
        super::maintenance::cache(&self.temp_root, clear)
    }
    pub fn health(
        &self,
        folder: Option<&str>,
        gpu: Option<&str>,
        engine_mode: Option<&str>,
    ) -> super::maintenance::Health {
        use super::maintenance::{Check, Health};
        let status = self.status_mode(engine_mode.unwrap_or("Auto"));
        let available = matches!(
            status.availability,
            crate::core::processing::engine::Availability::Available
        );
        let out = super::maintenance::output(folder, &self.default_output);
        let temp = std::fs::create_dir_all(&self.temp_root).is_ok()
            && tempfile::tempfile_in(&self.temp_root).is_ok();
        let selected = gpu.is_none() || status.devices.iter().any(|d| Some(d.id.as_str()) == gpu);
        let mut files = vec![];
        if let Some(root) = &status.location {
            for name in crate::engines::realesrgan::discovery::REQUIRED {
                if let Ok(m) = std::fs::metadata(std::path::Path::new(root).join(name)) {
                    files.push((name.into(), m.len()));
                }
            }
        }
        let cache = self.cache(false);
        let checks = vec![
            Check {code:"executable".into(),name:"Executable verification".into(),ok:available || matches!(status.availability,crate::core::processing::engine::Availability::Unsupported),message:String::new()},
            Check {code:"models".into(),name:"Model verification".into(),ok:available || matches!(status.availability,crate::core::processing::engine::Availability::Unsupported),message:String::new()},
            Check {code:"launch".into(),name:"Launch verification".into(),ok:available || matches!(status.availability,crate::core::processing::engine::Availability::Unsupported),message:String::new()},
            Check {code:"vulkan".into(),name:"Vulkan verification".into(),ok:available,message:String::new()},
            Check {code: "cache".into(), name:"Cache safety".into(),ok:cache.is_ok(),message:if cache.is_ok(){"Managed temporary workspace verified"}else{"Temporary workspace could not be safely inspected. Close the app and check its temp directory permissions."}.into()},
            Check {
                code: "engine".into(), name: "Engine, model and Vulkan".into(),
                ok: available,
                message: status.message,
            },
            Check {
                code: "temp".into(), name: "Temporary workspace".into(),
                ok: temp,
                message: if temp {
                    "Writable"
                } else {
                    "Cannot write temporary data; check disk permissions"
                }
                .into(),
            },
            Check {
                code: "output".into(), name: "Output directory".into(),
                ok: out.is_ok(),
                message: out
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|e| e.message),
            },
            Check {
                code: "gpu".into(), name: "GPU selection".into(),
                ok: selected,
                message: if selected {
                    "Available"
                } else {
                    "Saved device unavailable. Choose Auto or a detected GPU in Settings."
                }
                .into(),
            },
        ];
        let state = if checks.iter().all(|c| c.ok) {
            "ready"
        } else {
            "requires attention"
        }
        .to_string();
        log::info!("event=engine_health state={state}");
        Health {
            engine_state: status.availability,
            state,
            checks,
            files,
            cache_bytes: cache.ok(),
        }
    }
    pub fn active(&self) -> Result<Option<Task>> {
        Ok(self
            .active
            .lock()
            .map_err(|_| err(ErrorCode::IoError))?
            .as_ref()
            .map(|v| v.task.clone()))
    }
    pub fn install_engine(&self, source: &std::path::Path, parent: &std::path::Path) -> Result<()> {
        let active = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
        if active.as_ref().is_some_and(|a| !a.task.status.terminal()) {
            return Err(err(ErrorCode::Busy));
        }
        super::installation::install_verified(source, parent)
    }
    pub fn install_managed_engine(
        &self,
        data: &std::path::Path,
        source_url: &str,
        progress: impl FnMut(super::managed_engine::InstallProgress),
    ) -> Result<()> {
        let active = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
        if active.as_ref().is_some_and(|a| !a.task.status.terminal()) {
            return Err(err(ErrorCode::Busy));
        }
        super::managed_engine::install(data, source_url, progress)
    }
    pub fn start(self: &Arc<Self>, request: ProcessRequest, sink: TaskSink) -> Result<Task> {
        self.start_task(Task::queued(request), sink)
    }
    pub fn start_task(self: &Arc<Self>, mut task: Task, sink: TaskSink) -> Result<Task> {
        let request = &task.request;
        request.validate()?;
        let mut active = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
        if self.stopping.load(Ordering::SeqCst) {
            return Err(err(ErrorCode::Busy));
        }
        if active.as_ref().is_some_and(|a| !a.task.status.terminal()) {
            return Err(err(ErrorCode::Busy));
        }
        if task.status != TaskStatus::Queued {
            return Err(err(ErrorCode::Busy));
        }
        task.started_at = Some(crate::core::processing::task::now_ms());
        self.history.save(&task)?;
        let token = Cancellation::default();
        *active = Some(Active {
            task: task.clone(),
            cancel: token.clone(),
        });
        drop(active);
        log::info!("event=task_created task_id={}", task.id);
        sink(task.clone());
        let service = self.clone();
        let id = task.id.clone();
        std::thread::Builder::new()
            .name("enhancece-processing".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service.run(&token, &sink)
                }));
                if result.is_err() {
                    service.finish_error(err(ErrorCode::InferenceFailed), &sink);
                    log::error!("event=processing_failed task_id={id} reason=worker_panic");
                }
            })
            .map_err(|_| {
                if let Ok(mut active) = self.active.lock() {
                    if let Some(a) = active.as_mut() {
                        let _ = a
                            .task
                            .transition(TaskStatus::Failed, "Could not start worker");
                        a.task.error = Some(err(ErrorCode::ProcessLaunchFailed));
                        let _ = self.history.save(&a.task);
                    }
                }
                err(ErrorCode::ProcessLaunchFailed)
            })?;
        Ok(task)
    }
    fn update(&self, sink: &TaskSink, f: impl FnOnce(&mut Task)) {
        let snapshot = {
            let mut guard = self.active.lock().unwrap_or_else(|e| e.into_inner());
            let Some(active) = guard.as_mut() else { return };
            f(&mut active.task);
            active.task.clone()
        };
        sink(snapshot);
    }
    fn run(&self, token: &Cancellation, sink: &TaskSink) {
        self.update(sink, |task| {
            let _ = task.transition(TaskStatus::Preparing, "Checking local engine");
        });
        let result = (|| {
            token.check()?;
            let mut request = self
                .active()?
                .ok_or_else(|| err(ErrorCode::TaskNotFound))?
                .request;
            let status = self.status_mode(&request.engine_mode);
            use crate::core::processing::engine::Availability;
            match status.availability {
                Availability::Available => {}
                Availability::Missing => return Err(err(ErrorCode::EngineNotFound)),
                Availability::Unsupported => return Err(err(ErrorCode::GpuUnavailable)),
                _ => return Err(err(ErrorCode::EngineInvalid)),
            }
            token.check()?;
            let selected = if let Some(id) = &request.gpu_id {
                let device = status
                    .devices
                    .iter()
                    .find(|d| &d.id == id)
                    .ok_or_else(|| err(ErrorCode::InvalidGpuSelection))?;
                request.gpu_index = Some(device.index);
                device.name.clone()
            } else if let Some(index) = request.gpu_index {
                status
                    .devices
                    .iter()
                    .find(|d| d.index == index)
                    .map(|d| d.name.clone())
                    .ok_or_else(|| err(ErrorCode::InvalidGpuSelection))?
            } else {
                "Auto (NCNN default)".into()
            };
            self.update(sink, |task| {
                task.request = request.clone();
                task.selected_gpu = selected.clone();
                task.record(&format!("Selected GPU: {selected}"));
                task.engine_id = status.id;
                task.engine_version = status.version;
                task.model = if request.model_id == "auto" {
                    if request.mode == "Anime / Illustration"
                        && status
                            .models
                            .iter()
                            .any(|m| m == crate::engines::realesrgan::discovery::MODEL_ANIME)
                    {
                        crate::engines::realesrgan::discovery::MODEL_ANIME.into()
                    } else {
                        crate::engines::realesrgan::discovery::MODEL.into()
                    }
                } else {
                    request.model_id.clone()
                };
            });
            let input = pipeline::inspect(std::path::Path::new(&request.input_path))?;
            let target = self.limits.estimate(
                input.width,
                input.height,
                request.scale,
                self.engine.native_scale(),
            )?;
            self.update(sink, |task| {
                task.target = Some(target);
                task.stages = pipeline::plan(request.scale).unwrap_or_default();
                task.input = Some(input);
                let _ = task.transition(TaskStatus::Processing, "Preparing image");
            });
            log::info!("event=processing_started");
            let output = pipeline::process_with_limits(
                self.engine.as_ref(),
                &request,
                &self.temp_root,
                &self.default_output,
                token,
                &self.limits,
                &|event| {
                    self.update(sink, |task| {
                        task.record(&event.phase);
                        if let Some(i) = event.pass {
                            if let Some(stage) = task.stages.get_mut(i) {
                                stage.status = if event.completed {
                                    TaskStatus::Completed
                                } else {
                                    TaskStatus::Processing
                                };
                            }
                        }
                    })
                },
            )?;
            // Cancellation and publication share a lock. Cancellation accepted before publication wins.
            let snapshot = {
                let mut active = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
                token.check()?;
                let active = active
                    .as_mut()
                    .ok_or_else(|| err(ErrorCode::TaskNotFound))?;
                let output = pipeline::publish(output, &request)?;
                log::info!("event=output_written task_id={}", active.task.id);
                active.task.output = Some(output);
                active.task.transition(TaskStatus::Completed, "Completed")?;
                if let Err(e) = self.history.save(&active.task) {
                    active.task.warning = Some(e.message);
                    log::error!("event=history_save_failed task_id={}", active.task.id);
                }
                active.task.clone()
            };
            log::info!("event=processing_completed task_id={}", snapshot.id);
            sink(snapshot);
            Ok(())
        })();
        if let Err(e) = result {
            self.finish_error(e, sink);
        }
    }
    fn finish_error(
        &self,
        error: crate::core::processing::errors::ProcessingError,
        sink: &TaskSink,
    ) {
        self.update(sink, |task| {
            let cancelled = error.code == ErrorCode::Cancelled;
            let _ = task.transition(
                if cancelled {
                    TaskStatus::Cancelled
                } else {
                    TaskStatus::Failed
                },
                if cancelled { "Cancelled" } else { "Failed" },
            );
            log::info!(
                "event={} task_id={} code={:?}",
                if cancelled {
                    "processing_cancelled"
                } else {
                    "processing_failed"
                },
                task.id,
                error.code
            );
            task.error = Some(error);
            for stage in &mut task.stages {
                if stage.status == TaskStatus::Processing {
                    stage.status = task.status;
                }
            }
            if self.history.save(task).is_err() {
                task.warning = Some("Job history could not be saved.".into());
            }
        });
    }
    pub fn cancel(&self, id: &str) -> Result<()> {
        let active = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
        let active = active
            .as_ref()
            .filter(|a| a.task.id == id)
            .ok_or_else(|| err(ErrorCode::TaskNotFound))?;
        if !active.task.status.terminal() {
            self.engine.cancel(&active.cancel);
        }
        Ok(())
    }
    pub fn shutdown(&self) {
        if let Ok(active) = self.active.lock() {
            self.stopping.store(true, Ordering::SeqCst);
            if let Some(a) = active.as_ref() {
                a.cancel.cancel();
            }
        }
    }
    pub fn delete_history(&self, id: &str) -> Result<()> {
        let guard = self.active.lock().map_err(|_| err(ErrorCode::IoError))?;
        if guard
            .as_ref()
            .is_some_and(|a| a.task.id == id && !a.task.status.terminal())
        {
            return Err(err(ErrorCode::Busy));
        }
        self.history.delete(id)
    }
}
