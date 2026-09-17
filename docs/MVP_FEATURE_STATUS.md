# ClearCe MVP feature truth table

Version 0.1.0. Status describes implemented behavior, not future roadmap promises.

| Feature | Status | Behavior / limits |
| --- | --- | --- |
| Windows desktop shell | REAL | Tauri/WebView2, native file dialogs, local Rust services |
| English / Turkish | REAL | i18next catalogs, live switching, existing settings persistence, first-run Windows locale detection |
| Localized errors / processing states | REAL | Stable Rust codes mapped to friendly localized text; technical diagnostics retained |
| 2x enhancement | REAL | Native x4plus 4x AI followed by exact 2x resize |
| 4x enhancement | REAL | Native x4plus inference |
| 8x / 12x enhancement | REAL | Two native 4x passes, final resize to requested dimensions |
| General/Photo model | REAL | Real-ESRGAN NCNN Vulkan x4plus, managed or manually installed |
| Anime / Illustration model | REAL | RealESRGAN x4plus anime 6B from the approved managed package; selected by Auto for anime mode |
| Local engine installation | REAL | One-click pinned/hash-verified managed package and advanced folder import; separate atomic destinations |
| Auto model recommendation | REAL | Deterministic Vulkan device/type/dedicated-memory tiers with conservative fallback and EN/TR explanations |
| Engine health / first-run readiness | REAL | Integrity, executable launch, model header, Vulkan, selected GPU, temp/output usability |
| Bundled engine/model | DISABLED | Runtime distribution supplied for this project has no license notices; redistribution not assumed |
| Model version | REAL | Reported only if present in manifest; otherwise unknown |
| Batch processing | REAL | Up to 200 retained items, single GPU job; pause after current, cancel, retry |
| Folder import | REAL | Nonrecursive, bounded to 1000 scanned entries |
| Queue restart recovery | REAL | Waiting retained; interrupted execution failed explicitly; no auto-resume |
| History | REAL | Recorded paths, dimensions, status, times and actual output; maximum 500 records per listing |
| Delete history | REAL | Deletes metadata only; does not delete source/output files |
| Open output / folder | REAL | Native recorded-image actions |
| Re-run settings | REAL | Restores the job request to Home; requires explicit Enhance click |
| Before/After | REAL | Actual output; displayed results bounded to 2048px, full-size export unchanged |
| Demo image | REAL | Clearly marked sample / simulated softness until processed |
| Dark / Light / Midnight / Graphite / Forest / System | REAL | Persisted theme, system color-scheme listener, keyboard focus styles |
| Default scale / output format / folder | REAL | Persisted defaults; native folder validation before save; existing queue items unchanged |
| GPU selection / tile size | REAL | Auto or verified stable identity; stale selection rejected |
| Cache size / Clear Cache | REAL | App-owned job intermediates and generated previews only; active processing blocks deletion |
| Startup stale cleanup | REAL | Exclusive session lock; managed job directories only; reparse points rejected |
| Metadata retention | DISABLED | EXIF stripped, orientation applied before processing |
| Output overwrite | DISABLED | Fixed keep-both policy; collision suffixes; originals preserved |
| Preview quality selector | DISABLED | Fixed bounded display preview; exported quality unchanged |
| Quality profiles / separate denoise / sharpen | DISABLED | Fixed general model; no independent controls |
| AI preview patch | DEFERRED | Preview page is a comparison, not separate patch inference |
| Portrait / face / text / restore models | DEFERRED | No unique specialist processing claimed |
| Hardware tier / dedicated VRAM | REAL | Vulkan-reported device data; dedicated memory shown only for discrete GPUs; no live utilization or benchmark claims |
| Telemetry | DISABLED | No telemetry implementation |
| Automatic updates | DISABLED | No updater backend |
| Windows NSIS installer | REAL | Per-user x64, unsigned, version 0.1.0 |
| Cloud, accounts, payments, video, other OS | DEFERRED | Outside MVP |
