Historical checkpoint report. Branding references were normalized to ClearCe; original testing used the former EnhanceCe name. Consult PHASE4_2.md for the current release.

# Phase 4 — Windows MVP productization

Branch: `phase/4-release-productization`, directly based on approved `3a1d367`. Product name, identifier `dev.usaince.enhancece`, publisher UsainCe.dev and existing version **0.1.0** are preserved. No post-MVP feature work was started.

## Product and engine strategy

The existing Tauri 2 / React Windows desktop application, Rust ProcessingService, Real-ESRGAN adapter, sequential queue, SQLite history and local settings store remain. No cloud processing, shell execution or new AI models were added.

**Managed local installation is the release strategy.** The previously tested user-provided engine distribution has executable/model files and VC runtime DLLs, but no license notices. Its version is not independently reported. We do not infer redistribution permission, and do not bundle these files into the installer. This is the concrete reason for using the alternative permitted by the Phase 4 request. It means this is **not a self-contained, engine-included installer**: first-time users must have their own trusted extracted runtime distribution. Distribution/licensing clearance and a reproducible approved engine package remain the condition for a one-download consumer release.

Models → **Install local engine** opens a native folder picker. The backend copies only the fixed executable, x4plus `.param`/`.bin` and available VC runtime DLLs into a staging directory under app-local `engines`, writes a SHA-256 manifest, validates paths/model header/integrity, then publishes by rename. The selected source is untouched. Existing installations are not overwritten. No network download or arbitrary command is constructed. Selecting a runtime is an explicit trust decision: hashes detect later changes, not publisher authenticity.

Production paths are resolved using Tauri app-local data and resource directories. `%LOCALAPPDATA%\dev.usaince.enhancece\engines\realesrgan` is the managed location. Resource fallback discovery remains available for a future licensed bundle. The development engine environment override is compiled only for debug builds. Release processing does not depend on the repository, Node, Rust, Python or a developer environment variable.

## Readiness and model management

Home shows startup readiness; Models provides Verify readiness, actual installation path/source, model, known version (or unknown), availability and actual required-file sizes. Specialized cards remain clearly planned.

Structured health checks cover verified engine files/model format, executable CLI compatibility, Vulkan detection, persisted GPU identity, writable temporary workspace, safe cache inspection and configured output directory. No inference or benchmark is automatically run. Status inspection initializes the runtime/device enumerator; this may take several seconds. Failed checks show their actionable messages rather than a success dashboard. Initial storage/session/configuration failures display a native setup-attention message.

## Settings and result UX

Existing native store persists Dark/Light/System, default 2/4/8/12 scale, PNG/JPG/WebP, output folder, GPU identity, tile size and privacy reminder preference. Native output-folder probing and stale-GPU validation happen **before** store writes. Invalid choices do not replace saved preferences. Empty output folder explicitly means the normal Pictures/ClearCe default. Changing defaults never mutates existing queue requests.

Light theme has usable panel/text colors, contrast-adjusted status colors and visible titlebar controls; Dark remains default. Preview-quality/updater controls stay disabled. Metadata retention is not implemented: orientation is applied, EXIF is stripped. Overwrite remains fixed keep-both; existing files receive suffixes. These limits are stated rather than presented as working toggles.

History retains recorded real paths, dimensions, timestamps, elapsed times, statuses and errors. Re-run settings restores a request to Home without auto-execution. Native open actions are restricted to recorded generated images and validate image content before opening. Deleting a record keeps image files.

Before/After uses local asset URLs, not image bytes in IPC. Results above 2048px are decoded on the Rust worker and reduced to a display preview; final output files are unchanged. The normal image safety limits still apply to decoding. Comparison range is keyboard accessible and zoom/fit remains available. History listing is bounded at 500; queue UI paginates 25 of at most 200. Preview generation is not a disk-quota cache: explicit clear and next-start cleanup reclaim previews.

## Recovery, cache and filesystem safety

An exclusive OS file lock on app-local `session.lock` lasts for the ProcessingService lifetime. A second instance cannot start processing or clean another instance's job data. On fresh startup, stale `temp/job-*` entries are inspected and cleaned before workers can start. Existing Phase 3 persisted interrupted-job recovery stays in place and logs recovery; retry creates a new execution attempt.

Cache is precisely: app-local job intermediates and regenerable `job-preview-*` display PNGs. It excludes all source images, final outputs, SQLite history, settings, engine/model files and unrelated temp-root entries. Cache size is the actual sum of regular-file bytes. Clear Cache holds the service's active-state lock and refuses while processing. It returns refreshed size.

The frontend cannot supply a cleanup path. Cleanup uses only the backend-owned temp root, rejects Windows reparse points/symlinks, canonicalizes descendants inside the root, validates a subtree before deletion and bounds traversal to 10000 entries. Unsafe/unreadable entries fail closed and log a high-level skip; they may require manual inspection. No recursive operation targets a source or output directory. This protects against accidental traversal, not a hostile same-user process racing filesystem changes; the engine is trusted native code, not an OS filesystem/network sandbox.

Logs are local and bounded by the existing logging plugin configuration. Startup, health state, interrupted recovery, stale cleanup and task lifecycle are recorded. No image content, EXIF payload or huge stdout dump is logged. Existing task IDs and concise diagnostics may identify local work; user-facing history/settings necessarily store local paths. Windows Job Object child-tree containment, timeouts, output validation, pixel/memory/disk limits and no-clobber publishing are retained.

