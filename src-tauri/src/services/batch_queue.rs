use super::processing_service::{ProcessingService, TaskSink};
use crate::core::processing::{
    errors::{err, ErrorCode, ProcessingError, Result},
    pipeline,
    task::{ProcessRequest, Task, TaskStatus},
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItem {
    pub id: String,
    pub task: Task,
    pub retry_count: u32,
    #[serde(default)]
    pub cancel_pending: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub items: Vec<QueueItem>,
    pub running: bool,
    pub active_id: Option<String>,
    pub revision: u64,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueReport {
    pub accepted: usize,
    pub rejected: Vec<Rejected>,
    pub snapshot: QueueSnapshot,
}
#[derive(Debug, Serialize)]
pub struct Rejected {
    pub name: String,
    pub error: ProcessingError,
}
pub type QueueSink = Arc<dyn Fn(QueueSnapshot) + Send + Sync>;
pub struct BatchQueueService {
    pub processing: Arc<ProcessingService>,
    state: Mutex<QueueSnapshot>,
    worker: AtomicBool,
    closing: AtomicBool,
    sink: QueueSink,
    task_sink: TaskSink,
}
impl BatchQueueService {
    pub fn new(
        processing: Arc<ProcessingService>,
        sink: QueueSink,
        task_sink: TaskSink,
    ) -> Result<Arc<Self>> {
        let mut state: QueueSnapshot = processing.history.read_queue()?;
        for item in &mut state.items {
            if item.task.status != TaskStatus::Queued && !item.task.status.terminal()
                || state.active_id.as_ref() == Some(&item.id)
            {
                if let Ok(saved) = processing.history.get(&item.task.id) {
                    if saved.status.terminal() {
                        item.task = saved;
                        item.cancel_pending = false;
                        continue;
                    }
                }
                item.task
                    .transition(TaskStatus::Failed, "Previous session interrupted")?;
                item.task.error = Some(err(ErrorCode::InterruptedJob));
                processing.history.save(&item.task)?;
            }
            item.cancel_pending = false;
        }
        state.running = false;
        state.active_id = None;
        state.revision += 1;
        processing.history.save_queue(&state)?;
        Ok(Arc::new(Self {
            processing,
            state: Mutex::new(state),
            worker: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            sink,
            task_sink,
        }))
    }
    pub fn snapshot(&self) -> Result<QueueSnapshot> {
        Ok(self
            .state
            .lock()
            .map_err(|_| err(ErrorCode::StorageError))?
            .clone())
    }
    fn edit<T>(&self, f: impl FnOnce(&mut QueueSnapshot) -> Result<T>) -> Result<T> {
        let (result, snapshot) = {
            let mut guard = self
                .state
                .lock()
                .map_err(|_| err(ErrorCode::StorageError))?;
            let mut next = guard.clone();
            let result = f(&mut next)?;
            next.revision += 1;
            self.processing.history.save_queue(&next)?;
            *guard = next.clone();
            (result, next)
        };
        (self.sink)(snapshot);
        Ok(result)
    }
    pub fn enqueue(&self, paths: Vec<String>, settings: ProcessRequest) -> Result<EnqueueReport> {
        settings.validate()?;
        let mut accepted = 0;
        let mut rejected = vec![];
        if paths.len() > 201 {
            rejected.push(Rejected {
                name: format!("{} additional files exceed import limit", paths.len() - 201),
                error: err(ErrorCode::QueueLimitExceeded),
            });
        }
        for path in paths.into_iter().take(201) {
            let result = (|| {
                let input = pipeline::inspect(Path::new(&path))?;
                self.processing.limits.estimate(
                    input.width,
                    input.height,
                    settings.scale,
                    self.processing.engine.native_scale(),
                )?;
                self.edit(|state| {
                    if state.items.len() >= 200 {
                        return Err(err(ErrorCode::QueueLimitExceeded));
                    }
                    // Windows canonical paths are compared without case. All retained
                    // items participate; remove an old item to deliberately re-add it.
                    if state.items.iter().any(|i| {
                        i.task.request.input_path.to_lowercase() == input.path.to_lowercase()
                    }) {
                        return Err(err(ErrorCode::DuplicateQueueItem));
                    }
                    let mut request = settings.clone();
                    request.input_path = input.path.clone();
                    let mut task = Task::queued(request);
                    task.input = Some(input);
                    state.items.push(QueueItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        task,
                        retry_count: 0,
                        cancel_pending: false,
                    });
                    Ok(())
                })
            })();
            match result {
                Ok(()) => accepted += 1,
                Err(error) => rejected.push(Rejected {
                    name: Path::new(&path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    error,
                }),
            }
        }
        Ok(EnqueueReport {
            accepted,
            rejected,
            snapshot: self.snapshot()?,
        })
    }
    pub fn enqueue_folder(&self, path: String, settings: ProcessRequest) -> Result<EnqueueReport> {
        let folder = Path::new(&path)
            .canonicalize()
            .map_err(|_| err(ErrorCode::InvalidImage))?;
        let mut paths = vec![];
        let mut scan_truncated = false;
        for (index, entry) in std::fs::read_dir(folder)
            .map_err(|_| err(ErrorCode::InvalidImage))?
            .take(1001)
            .enumerate()
        {
            if index == 1000 {
                scan_truncated = true;
                break;
            }
            let entry = entry.map_err(|_| err(ErrorCode::IoError))?;
            if entry
                .file_type()
                .map_err(|_| err(ErrorCode::IoError))?
                .is_file()
                && entry.path().extension().is_some_and(|e| {
                    ["jpg", "jpeg", "png", "webp"]
                        .contains(&e.to_string_lossy().to_lowercase().as_str())
                })
            {
                paths.push(entry.path().to_string_lossy().into_owned());
            }
        }
        paths.sort();
        let truncated = scan_truncated || paths.len() > 200;
        paths.truncate(200);
        let mut report = self.enqueue(paths, settings)?;
        if truncated {
            report.rejected.push(Rejected {
                name: "Folder import limit".into(),
                error: err(ErrorCode::QueueLimitExceeded),
            });
        }
        Ok(report)
    }
    pub fn apply_settings(&self, settings: ProcessRequest) -> Result<()> {
        settings.validate()?;
        self.edit(|state| {
            for item in &mut state.items {
                if item.task.status == TaskStatus::Queued
                    && state.active_id.as_ref() != Some(&item.id)
                {
                    if let Some(input) = &item.task.input {
                        self.processing.limits.estimate(
                            input.width,
                            input.height,
                            settings.scale,
                            self.processing.engine.native_scale(),
                        )?;
                    }
                    let path = item.task.request.input_path.clone();
                    item.task.request = settings.clone();
                    item.task.request.input_path = path;
                }
            }
            Ok(())
        })
    }
    pub fn control(self: &Arc<Self>, action: &str, id: Option<&str>) -> Result<()> {
        match action {
            "start" => self.start(),
            "pause" => self.edit(|s| {
                s.running = false;
                Ok(())
            }),
            "clear" => self.edit(|s| {
                s.items.retain(|i| s.active_id.as_ref() == Some(&i.id));
                Ok(())
            }),
            "remove" => self.edit(|s| {
                let id = id.ok_or_else(|| err(ErrorCode::QueueItemNotFound))?;
                if s.active_id.as_deref() == Some(id) {
                    return Err(err(ErrorCode::Busy));
                }
                let len = s.items.len();
                s.items.retain(|i| i.id != id);
                if len == s.items.len() {
                    return Err(err(ErrorCode::QueueItemNotFound));
                }
                Ok(())
            }),
            "retry" => self.edit(|s| {
                if s.active_id.as_deref() == id {
                    return Err(err(ErrorCode::Busy));
                }
                let item = s
                    .items
                    .iter_mut()
                    .find(|i| Some(i.id.as_str()) == id)
                    .ok_or_else(|| err(ErrorCode::QueueItemNotFound))?;
                if !matches!(item.task.status, TaskStatus::Failed | TaskStatus::Cancelled) {
                    return Err(err(ErrorCode::Busy));
                }
                let mut next = Task::queued(item.task.request.clone());
                next.input = item.task.input.clone();
                item.task = next;
                item.retry_count += 1;
                item.cancel_pending = false;
                Ok(())
            }),
            "cancel" => {
                let execution = self.edit(|s| {
                    let item = s
                        .items
                        .iter_mut()
                        .find(|i| Some(i.id.as_str()) == id)
                        .ok_or_else(|| err(ErrorCode::QueueItemNotFound))?;
                    if item.task.status.terminal() {
                        return Ok(None);
                    }
                    if s.active_id.as_ref() == Some(&item.id) {
                        item.cancel_pending = true;
                        Ok(Some(item.task.id.clone()))
                    } else {
                        item.task
                            .transition(TaskStatus::Cancelled, "Cancelled before execution")?;
                        Ok(None)
                    }
                })?;
                if let Some(id) = execution {
                    let _ = self.processing.cancel(&id);
                }
                Ok(())
            }
            _ => Err(err(ErrorCode::QueueItemNotFound)),
        }
    }
    pub fn start(self: &Arc<Self>) -> Result<()> {
        if self.closing.load(Ordering::SeqCst) {
            return Err(err(ErrorCode::Busy));
        }
        self.edit(|s| {
            s.running = true;
            s.error = None;
            Ok(())
        })?;
        if self
            .worker
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let service = self.clone();
            if std::thread::Builder::new()
                .name("enhancece-queue".into())
                .spawn(move || service.schedule())
                .is_err()
            {
                self.worker.store(false, Ordering::SeqCst);
                self.edit(|s| {
                    s.running = false;
                    Ok(())
                })?;
                return Err(err(ErrorCode::ProcessLaunchFailed));
            }
        }
        Ok(())
    }
    fn schedule(self: Arc<Self>) {
        loop {
            let next = {
                let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
                if !guard.running
                    || self.closing.load(Ordering::SeqCst)
                    || !guard
                        .items
                        .iter()
                        .any(|i| i.task.status == TaskStatus::Queued)
                {
                    guard.running = false;
                    guard.active_id = None;
                    guard.revision += 1;
                    self.worker.store(false, Ordering::SeqCst);
                    let snapshot = guard.clone();
                    let _ = self.processing.history.save_queue(&snapshot);
                    drop(guard);
                    (self.sink)(snapshot);
                    return;
                }
                guard
                    .items
                    .iter()
                    .find(|i| i.task.status == TaskStatus::Queued)
                    .unwrap()
                    .clone()
            };
            if self
                .processing
                .active()
                .ok()
                .flatten()
                .is_some_and(|t| !t.status.terminal())
            {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            let claimed = self.edit(|s| {
                if !s.running {
                    return Ok(None);
                }
                if let Some(item) = s
                    .items
                    .iter()
                    .find(|i| i.id == next.id && i.task.status == TaskStatus::Queued)
                {
                    s.active_id = Some(item.id.clone());
                    Ok(Some(item.task.clone()))
                } else {
                    Ok(None)
                }
            });
            let task = match claimed {
                Ok(Some(task)) => task,
                Ok(None) => continue,
                Err(_) => {
                    self.stop_on_error();
                    self.worker.store(false, Ordering::SeqCst);
                    return;
                }
            };
            let (tx, rx) = mpsc::channel();
            let queue = self.clone();
            let item_id = next.id.clone();
            let outcome = self.processing.start_task(
                task,
                Arc::new(move |task| {
                    let terminal = task.status.terminal();
                    let execution = task.id.clone();
                    let cancel = queue.edit(|s| {
                        let item = s
                            .items
                            .iter_mut()
                            .find(|i| i.id == item_id)
                            .ok_or_else(|| err(ErrorCode::QueueItemNotFound))?;
                        item.task = task.clone();
                        Ok(item.cancel_pending)
                    });
                    if matches!(cancel, Ok(true)) && !terminal {
                        let _ = queue.processing.cancel(&execution);
                    }
                    if cancel.is_err() {
                        queue.stop_on_error();
                        if !terminal {
                            let _ = queue.processing.cancel(&execution);
                        }
                    }
                    (queue.task_sink)(task);
                    if terminal {
                        let _ = tx.send(());
                    }
                }),
            );
            match outcome {
                Ok(_) => {
                    let _ = rx.recv();
                }
                Err(e) if e.code == ErrorCode::Busy => {
                    let _ = self.edit(|s| {
                        s.active_id = None;
                        Ok(())
                    });
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(error) => {
                    let _ = self.edit(|s| {
                        if let Some(item) = s.items.iter_mut().find(|i| i.id == next.id) {
                            item.task
                                .transition(TaskStatus::Failed, "Could not start processing")?;
                            item.task.error = Some(error);
                        }
                        Ok(())
                    });
                }
            }
            if self
                .edit(|s| {
                    s.active_id = None;
                    Ok(())
                })
                .is_err()
            {
                self.stop_on_error();
                self.worker.store(false, Ordering::SeqCst);
                return;
            }
        }
    }
    fn stop_on_error(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.running = false;
            s.error = Some("Queue persistence failed. Processing stopped.".into());
            s.revision += 1;
            let snapshot = s.clone();
            drop(s);
            (self.sink)(snapshot);
        }
    }
    pub fn shutdown(&self) {
        self.closing.store(true, Ordering::SeqCst);
        let _ = self.edit(|s| {
            s.running = false;
            Ok(())
        });
        self.processing.shutdown();
    }
}
