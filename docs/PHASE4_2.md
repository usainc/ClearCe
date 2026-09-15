# ClearCe — Phase 4.2 rebrand and public repository

Version 0.1.0, Windows x64. Private development baseline: `9ed6433`, branch `phase/4.1-localization-hardening`. Work continued on `phase/4.2-clearce-rebrand-public-repo`; approved history was not rewritten.

## Identity and assets

EnhanceCe is now **ClearCe**, with the canonical tagline **Local AI Image Enhancer** and signature **by UsainCe.dev**. The owner-provided logo is preserved byte-for-byte as `public/brand/clearce-logo-source.png`; display/logo/icon variants only crop and resize that artwork. No alternative logo was generated. Its source PNG has no EXIF or additional metadata.

Sidebar, topbar, HTML title/favicon, localized product copy, native window/startup messages, npm/Cargo package names, Windows icons, NSIS display name and release filenames use ClearCe. Unused iOS/Android/macOS icon artifacts and obsolete Phase 1 screenshots were removed. The established interface layout and functionality remain.

## Compatibility

The bundle identifier `dev.usaince.enhancece`, Rust library name `enhancece_lib`, persisted settings key `enhancece.settings.v1`, legacy developer environment variables and default Pictures/EnhanceCe output directory remain intentional. Existing local/roaming storage, history, queue, engine and custom output preferences retain their locations. No data migration or destructive rename is required.

ClearCe emits `clearce.exe` and `ClearCe_0.1.0_x64-setup.exe`. The installer uses its new product name, so an old EnhanceCe shortcut/install may coexist. Both apps share the existing exclusive session lock; close the old app before launching ClearCe. Retire the old installation without deleting its app data. The rebrand does not pretend to be a tested automatic cross-name uninstall migration.

## Public repository preparation

README describes actual features, prerequisites, development/build commands, engine setup, limitations and pending project license. ENGINE_SETUP, SCREENSHOTS, CONTRIBUTING and SECURITY provide concise supporting guidance. Dependency notices remain intact; no engine/model redistribution rights are assumed. License selection remains an owner decision, and public visibility is not described as an open-source license grant.

Ignore rules cover environment files, credentials, model weights, engine directories, logs, databases, temporary files, build/installer output and editor/OS junk. Required source PNG/ICO assets remain tracked. Current tracked text was scanned for common credential patterns and personal home-directory paths; no obvious matches remained. Machine-specific documentation paths were replaced with repository-relative or environment-variable examples. Public screenshots are captured from a QA profile rather than the owner's real settings/history.

Historical commits contain machine-specific documentation paths and obsolete screenshots. **Do not publish all branches or mirror this development repository.** Publish only a fresh current-tree snapshot with one new public initial commit. This preserves approved private history without exposing it or force-rewriting it. The public snapshot is prepared under ignored `.qa/ClearCe-public`; future public work should use a clone of the public repository.

## Validation

- 17 frontend tests and 32 Rust tests passed; the three opt-in native/GPU tests remain excluded from the default Rust run.
- Frontend production build, cargo check, Windows debug/release build and NSIS packaging passed. Debug frontend asset hashes match the release frontend assets after formatting.
- Installed ClearCe launches with the approved logo/name and English/Turkish copy. Fresh-profile locale detection and live switching retain the existing behavior.
- Installed regression exercises real 2x/4x/8x/12x, three-image batch, recorded result comparison, engine missing/invalid/valid states, and settings/history persistence after relaunch.
- QA files, generated outputs and runtime binaries remain ignored. Original user profiles are preserved separately during testing and restored afterward.

## Limitations

Unsigned MVP installer, no bundled AI engine, no final project license. Updater, overwrite, metadata retention, specialist models, video, cloud services and other OS releases remain deferred. No v1.1 feature work was started. Earlier phase reports are historical records; current feature truth is in MVP_FEATURE_STATUS.md.
