# ClearCe Phase 4.6 — final installer repack and release candidate

Baseline: clean `phase/4.5-managed-engine-auto-models` at `35fa03c1430cbc954042213ecaa03ca79ef87e0b`. Release branch: `phase/4.6-final-release-candidate`. Version remains `0.1.0` because ClearCe has not been publicly released.

## Final package

- Windows x64 NSIS: `ClearCe_0.1.0_x64-setup.exe` — 13,092,243 bytes.
- Standalone release executable: `clearce.exe` — 25,988,096 bytes.
- Signing state: unsigned. No self-signed certificate was used.
- Real-ESRGAN and model files are not bundled. The installed app downloads the pinned official package only after explicit user action.

## Release integrity

- `clearce.exe`: `6caa030b1820c633335f4ec166eacfe465ec6c59e1e2341ae143a1c0bcb81769`
- `ClearCe_0.1.0_x64-setup.exe`: `32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116`

`SHA256SUMS.txt`, `release-manifest.json`, and independent `Get-FileHash` calculations matched both final files. The manifest records Windows/x64, EN/TR, `signed:false`, `engineBundled:false`, and `managedEngineDownload:true`.

Microsoft Defender scanned both exact final artifacts with engine `1.1.26080.3` and definitions `1.459.248.0`. The result was `scan completed; no matching detections reported`.

## Installed-app validation

The final NSIS package was installed over the existing per-user installation. The installed executable reported ClearCe 0.1.0 and the actual installed app exposed the Phase 4.5 Models experience. Auto/Recommended, the managed download action, EN/TR engine help, RTX 4090/Ultra recommendation, pinned source policy, and manual-engine fallback were visible in the installed WebView.

A separate fresh profile was tested with the actual installed application. With no engine present, direct Vulkan detection produced an RTX 4090/Ultra recommendation and enabled **Download Recommended**. The official 43.4 MiB package downloaded, matched the pinned SHA-256, extracted only the approved files, passed health checks, exposed both x4plus models, and activated as a managed engine.

Real installed-app processing passed 2x, 4x, 8x, and 12x. PNG, JPG, WebP, cancellation, and a three-image batch passed. Relaunching with HTTP/HTTPS directed to an unavailable local proxy still detected the installed managed engine and completed a real 2x enhancement, confirming that inference has no GitHub dependency after installation.

English and Turkish both rendered in the installed app. An English selection survived restart. The fresh managed profile survived same-version reinstall with all 11 stable tracked files unchanged and no forced redownload.

The original user profile was restored after the fresh-profile test. Its settings and manual-engine hashes matched the pre-test snapshot, and history retained the same logical table counts (`queue_state: 1`, `tasks: 3`). The final installed app reopened that profile with the existing legacy engine ready.

## Automated validation

- Frontend: 24 tests passed.
- Rust: 39 tests passed; 4 opt-in hardware/permission tests ignored by the default suite.
- Frontend production build, Cargo check, and `cargo fmt --check` passed.
- Tauri release build and NSIS packaging passed.
- Managed failure coverage includes network/HTTP errors, source allowlisting, SHA mismatch, invalid/missing archive content, zip-slip rejection, and preservation of the working engine.
- `npm audit --omit=dev`: zero reported vulnerabilities.
- Release guard, secret-pattern scan, forbidden artifact scan, manifest comparison, and independent hashes passed.

## Limits

The release candidate is unsigned and may trigger SmartScreen. Windows x64, WebView2, and a compatible Vulkan driver are required. GFPGAN, specialist portrait/text/restoration models, automatic updates, cloud processing, and video remain deferred. No artifact was published by this phase.
