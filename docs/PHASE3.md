Historical checkpoint report. Branding references were normalized to ClearCe; original testing used the former EnhanceCe name. Consult PHASE4_2.md for the current release.

# Phase 3 checkpoint

Branch: `phase/3-multipass-batch`, based on approved Phase 2 commit `2cd9d898693a577ebd7e660245346570836dbf95`.

## Processing architecture

The existing Rust ProcessingService, verified Real-ESRGAN adapter, Windows Job Object containment, SQLite history and React desktop shell remain. A pipeline plan exposes real pass states and timestamped activity. Inference has indeterminate progress; completed-job counts are used only for overall batch progress.

| Output | Actual operations | Verified 64 × 48 source output |
| --- | --- | --- |
| 2x | Native x4plus 4x, Lanczos3 reduction to 2x | 128 × 96 |
| 4x | Native x4plus 4x | 256 × 192 |
| 8x | Native 4x, another native 4x, reduce raw 16x to 8x (effective second pass 2x) | 512 × 384 |
| 12x | Native 4x, another native 4x, reduce raw 16x to 12x (effective second pass 3x) | 768 × 576 |

Both multipass stages run genuine local AI inference using `realesrgan-x4plus`. The final resize is disclosed; it does not replace the second inference. Every intermediate and final file is decoded and checked against expected dimensions before completion. Pass failures retain their pass number; cancellation terminates the contained child process tree.

### Scale-3 compatibility

The installed executable accepted a direct `-s 3` probe and returned exit code 0. This does **not** certify a native 3x x4plus network. The upstream [CLI implementation](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan/blob/master/src/main.cpp) accepts scales 2/3/4 and selects scale-specific model files for animevideov3, whereas x4plus uses the same weights. This implementation therefore consistently invokes verified native `-s 4` and obtains the effective 3x second stage by reducing its checked output. No unverified native-3x claim is made.

## Safety, tiling and files

- Existing input limits: 100 MB, 6 million pixels, decoded JPG/PNG/WebP with orientation applied.
- Final hard ceiling: 96 million pixels. Intermediate hard ceiling: 128 million pixels. Checked arithmetic rejects dimension overflow.
- Estimated working image memory ceiling: 2 GiB. Estimate includes the largest native intermediate, target and source. This is a conservative configured budget, not a measurement or reservation of currently free RAM/VRAM.
- Multipass creates a raw 16x image, so its 256-times pixel count can reject inputs well below the general 6 MP limit. These limits intentionally reject unsafe large 8x/12x requests before inference.
- Optional app-data `processing-limits.json` can tighten `maxOutputPixels`, `maxIntermediatePixels`, `maxWorkingBytes` and configure `diskMarginBytes`. Invalid configuration fails closed. Defaults are 96000000, 128000000, 2147483648 and 134217728 respectively.
- Available disk space is checked on temporary and output volumes before execution and before each pass. Conservative estimates include intermediates and final output plus a 128 MiB margin. The checks are not a disk reservation; later write failures remain typed failures.
- Tile choices: Auto (`-t 0`, engine-managed), 32, 64, 128 (default), 256. The chosen value is sent to every inference. Smaller tiles reduce GPU working demand but do not eliminate full-image CPU memory needs. No automatic OOM retry or invented GPU metrics.
- Each attempt owns a unique temporary directory. RAII cleanup handles success, failure and cancellation. An abrupt OS/process termination may leave an orphan temporary directory; automatic orphan cleanup is not implemented. Lossless PNG intermediates are private to the attempt.
- Output is staged and validated before publication. Originals and existing outputs are never overwritten; collisions receive numeric suffixes. The visible overwrite policy is fixed to **Keep both** in this phase.

## Batch scheduler and persistence

`BatchQueueService` owns queue state in Rust and stores versioned snapshots in SQLite. The frontend subscribes to `batch-queue` events and issues typed commands; it does not run the scheduler.

