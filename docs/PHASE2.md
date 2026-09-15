Historical checkpoint report. Branding references were normalized to ClearCe; original testing used the former EnhanceCe name. Consult PHASE4_2.md for the current release.

# Phase 2 — Local AI processing

Checkpoint: 2026-09-14. Repository: `<repository>`. Branch: `phase/2-local-ai-engine`. Approved Phase 1 baseline was preserved as commit `6a25e55a4329ee318e7c7a3eb36a20353de429e6`. Phase 3 has not started.

## Architecture

`core/processing/engine.rs` defines `EnhancementEngine`: identity/status, supported scales/formats, native scale, processing, and cancellation. The service depends on this trait. `engines/realesrgan` implements discovery, Vulkan capability detection, CLI argument construction, bounded output collection, and Windows process containment. Other adapters can implement the same contract without changing the UI or service.

`ProcessingService` owns one active task and a cooperative cancellation token. A background worker performs engine checks, image preparation, inference, encoding and publication. Long Tauri command work uses `spawn_blocking`; the window thread and React stay responsive. Tasks move through queued → preparing → processing → completed, or failed/cancelled. Terminal states cannot restart. Errors have stable codes and clean user messages. State contains input/output paths, actual dimensions/size, timestamps, engine/model identity and optional warnings.

`storage/History` stores actual jobs in SQLite under application data, with WAL journaling. Queued tasks are persisted before work starts and terminal results are saved before events are emitted. On reopening, interrupted tasks are marked failed. History displays the latest 500 records and supports filtering, searching, result opening, comparison and record removal; removing a record does not delete an image. Phase 1 demo history is no longer shown in History.

## Engine installation and trust

Discovery order:

1. Installed: application local data `/engines/realesrgan`.
2. Bundled: application resources `/engines/realesrgan`.
3. Debug builds only: optional `ENHANCECE_DEV_ENGINE_DIR`.

There is no machine-specific engine path in application source. `scripts/install-engine.ps1` provisions a user-supplied trusted package into `%LOCALAPPDATA%\dev.usaince.enhancece\engines\realesrgan`. It copies the executable, x4plus parameter/weight files and available VC runtime DLLs, then records SHA-256 hashes in `manifest.json`. Existing installations are not overwritten. A failed installation retains its staging directory for inspection.

Discovery validates required files, canonical containment, manifest hashes, parameter header, and the executable's advertised CLI options. An invalid higher-priority installation fails closed. The manifest detects corruption/accidental changes; it is not a publisher signature or a sandbox for an untrusted executable. Only trusted packages should be installed. Runtime/model binaries are deliberately not committed or redistributed; license/redistribution review and signed distribution remain future packaging work. The supplied executable does not report a semantic version; UI says so instead of inventing one.

The existing user proof-of-concept engine was copied to the managed directory for this machine. Source package and original user images were not changed. F: and Windows system-folder permissions were not touched.

## Real inference and output

The model is `realesrgan-x4plus`, using fixed 128-pixel tiles, one load/process/save thread, PNG intermediate output, and automatic NCNN GPU selection. Arguments use `Command::args`/`OsString`, with no shell. Backend requests also support a bounded explicit GPU index for future UI integration.

The x4plus network is intrinsically 4x: every supported request runs actual 4x inference. 2x then downsamples that generated image with Lanczos3. Both intermediate 4x dimensions and final requested dimensions must match before publication. This follows the model selection behavior in the [upstream source](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan/blob/master/src/main.cpp); supported executable options were also checked against the installed `-h` output and [upstream CLI documentation](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan).

