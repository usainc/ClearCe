mod commands;
pub mod core;
pub mod engines;
pub mod services;
pub mod storage;
use services::batch_queue::BatchQueueService;
use services::processing_service::ProcessingService;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .max_file_size(1_000_000)
                .build(),
        )
        .setup(|app| {
            let data = app.path().app_local_data_dir()?;
            let installed = data.join("engines/realesrgan");
            let bundled = app.path().resource_dir()?.join("engines/realesrgan");
            #[allow(unused_mut)] // Mutable only for the debug-only engine override below.
            let mut locations = vec![
                (installed.clone(), "installed".into()),
                (bundled, "resource".into()),
            ];
            #[cfg(debug_assertions)]
            if let Some(path) = std::env::var_os("ENHANCECE_DEV_ENGINE_DIR") {
                locations.push((path.into(), "development override".into()));
            }
            let engine = Arc::new(engines::realesrgan::RealEsrgan {
                locations,
                install_dir: installed,
            });
            let output = app.path().picture_dir()?.join("EnhanceCe");
            let processing = match ProcessingService::new(engine, data, output) {
                Ok(service)=>service,
                Err(error)=>{
                    log::error!("event=startup_failed code={:?}",error.code);
                    let turkish=commands::system_locale().to_lowercase().starts_with("tr");
                    app.dialog().message(if turkish {"ClearCe çalışma alanını açamadı. Diğer ClearCe pencerelerini kapatın; depolama izinlerini ve processing-limits.json ayarlarını kontrol edin."} else {"ClearCe could not open its workspace. Close other ClearCe windows and check storage permissions or processing-limits.json."}).title(if turkish {"ClearCe kurulumu kontrol edilmeli"} else {"ClearCe setup requires attention"}).kind(tauri_plugin_dialog::MessageDialogKind::Error).blocking_show();
                    return Err(error.into());
                }
            };
            let handle = app.handle().clone();
            let queue = BatchQueueService::new(
                processing.clone(),
                Arc::new(move |snapshot| {
                    let _ = handle.emit_to("main", "batch-queue", snapshot);
                }),
                commands::task_sink(app.handle().clone()),
            )?;
            app.manage(queue);
            app.manage(processing);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::inspect_image,
            commands::engine_status,
            commands::start_enhancement,
            commands::cancel_enhancement,
            commands::active_enhancement,
            commands::processing_history,
            commands::delete_processing_history,
            commands::result_image,
            commands::open_result,
            commands::queue_snapshot,
            commands::queue_add,
            commands::queue_add_folder,
            commands::queue_control,
            commands::queue_apply_settings,
            commands::engine_health,
            commands::system_locale,
            commands::validate_preferences,
            commands::processing_cache,
            commands::install_local_engine
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.state::<Arc<BatchQueueService>>().shutdown();
                let service = window.state::<Arc<ProcessingService>>().inner().clone();
                if service
                    .active()
                    .ok()
                    .flatten()
                    .is_some_and(|t| !t.status.terminal())
                {
                    api.prevent_close();
                    service.shutdown();
                    let window = window.clone();
                    // Keep the runtime alive until cancellation has reaped the child
                    // and cleaned its temporary files. Never block the UI thread.
                    std::thread::spawn(move || {
                        while service
                            .active()
                            .ok()
                            .flatten()
                            .is_some_and(|t| !t.status.terminal())
                        {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        let _ = window.destroy();
                    });
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to run ClearCe");
}
