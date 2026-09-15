# Phase 4.1 — Localization and release hardening

Windows x64, version 0.1.0. Branch `phase/4.1-localization-hardening`, based directly on approved `ee270f2`. No v1.1 model work.

## Localization

i18next/react-i18next owns English and Turkish catalogs in `src/i18n/locales`. The central module handles translation, enum labels, typed errors, dates, times and numbers. English is the fallback. Catalog tests enforce matching keys/interpolation and coverage of every Rust ErrorCode variant.

First launch calls Windows GetUserDefaultLocaleName: `tr*` selects Turkish; other locales select English. An explicit language in the existing settings store wins. Language changes save independently of output/GPU validation, take effect immediately, and persist through relaunch/reinstall. No separate settings database was added. Native system dialogs retain Windows shell localization; an early fatal startup dialog follows Windows locale because saved settings may be unavailable.

Navigation, Home, Preview, processing stages/activity, queue/statuses, History, Models, Settings, onboarding and important errors use catalogs. Internal model/mode IDs stay stable across language changes. Technical product names and paths remain unchanged. Rust retains technical diagnostic messages; React maps error codes and phase codes instead of displaying those messages. Older persisted activity records receive a neutral localized fallback.

## Engine setup and recovery

Missing-engine onboarding explains successful app installation and the need to select a trusted local Real-ESRGAN folder. Fixed expected files are shown. Import verifies paths, hashes, model headers, executable launch and Vulkan discovery before publishing the managed installation. A valid runtime with no usable GPU is explicitly not ready. Invalid imports leave the previous working installation intact. Replacements retain the previous folder under `previous-<uuid>`; failed publication attempts rollback. Import is serialized against active processing.

The engine is intentionally not bundled: redistribution terms for the supplied engine/model were not verified. No automatic download or specialist model was added. A disappeared engine does not erase settings/history or prevent access to those pages.

## Cleanup review

Startup and explicit cache clearing use the same owned-job cleanup path, under the exclusive application session lock and with active-job exclusion. Root deletion, external targets, parent traversal and non-job targets are rejected. Root, target and recursive entries are checked for Windows reparse attributes and canonical containment; scanning is bounded. Sources, outputs, settings, history and engines are outside the deletion scope.

The opt-in `real_symlink_escape_is_rejected` test **ran successfully on Windows**. It creates an actual directory symlink and a nested directory symlink in one disposable test tree, pointing to a sibling sentinel directory. Cleanup refuses both and the sentinel survives. No OS security feature was disabled and no blocked command was bypassed. This is an actual Windows directory-symlink/reparse test, not an NTFS junction-tag creation test. The latter remains a manual variant; all reparse attributes are rejected by the same guard. These application guards are not a sandbox against a hostile same-user process concurrently replacing filesystem objects.

## Validation — 2026-09-15

- Frontend: 17 tests passed; catalog parity, real Rust error-code coverage, fallback, locale selection, runtime translation, locale formatting and independent language persistence.
- Rust: 32 tests passed; three opt-in tests skipped by default. The real directory-symlink test was then run explicitly and passed. Real GPU coverage was exercised through the installed application below.
- Frontend production build, cargo check, Windows debug/release and NSIS builds passed.
- A new test initially used Node filesystem types unavailable to the frontend tsconfig. It was changed to a Vite raw source import; the 17-test suite and final frontend/debug build then passed. Final frontend assets match those packaged in the release.
- Fresh installed profile detected the actual Windows `tr-TR` locale. English/Turkish switching and stable `Photo` option values passed.
- Installed real x4plus inference: 64×48 source produced 128×96, 256×192, 512×384 and 768×576 at 2x/4x/8x/12x. Three-image batch completed. No fabricated inference/status data.
- Relaunch and same-version installer repair preserved Turkish, WEBP preference and seven successful history records.
- Missing, invalid and valid engine checks passed. Invalid selection preserved the working engine. Removing the test profile's managed engine left History/Settings usable; installing the preserved trusted runtime restored readiness.
- Native Windows picker opened from the keyboard, invalid folder selection produced the localized error, actual image selection imported a local image, and Escape closed the engine dialog. Visible focus styling is retained. Native dialog shell controls follow Windows language.
- Both languages' six primary pages were captured and checked in actual installed Tauri WebView2 via local CDP. No document horizontal overflow at 1440×1000. Turkish settings and history text were visually reviewed. This is not an exhaustive screen-reader or minimum-window-size certification.
- Some first automation attempts failed due to CDP error-object serialization, startup timing and native picker automation; corrected checks above passed. These were test harness issues, not claimed passing runs.

QA scripts/screens/results are local ignored `.qa/phase4_1-*` artifacts. Existing user profiles were preserved before clean-profile testing and restored afterward. Generated images, test profiles and third-party runtime binaries are not committed.

## Release limits

Unsigned NSIS, version 0.1.0. WebView2 bootstrapper may need network if the prerequisite is absent; image processing remains local. No updater, overwrite, metadata retention, adjustable denoise/sharpen/quality, face restoration, portrait/anime/text/restore model, cloud, accounts, video or non-Windows release. Disabled/deferred controls stay explicit. Independent AI preview patches and specialist models are future work, not part of this checkpoint.
