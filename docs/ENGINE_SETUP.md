# ClearCe local engine setup

ClearCe requires a trusted Windows Real-ESRGAN NCNN Vulkan runtime and general `realesrgan-x4plus` weights. They are not bundled. Review upstream instructions and licensing before obtaining or redistributing a runtime; ClearCe does not certify a download source.

In **Models / Modeller**, choose **Select Local AI Engine / Yerel AI Motorunu Seç**. Select the extracted folder containing `realesrgan-ncnn-vulkan.exe`, `models/realesrgan-x4plus.param` and `models/realesrgan-x4plus.bin`.

Available required runtime DLLs are copied with the fixed files. Import checks containment, integrity, model format, launchability and Vulkan. Readiness also checks temporary/output storage and GPU selection. A valid runtime without a compatible GPU is not ready for processing.

| State | Next step |
| --- | --- |
| Missing | Select an extracted trusted runtime; app installation does not install AI weights. |
| Invalid | Check all required files and select again. Invalid imports preserve the working copy. |
| No Vulkan device | Check/install your GPU manufacturer's compatible driver. |
| Stale GPU | Select Auto or a detected device in Settings. |
| Unwritable output | Choose an existing writable directory. |
| Unsafe target / insufficient resources | Use a smaller image/lower scale and free disk space. |

2x/4x/8x/12x and PNG/JPG/WebP use the general pipeline. Files stay local. Existing outputs are preserved; metadata retention is unavailable.

## Existing installations

Technical identifier: `dev.usaince.enhancece`. Engine: `%LOCALAPPDATA%\dev.usaince.enhancece\engines\realesrgan`. Settings: `%APPDATA%\dev.usaince.enhancece`, key `enhancece.settings.v1`. The legacy Pictures/EnhanceCe default output directory remains intentional. Custom saved paths are unchanged.

Both app names share the exclusive session lock. Close EnhanceCe before opening ClearCe. Installer names differ, so an old shortcut may remain until the previous app is uninstalled. Do not remove application data when retiring it.

## Developer checks

The UI is the recommended setup path. `scripts/install-engine.ps1` remains a compatibility helper that refuses to replace an existing installation; the UI adds launch/Vulkan verification and recovery.

```powershell
$env:ENHANCECE_TEST_ENGINE_DIR = Join-Path $env:LOCALAPPDATA 'dev.usaince.enhancece/engines/realesrgan'
cargo test --manifest-path src-tauri/Cargo.toml --test processing real_gpu_inference -- --ignored --nocapture
cargo test --manifest-path src-tauri/Cargo.toml real_symlink_escape_is_rejected -- --ignored --nocapture
```

Only opt in with trusted executable files.
