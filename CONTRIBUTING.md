# Contributing to ClearCe

Read README and the feature status table. Preserve local processing and existing-user data compatibility. Add both English and Turkish localization keys. Keep disabled/deferred features truthful.

Install the Node.js, Rust MSVC, Visual Studio C++ Build Tools and WebView2 prerequisites listed in README, then run `npm ci`. Use `npm run tauri dev` for the complete desktop application; `npm run dev` exercises only the React frontend. Native code lives under `src-tauri`.

Run the frontend and Rust checks in README. Native processing/packaging changes also need a Windows smoke test; browser-only tests cannot prove native behavior. Real-ESRGAN is not bundled, so never commit engines, weights, credentials, private images, logs, databases or build outputs.

Project license selection is pending. Discuss contribution licensing with the maintainer before submitting code/assets; do not assume this repository grants an open-source license.
