# ClearCe Phase 4.5 — managed engine and automatic model recommendation

Baseline: clean `phase/4.3-installer-trust` at `49b409854706e3cd1c32000f3437f8dff5dea757`. Work branch: `phase/4.5-managed-engine-auto-models`. ClearCe remains a native Tauri Windows x64 application; no server, cloud inference, Vercel deployment, telemetry or updater was added.

## Delivered behavior

The Models screen now always shows the current engine source, model selection, Vulkan status, deterministic GPU tier, recommendation reason and next action. Auto is the default and persists with the selected model. Manual mode remains available through the existing native directory picker and uses a separate managed directory.

Rust owns the typed catalog, GPU-tier rules, source allowlist, pinned package metadata and install transaction. The managed flow downloads the official Real-ESRGAN 0.2.5.0 Windows archive, verifies its pinned SHA-256, checks the complete archive for unsafe paths, extracts only seven expected files, builds a local integrity manifest, performs stable-path health verification and activates only after success. Previous managed and manual installations are preserved on failure.

Available models are `realesrgan-x4plus` and `realesrgan-x4plus-anime`. `realesr-general-x4v3` and GFPGAN are explicitly deferred because the approved runtime does not provide a currently integrated package. Portrait and screenshot modes do not claim specialist processing.

English and Turkish catalogs cover recommendations, status, GPU tiers, install progress, failure recovery, manual/managed explanations and the engine guide. Byte progress is shown only when the response reports a total; all other progress is stage based.

## Validation record

- Frontend: 24 tests passed, including settings persistence and EN/TR catalog parity/error coverage.
- Rust default suite: 38 passed, 4 ignored opt-in hardware/permission tests. Managed tests cover deterministic tiers, anime routing, manual fallback, source allowlisting, SHA mismatch, missing files, zip-slip and manual preservation.
- Official managed package: real network download, pinned SHA-256, safe extraction, stable-path health check, x4plus/anime discovery and one real GPU inference passed in a temporary profile.
- Real existing engine: 2x, 4x, 8x, 12x, three-file batch, cancellation, PNG/JPG/WebP and Vulkan device discovery passed on the detected NVIDIA GeForce RTX 4090. Test outputs used temporary directories; the user profile engine was read only.
- Frontend production build and Cargo check passed.

Debug/release build results and final commit are recorded in the Phase 4.5 handoff. Installer behavior and app-data preservation remain unchanged. The AI package is downloaded only inside the installed app after explicit user action; it is not bundled into the NSIS installer.

## Limits

The managed source is one pinned upstream GitHub release. There is no dynamic web search, mirror fallback, automatic background download, engine updater, portrait restoration or general-x4v3 integration. Upstream native code is hash pinned and contained as a child process, but it is not a security sandbox. The Windows release remains unsigned until a legitimate signing identity is supplied.