Multi-file import validates each image separately and reports rejected files. Folder import is nonrecursive, scans at most 1000 entries and admits supported regular image files. The queue retains at most 200 items; truncation is reported. Canonical Windows paths are compared without case across all retained items. Remove a previous item to deliberately add that file again. The UI paginates 25 rows.

One scheduler feeds the existing globally exclusive ProcessingService: maximum one GPU job, including Home processing. Bulk settings apply atomically to waiting, unclaimed jobs. Each attempt records its own settings, device, stages, timestamps, errors and real output. Completed jobs enter normal history and can load their actual result for comparison.

Pause stops dispatch **after the current job completes**. Cancel can target a waiting item or the active attempt. Remove/clear preserve the active item. Retry only accepts failed/cancelled inactive items, creates a fresh execution ID and increments attempt count; previous executed attempts remain in history. Failures do not silently count as successful outputs.

Restart retains waiting jobs without automatically starting them. Nonterminal interrupted attempts become failed with `InterruptedJob`; an already persisted terminal history record is recovered when available. Close prevents new dispatch, cancels active processing and uses the existing native shutdown path.

## GPU selection

Settings persist Auto or a device identity assembled from Vulkan vendor/device properties and pipeline-cache UUID. Actual NCNN indices are discovered from the installed engine's device enumeration, matched by unique device name to Vulkan identities, and resolved again before execution. Stale selections fail with `InvalidGpuSelection`; they never silently switch devices. Duplicate indistinguishable names remain Auto-only. Driver changes may invalidate identity and require reselecting. Verified here: RTX 4090 at NCNN index 0 and AMD at index 1; real smoke jobs explicitly selected the RTX 4090.

## Validation performed on Windows, 2026-09-14

- Frontend build and TypeScript: passed through Tauri's build command.
- Frontend tests: 7 passed, including persisted GPU/tile validation and late-event ordering for previously queued jobs.
- `cargo check`: passed.
- `cargo test`: 27 passed; 2 opt-in GPU tests ignored by default. Covers pipeline planning, safety/overflow, real decoded mock outputs, first/second-pass failure and cancellation, serial scheduling, pause/resume, retry, duplicate/folder validation, restart recovery and Windows child-tree cancellation.
- Windows executable: `npm run tauri build -- --debug --no-bundle` passed. Development executable at `src-tauri/target/debug/clearce.exe`; installer/release packaging was not tested.
- Opt-in `real_multipass_and_batch`: passed using installed NCNN Vulkan/x4plus on RTX 4090. Real 2x/4x/8x/12x dimensions are listed above. Three additional PNG/JPG/WebP batch sources produced three valid 256 × 192 outputs, with seven completed history records across the single/batch checks. Real cancellation during pass 2 produced no final output and cleaned temporary files; source bytes were unchanged.
- Actual native WebView2 UI via CDP: GPU/tile controls and saving, three real batch outputs, Start, Pause after current, Resume, History/Models/Settings/Batch layout checks passed. After closing/reopening the Windows app, all three completed queue entries persisted in stopped state and an actual enhanced comparison image loaded. Folder ingestion in this smoke used the typed native command; the Phase 3 Add Folder OS picker itself was not exercised.
- Evidence and generated smoke files are local ignored `.qa/phase3-*` artifacts. No image data was sent to a cloud inference service.

Run the opt-in smoke explicitly:

```powershell
$env:ENHANCECE_TEST_ENGINE_DIR = Join-Path $env:LOCALAPPDATA 'dev.usaince.enhancece\engines\realesrgan'
$env:ENHANCECE_PHASE3_OUTPUT = (Join-Path $PWD '.qa/new-phase3-smoke')
cargo test --manifest-path src-tauri/Cargo.toml --test phase3 real_multipass_and_batch -- --ignored --nocapture
```

## Limitations and Phase 4 deferrals

Photo/x4plus only. No separate face model, adjustable enhancement profiles, AI preview patches, native-3x model certification, simultaneous GPU jobs, automatic OOM recovery, overwrite mode, recursive folder import, automatic interrupted-job resume, or release installer certification. Large inputs may be rejected by intermediate/memory limits even if final dimensions alone fit. Phase 4 was not started.