## Packaging and notices

Primary bundle: per-user **NSIS x64**. Installer: `src-tauri/target/release/bundle/nsis/ClearCe_0.1.0_x64-setup.exe`. Release executable: `src-tauri/target/release/clearce.exe`. Tested installation location: `%LOCALAPPDATA%\Programs\EnhanceCe` (historical installation name).

The bundle includes application executable, established icon and `THIRD_PARTY_NOTICES.md`; it intentionally excludes the supplied engine with unverified redistribution terms, models and developer artifacts. Notices reproduce verified license files from local dependency distributions, including conditional/build dependencies. They do not invent a runtime/model license. WebView2 uses Tauri's Microsoft bootstrapper if absent; that prerequisite can require network access. A compatible Vulkan driver remains a hardware prerequisite.

No signing certificate was configured. Authenticode inspection reported **NotSigned**. Windows/SmartScreen may warn; future signing belongs in the normal Tauri Windows signing configuration with a legitimate certificate. No signing workaround or updater service is used.

## Validation record — Windows, 2026-09-14

- Frontend production build: passed.
- Frontend tests: **9 passed** (including native preference validation-before-save and rejected settings preservation).
- Rust tests: **31 passed**, two real-GPU tests ignored in default runs. Coverage includes managed import integrity/preservation, missing files, native output-directory validation, session lock, stale cleanup, source/output/settings/history preservation, readiness and stale GPU, and retained Phase 2/3 processing/queue/recovery/Windows containment coverage.
- `cargo check`: passed.
- Debug Windows executable build: passed.
- Optimized release build and NSIS packaging: passed.
- Actual NSIS installation to normal user directory: exit 0; installed application launched.
- Clean-profile first run: missing engine reported; native `install_local_engine` command imported the existing trusted runtime without script/CLI setup; subsequent health ready. The import command was exercised through the actual installed WebView2/Tauri bridge; the OS folder-picker selection itself was not automated in this smoke.
- Installed real inference: 64×48 input produced 128×96 (2x), 256×192 (4x), 512×384 (8x), 768×576 (12x). Source SHA-256 unchanged.
- Installed batch: three PNG/JPG/WebP inputs completed through the UI Start Processing control. Seven completed records persisted across close/relaunch together with the three stopped queue entries and a saved WebP default.
- Native relaunch removed an intentionally left `job-stale-smoke` directory; comparison and zoom loaded real output.
- Larger installed inference: 1536×1024 source → real 3072×2048 2x output, displayed as 2048×1365 preview. Measured cache was 8407080 bytes; Clear Cache preserved source and full output. Native Open folder, Open output and Re-run settings actions returned successfully.
- QA artifacts are ignored `.qa/phase4-*` files. Existing user app data was preserved separately during clean-profile validation.

Reproduce builds with `npm ci`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`, and `npm run tauri build -- --bundles nsis`. GPU smoke is opt-in as documented in Phase 3. Installer validation is distinct from a development executable test.


### Final installer checks and verification limits

The final NSIS installer was uninstalled and reinstalled successfully (both exit 0). Eight existing completed jobs survived; a final real 4x WebP job then completed. Dark, Light and System were saved through the installed UI. Final result preview metadata correctly reports 2048 × 1365 for the larger output. Original pre-QA local and roaming app data were restored; the installed release was launched without the remote-debugging flag. Isolated smoke profiles were retained separately, not merged into user history.

Installer SHA-256: `D9354F7C9B5F652CCB6C83F16030B3189FF1473B056644C8ABA668552671DADF`.
Installed EXE SHA-256: `BE7CDDFC2E7DEB2D683FE7E15266344D0D0ADFC168B870D672EDE8A7B4B30164`.
The installed and target release executable differ in exactly three bytes: Tauri replaces `__TAURI_BUNDLE_TYPE_VAR_UNK()` with `__TAURI_BUNDLE_TYPE_VAR_NSS()` for NSIS. Byte comparison found no other differences; raw hash equality is therefore not the correct packaging check here.

An additional native junction-escape test was **not performed**: automatic approval review rejected its test setup/cleanup command with `blocked by policy`, before execution. No junction was created. Reparse-point rejection was reviewed in source; existing safe cleanup/preservation tests passed. This is a stated validation limit, not a passed junction test. The fresh-profile engine import used the native command bridge, so OS folder-picker selection is likewise not claimed as tested in Phase 4. Full clean-machine/driver certification and hostile same-user filesystem race testing are outside this validation.

## Known MVP limits and post-MVP backlog

The main distribution limitation is the external trusted engine prerequisite; this installer must not be advertised as engine-included/offline-complete. Obtain verified redistribution notices and an approved reproducible engine package before changing that policy. Signing, native specialist models, automatic model downloads/updates, metadata retention, overwrite mode, configurable preview quality, quota-based preview caching, broader accessibility certification, multi-GPU concurrency and other platforms are deferred. Large multipass inputs remain restricted by the approved intermediate/memory ceilings. No Phase 5/v1.1 work was begun.

See [feature truth table](MVP_FEATURE_STATUS.md), [changed files](FILES.txt), and [third-party notices](../THIRD_PARTY_NOTICES.md).
