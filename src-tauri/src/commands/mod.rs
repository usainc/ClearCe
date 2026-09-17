use crate::{
    core::processing::{
        errors::{err, ErrorCode, Result},
        pipeline,
        task::{ImageInfo, ProcessRequest, Task},
    },
    services::processing_service::ProcessingService,
};
use std::{path::Path, sync::Arc};
use tauri::{Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
type Service = Arc<ProcessingService>;
type Queue = Arc<crate::services::batch_queue::BatchQueueService>;
#[tauri::command]
pub fn system_locale() -> String {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetUserDefaultLocaleName(buffer: *mut u16, size: i32) -> i32;
        }
        let mut buffer = [0u16; 85];
        let count = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), 85) };
        if count > 1 {
            return String::from_utf16_lossy(&buffer[..count as usize - 1]);
        }
    }
    "en-US".into()
}
#[tauri::command]
pub async fn engine_health(
    service: State<'_, Service>,
    output_dir: Option<String>,
    gpu_id: Option<String>,
    engine_mode: Option<String>,
) -> Result<crate::services::maintenance::Health> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.health(
            output_dir.as_deref(),
            gpu_id.as_deref(),
            engine_mode.as_deref(),
        )
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))
}
#[tauri::command]
pub async fn validate_preferences(
    service: State<'_, Service>,
    output_dir: Option<String>,
    gpu_id: Option<String>,
) -> Result<()> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::maintenance::output(output_dir.as_deref(), &service.default_output)?;
        if let Some(id) = gpu_id {
            if !service.status().devices.iter().any(|d| d.id == id) {
                return Err(err(ErrorCode::InvalidGpuSelection));
            }
        }
        Ok(())
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn processing_cache(service: State<'_, Service>, clear: bool) -> Result<u64> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.cache(clear)?;
        service.cache(false)
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn install_local_engine(
    app: tauri::AppHandle,
    service: State<'_, Service>,
    source: String,
) -> Result<()> {
    let service = service.inner().clone();
    let parent = app
        .path()
        .app_local_data_dir()
        .map_err(|_| err(ErrorCode::IoError))?
        .join("engines")
        .join("manual");
    tauri::async_runtime::spawn_blocking(move || {
        service.install_engine(Path::new(&source), &parent)
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn managed_engine_state(
    app: tauri::AppHandle,
    service: State<'_, Service>,
    category: String,
    engine_mode: String,
    model_id: String,
) -> Result<crate::services::managed_engine::ManagedState> {
    let data = app
        .path()
        .app_local_data_dir()
        .map_err(|_| err(ErrorCode::IoError))?;
    let mut status = service.status_mode(&engine_mode);
    // Recommendation must work before an engine exists. NCNN-indexed devices
    // remain authoritative for processing, while this direct Vulkan query is
    // used only for the hardware summary and install recommendation.
    if status.devices.is_empty() {
        status.devices = crate::engines::realesrgan::gpu::recommendation_devices();
    }
    Ok(crate::services::managed_engine::state(
        &data,
        &status,
        &category,
        &engine_mode,
        &model_id,
    ))
}
#[tauri::command]
pub async fn install_managed_engine(
    app: tauri::AppHandle,
    service: State<'_, Service>,
    source_url: String,
) -> Result<()> {
    let data = app
        .path()
        .app_local_data_dir()
        .map_err(|_| err(ErrorCode::IoError))?;
    let service = service.inner().clone();
    let events = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.install_managed_engine(&data, &source_url, move |progress| {
            let _ = events.emit_to("main", "managed-engine-progress", progress);
        })
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
pub fn task_sink(app: tauri::AppHandle) -> crate::services::processing_service::TaskSink {
    Arc::new(move |task| {
        if let Some(output) = &task.output {
            let _ = app.asset_protocol_scope().allow_file(&output.path);
        }
        let _ = app.emit_to("main", "enhancement-task", task);
    })
}
#[tauri::command]
pub async fn queue_snapshot(
    queue: State<'_, Queue>,
) -> Result<crate::services::batch_queue::QueueSnapshot> {
    queue.snapshot()
}
#[tauri::command]
pub async fn queue_add(
    queue: State<'_, Queue>,
    paths: Vec<String>,
    settings: ProcessRequest,
) -> Result<crate::services::batch_queue::EnqueueReport> {
    let queue = queue.inner().clone();
    tauri::async_runtime::spawn_blocking(move || queue.enqueue(paths, settings))
        .await
        .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn queue_add_folder(
    queue: State<'_, Queue>,
    path: String,
    settings: ProcessRequest,
) -> Result<crate::services::batch_queue::EnqueueReport> {
    let queue = queue.inner().clone();
    tauri::async_runtime::spawn_blocking(move || queue.enqueue_folder(path, settings))
        .await
        .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn queue_control(
    queue: State<'_, Queue>,
    action: String,
    id: Option<String>,
) -> Result<()> {
    let queue = queue.inner().clone();
    tauri::async_runtime::spawn_blocking(move || queue.control(&action, id.as_deref()))
        .await
        .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn queue_apply_settings(queue: State<'_, Queue>, settings: ProcessRequest) -> Result<()> {
    let queue = queue.inner().clone();
    tauri::async_runtime::spawn_blocking(move || queue.apply_settings(settings))
        .await
        .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn inspect_image(app: tauri::AppHandle, path: String) -> Result<ImageInfo> {
    tauri::async_runtime::spawn_blocking(move || {
        let info = pipeline::inspect(Path::new(&path))?;
        app.asset_protocol_scope()
            .allow_file(&info.path)
            .map_err(|_| err(ErrorCode::IoError))?;
        Ok(info)
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn engine_status(
    service: State<'_, Service>,
) -> Result<crate::core::processing::engine::EngineStatus> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.status())
        .await
        .map_err(|_| err(ErrorCode::IoError))
}
#[tauri::command]
pub async fn start_enhancement(
    app: tauri::AppHandle,
    service: State<'_, Service>,
    request: ProcessRequest,
) -> Result<Task> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        service.start(
            request,
            Arc::new(move |task| {
                if let Some(output) = &task.output {
                    let _ = app.asset_protocol_scope().allow_file(&output.path);
                }
                let _ = app.emit_to("main", "enhancement-task", task);
            }),
        )
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub fn cancel_enhancement(service: State<'_, Service>, id: String) -> Result<()> {
    service.cancel(&id)
}
#[tauri::command]
pub fn active_enhancement(service: State<'_, Service>) -> Result<Option<Task>> {
    service.active()
}
#[tauri::command]
pub async fn processing_history(service: State<'_, Service>) -> Result<Vec<Task>> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.history.list())
        .await
        .map_err(|_| err(ErrorCode::StorageError))?
}
#[tauri::command]
pub async fn delete_processing_history(service: State<'_, Service>, id: String) -> Result<()> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.delete_history(&id))
        .await
        .map_err(|_| err(ErrorCode::StorageError))?
}
#[tauri::command]
pub async fn result_image(
    app: tauri::AppHandle,
    service: State<'_, Service>,
    id: String,
) -> Result<ImageInfo> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let info = service
            .history
            .get(&id)?
            .output
            .ok_or_else(|| err(ErrorCode::OutputMissing))?;
        let path = Path::new(&info.path)
            .canonicalize()
            .map_err(|_| err(ErrorCode::OutputMissing))?;
        let img = pipeline::decode(&path, pipeline::MAX_PIXELS * 16)?;
        if img.width() != info.width || img.height() != info.height {
            return Err(err(ErrorCode::DimensionMismatch));
        }
        // The exported file stays untouched. Bound the displayed result to 2048px.
        let preview = if img.width() > 2048 || img.height() > 2048 {
            let dir = service
                .temp_root
                .join(format!("job-preview-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).map_err(|_| err(ErrorCode::IoError))?;
            let preview = dir.join("preview.png");
            img.thumbnail(2048, 2048)
                .save(&preview)
                .map_err(|_| err(ErrorCode::IoError))?;
            preview
        } else {
            path
        };
        app.asset_protocol_scope()
            .allow_file(&preview)
            .map_err(|_| err(ErrorCode::IoError))?;
        let (width, height) =
            image::image_dimensions(&preview).map_err(|_| err(ErrorCode::InvalidImage))?;
        let bytes = std::fs::metadata(&preview)
            .map_err(|_| err(ErrorCode::IoError))?
            .len();
        Ok(ImageInfo {
            path: preview.to_string_lossy().into_owned(),
            width,
            height,
            bytes,
            ..info
        })
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
#[tauri::command]
pub async fn open_result(
    app: tauri::AppHandle,
    service: State<'_, Service>,
    id: String,
    folder: bool,
) -> Result<()> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let info = service
            .history
            .get(&id)?
            .output
            .ok_or_else(|| err(ErrorCode::OutputMissing))?;
        let path = Path::new(&info.path)
            .canonicalize()
            .map_err(|_| err(ErrorCode::OutputMissing))?;
        if !path.is_file() {
            return Err(err(ErrorCode::OutputMissing));
        }
        // Only recorded generated images can be opened, never arbitrary user-provided commands/URLs.
        pipeline::decode(&path, pipeline::MAX_PIXELS * 16)?;
        let target = if folder {
            path.parent().ok_or_else(|| err(ErrorCode::IoError))?
        } else {
            &path
        };
        app.opener()
            .open_path(target.to_string_lossy(), None::<&str>)
            .map_err(|_| err(ErrorCode::IoError))
    })
    .await
    .map_err(|_| err(ErrorCode::IoError))?
}