A real test exposed an upstream Windows path limitation: the CLI treats verbatim `\\?\` model paths as relative. The adapter now passes the fixed `models` directory relative to the verified installation working directory. Canonical discovery and file-integrity checks remain in place. Spaces and Turkish characters in input/output paths were exercised successfully.

Input detection uses signatures and decoding, not extensions alone. Supported formats are JPG/JPEG, PNG and WebP. Files are capped at 100 MB and 6 million input pixels, with bounded decoder allocation. EXIF orientation is applied before inference and dimension validation. All engine input is a controlled PNG copy in a unique app-data job directory. Encoding produces PNG, JPG or lossless WebP; original metadata is not copied. JPEG discards alpha, and ICC/HDR/color-profile preservation is not implemented.

The pipeline probes the output folder before starting inference. The default Pictures/ClearCe folder is created as needed; custom folders must already exist. Encoded output is staged in a unique temporary file in that folder, decoded again for validation, then atomically published without clobbering. Names use a sanitized source stem and `_enhanced_2x`/`_enhanced_4x`; collisions get numeric suffixes. No source writes, wildcard deletion or overwrite mode exist.

## Cancellation, cleanup and logging

Windows children are created suspended and assigned to a Job Object with kill-on-close before their main thread resumes. If containment fails, the suspended child is terminated and reaped. Cancellation/timeout kills the job, waits for the direct child, drains bounded stdout/stderr and cleans temporary files. A native close during processing first requests cancellation and keeps the runtime alive asynchronously until cleanup completes. The Job Object also prevents orphaned owned descendants if the app process dies.

Cancellation and final publication share a lock. Cancellation accepted before publication prevents an output from being published; a request after completion leaves the completed file intact. Normal success, failure and cancellation remove the unique job directory and staged output. Abrupt OS termination/power loss can leave temporary files; automatic crash scavenging is deferred.

Progress is honest phase text plus an indeterminate indicator. No estimated percentages are fabricated; only completed tasks have 100%. Engine help checks have a 5-second deadline and inference has a 20-minute deadline. Cancellation can wait for a decoder/encoder or readiness check already in progress to return.

Structured log events cover creation, discovery, process start/exit, publication, completion/failure/cancellation and cleanup. Exit codes and typed error codes are logged. Logs do not contain image content or raw engine output dumps. Captured process output is bounded to 16 KB per stream.

## Native commands and frontend

Typed commands: `inspect_image`, `engine_status`, `start_enhancement`, `cancel_enhancement`, `active_enhancement`, `processing_history`, `delete_processing_history`, `result_image`, `open_result`. The `enhancement-task` event carries a complete task snapshot to the main window. A small active-task poll recovers missed deliveries; reconciliation prevents a late older task or queued response from undoing completion.

Native dialog and native drop events supply local paths. `inspect_image` validates and grants asset-protocol access to that exact file. Result commands validate recorded outputs and grant only their exact paths. The asset protocol starts with an empty scope; CSP permits the local asset origin, with no global filesystem allowlist or remote image source. The native capability set remains core events/window controls, settings Store and the open dialog. The opener plugin is called only from Rust for recorded generated images or their parent folders; no generic frontend shell/process/file API was added.

Home starts real tasks, shows phases/cancel/errors/results, and compares original on the left with the actual generated image on the right. Preview can inspect completed comparisons; independent patch generation remains disabled. History uses actual persisted jobs. Models and the sidebar show real engine readiness and Vulkan device names. 8x/12x, non-Photo modes, adjustable quality and standalone face restoration are visibly unavailable. Batch remains a session queue with orchestration disabled; Home is the real single-image processing path.

## GPU behavior

The Vulkan loader is dynamically loaded and physical non-CPU device names are enumerated. This machine reported **NVIDIA GeForce RTX 4090** and **AMD Radeon(TM) Graphics**. NCNN selects its own default GPU when `-g` is omitted; enumeration order is not presented as a guaranteed NCNN index mapping. No VRAM/utilization/driver version is invented. Readiness is not a promise that all image sizes or driver combinations will work; inference errors remain typed and visible.

## Validation and evidence

| Check | Result |
| --- | --- |
| Frontend build | PASS: TypeScript + Vite |
| Frontend tests | PASS: 5 tests, including late-task/terminal event reconciliation |
| Rust check | PASS |
| Normal Rust tests | PASS: 18 tests; GPU-only test ignored by default |
| Windows debug executable | PASS: Tauri build with embedded UI, no installer |
| Real GPU inference | PASS: opt-in test, actual engine/model; PNG/JPG/WebP inputs and outputs |
| 2x | PASS: 192 × 128 → 384 × 256 |
| 4x | PASS: 192 × 128 → 768 × 512 |
| Source safety | PASS: byte equality before/after; output collisions preserve earlier files |
| Windows process containment | PASS: cancellation kills a fixture parent and its spawned descendant |
| Native UI | PASS: actual executable/WebView2 and Windows file picker, not a browser mock |

Tests cover explicit task transitions, invalid formats/scales/modes, naming, Unicode/spaced arguments, signature validation, EXIF orientation, missing/incomplete engines, invalid dimensions, failed launch/exit/inference, unwritable output folders, busy-task rejection, cancellation/cleanup, no-clobber publication, and SQLite persistence/deletion behavior. Test adapters are confined to test code and are never used by the application.

Actual generated smoke outputs are retained locally under `.qa/real-inference-final` (ignored by Git). The first real test failed on the Windows model path issue described above; after fixing the adapter, inference passed. A later test run was blocked by the running app's executable lock; the app was closed through its own native close control, then tests passed.

Native QA used the bundled Playwright runtime connected to the real WebView2 debugging endpoint on loopback, plus scoped Win32 dialog interaction. The Browser plugin was not available; no external browser rendering was substituted for native verification. The debugging endpoint was enabled only through the QA process environment and is not configured in shipped code.

Native flow verified: app launch → Windows file picker selects a spaced/Turkish filename → 2x/4x Enhance buttons → actual Rust completion events → local asset images decode at original/output dimensions → comparison → Open output opens the generated file in Windows Photos. Cancellation was exercised during preparation and inference. Closing the app during inference exited cleanly with no engine process or temp job left. Navigation through all six screens showed no page errors, blank screens, framework overlays, broken images or horizontal page overflow at the tested native viewport (1078 × 934). Home, History and Models screenshots were visually inspected. The approved palette, sidebar and central/right-panel structure remain; real status controls replace placeholders.

Native screenshots and temporary QA scripts are in `%TEMP%\enhancece-phase2-qa`. No viewport-wide/mobile or exhaustive driver/memory matrix is claimed. Hardware performance figures above are observations from small smoke inputs, not benchmark promises.

## Limits and Phase 3 boundary

- Single active task; no batch orchestration, pause/resume, multi-job persistence/queue worker or automatic retries.
- 2x/4x Photo only. No 8x/12x multi-pass, portrait/face, anime, restoration, diffusion or video engines.
- No preview patch inference, model download UI, signed engine distribution, installer/signing or updater infrastructure.
- Metadata copying, ICC/HDR handling, configurable tiling/VRAM strategy, crash-temp scavenging and large-image memory optimization remain future work. PNG/WebP alpha behavior depends on the engine; JPG drops alpha.
- No cloud processing, accounts, telemetry, payments or authentication.

Phase 3 work requires a separate request. This checkpoint stops at Phase 2.
