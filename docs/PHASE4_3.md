# Phase 4.3 — Installer wizard and Windows trust

Baseline: clean `phase/4.2-clearce-rebrand-public-repo`, `617000cdc811d15cb4a4c2572daf9791fcb7dbcd`. Work branch: `phase/4.3-installer-trust`. Product remains ClearCe 0.1.0, native Tauri/NSIS Windows x64. No web deployment, server, model download, telemetry or v1.1 features.

## Delivered

* First-interaction English/Türkçe language picker; localized welcome, directory, shortcuts, summary, progress and finish. Standard Windows controls and approved ClearCe icon/header/sidebar artwork.
* Current-user installation, no unnecessary elevation. Start Menu on/Desktop off defaults; independent shortcut choices and finish launch option. No startup persistence.
* Versioned, bounded, fixed-path installer-language handoff into the existing settings store. Saved language wins; successful first-run persistence removes the handoff. Invalid tokens rejected; failed persistence retains a valid handoff for retry.
* Existing data identity retained. No uninstall app-data deletion. Settings/history/engine/output preservation validated. Existing stale-job cleanup/recovery unchanged.
* WebView2 download/bootstrap code removed from the custom template. A localized prerequisite check requires an already installed runtime. External AI engines remain unbundled.
* Imported engine health checks now execute from the stable managed directory after staging promotion. Rejected imports restore the prior installation. No shell, arbitrary executable picker or additional Tauri capability was introduced.
* Production VERSIONINFO finalization, source-path remapping and artifact checks. Environment-driven legitimate certificate signing configuration, timestamp validation, hashes/manifest, optional Defender scan and public-source release guards.

See [INSTALLER.md](INSTALLER.md) for exact storage/uninstall behavior and template provenance; [WINDOWS_TRUST.md](WINDOWS_TRUST.md) for threat review, signing, SmartScreen and false-positive handling.

## Validation (Windows host, September 16, 2026)

| Check | Result |
| --- | --- |
| Frontend tests | 22 passed; installer EN/TR mapping, explicit-language preservation, invalid tokens and failed-save retry included |
| Rust tests | 33 passed; 3 opt-in tests ignored in default suite (two GPU fixtures and symlink permission fixture) |
| Stable engine import | Targeted invalid-PE rollback test passed; native valid/invalid import and inference passed |
| Frontend production build / cargo check / debug / release / NSIS | Passed |
| npm audit | 0 vulnerabilities; CLI pinned to 2.11.4 to match the reviewed NSIS template |
| Rust dependency audit tooling | cargo-audit/cargo-deny not installed; no vulnerability-database audit claimed |
| Secret/signing-file/config guards | Passed for current tracked/publishable files; no secret values logged |
| Release integrity | Hashes recalculated from final artifacts; manifest checked against files; no checkout path detected in UTF-8/UTF-16 |
| English installation | Language picker, branded welcome, directory, options, summary and installed EN preference verified |
| Turkish installation | Localized pages and finish verified; fresh app opens in TR with missing-engine setup; handoff removed after persistence |
| Directory | Installed and launched from a path containing spaces and Turkish characters; restored standard install location afterward |
| Shortcuts | Default Start Menu on/Desktop off; alternate Start Menu off/Desktop on verified against actual `.lnk` existence |
| Keyboard | Space On→Off, Tab to Finish, Shift+Tab to Launch, Enter finish and Escape language-picker cancellation verified with native controls |
| Upgrade/reinstall | Existing 0.1.0 → new 0.1.0 package; persisted settings/history/engine hashes unchanged; English installer seed did not overwrite saved Turkish preference; app relaunched |
| Apps & Features | Exactly one ClearCe uninstall entry after final install |
| Uninstall | Binaries/owned shortcuts removed; settings/history and generated output hashes preserved |
| Real processing | 2x, 4x, 8x, 12x and three-file batch passed in installed Tauri/WebView2 with a real Vulkan engine |
| App regressions | History/result comparison, settings save/relaunch, EN/TR switching, engine verification and missing/invalid engine passed |
| Defender | Final EXE and installer scanned normally after definitions update; exact versions and result recorded in generated manifest |
| User workspace restoration | Original settings, history and engine restored with matching SHA-256; original EN/Light preference and ready engine verified |

Native wizard automation initially had focus/timing problems; final keyboard assertions waited for confirmed control focus. Failed automation attempts were not treated as product successes. The packaged application checks used actual WebView2→Tauri IPC, not a browser-only mock.

## Signing and limitations

`SIGNING_READY=true`, `SIGNED=false`: no legitimate code-signing certificate was available. Both artifacts are `NotSigned`, with no signer/timestamp. They are **UNSIGNED TEST RELEASE** artifacts. CA-backed signing, timestamp-provider operation and signed installed-uninstaller qualification remain unverified until a real identity exists. No SmartScreen-warning-free claim is made.

Clean installs used isolated application profiles on this existing Windows host, not a fresh Windows VM. The missing-WebView2 branch was reviewed statically; the host's shared runtime was not removed to test it. A different numeric-version upgrade and other Windows versions/DPI configurations remain future release-matrix checks. Windows/platform traffic was not packet-captured. No public GitHub release, Store/MSIX package or external false-positive submission was created.

Artifacts and local QA captures/logs remain ignored. Publish only reviewed final artifacts with their matching `SHA256SUMS.txt` and `release-manifest.json`; keep the private development history separate from the public snapshot repository.
