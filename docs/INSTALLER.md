# ClearCe Windows installer

ClearCe 0.1.0 uses a native, Unicode NSIS wizard for Windows x64. The approved dark ClearCe artwork is converted to conventional header/sidebar bitmaps; application themes remain independent.

## Wizard

The language dialog is the first interaction: English or Türkçe. Then Welcome → Directory → Shortcut options → Summary → Progress → Finish. Existing installations additionally show Tauri's maintenance page. Installation is **current user only**, so there is no unnecessary scope page or administrator request. Default directory is `%LOCALAPPDATA%\ClearCe`; spaces and Unicode are allowed.

Start Menu shortcut defaults on; Desktop defaults off. Finish offers launching ClearCe, checked by default. `/S` is unattended and does not launch unless `/R` is explicitly supplied. `/NS` suppresses new shortcuts. Existing shortcuts are maintained during updates. No startup registration, service, scheduled task, watchdog or tray persistence is created.

Microsoft Edge WebView2 Runtime must already be installed. Setup checks its registered version and stops with a localized message when missing. It does not download or execute a prerequisite installer. Obtain WebView2 separately from [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/). Real-ESRGAN and models are not bundled or downloaded by NSIS. After installation, the user can explicitly start the app's pinned, verified managed-engine download or use the local directory import.

## Initial application language

NSIS writes only `{"version":1,"language":"en"}` or `"tr"` to `%APPDATA%\dev.usaince.enhancece\installer-language.json`. The legacy identifier intentionally preserves existing installations.

The Rust `installer_language` command accepts only this fixed path, schema version 1, two enum values and at most 128 bytes; non-files, symlinks, reparse files, extra fields and malformed content are rejected. It returns no caller-selected settings or paths. The frontend reads the existing `enhancece.settings.v1` entry in `settings.json`. Saved `en`/`tr` wins; otherwise it saves the installer choice through that same store, then calls the fixed-path acknowledgment to remove the handoff. If saving fails, the handoff remains for retry. Without a valid handoff, existing Windows locale fallback applies. Invalid/obsolete handoffs are removed. There is no second lasting language store. NSIS's normal installer-language registry value controls only the wizard/uninstaller.

## Upgrade and removal

The product name, binary name, bundle ID and single `ClearCe` uninstall registry key remain stable. Settings in roaming app data, local history, engine configuration and engines survive reinstall/update. The application's existing startup cleanup handles owned stale job intermediates and marks interrupted jobs failed; it never resumes them automatically.

Uninstall removes installed application resources and matching installer-owned shortcuts. Settings, history, engines and output images are always preserved; there is no recursive app-data deletion option. The app never owns the user's source images. The legacy `Pictures\EnhanceCe` default output location remains compatible. Manually relocating an existing installation should use uninstall/reinstall; user data survives both.

## Build tooling and upstream maintenance

Run `npm run release:windows` on Windows with Node/npm, Rust/MSVC, Python 3 (standard library only for metadata), WebView2 and Tauri prerequisites. Developer scripts may use PowerShell/Python; neither is shipped or invoked by the installed application/setup. The pre-bundle step uses standard Windows VERSIONINFO resource APIs before any signing. Release Rust source paths are remapped; artifact verification rejects the actual checkout path in UTF-8/UTF-16.

`src-tauri/installer/installer.nsi` derives from the [Tauri CLI v2.11.4 template](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi), under its MIT/Apache-2.0 licenses. Local changes: localized welcome/finish; shortcut and summary pages; prerequisite check without download code; app-language seed; preserve app data; metadata. Retain upstream signing, uninstall resource inventory and update mechanics. Review template changes before upgrading the pinned CLI. NSIS's standard LZMA archive compression is not PE packing/obfuscation. NSIS necessarily extracts its own plugins/uninstaller helper into its private temporary directory; no AI/app executable is fetched there.
