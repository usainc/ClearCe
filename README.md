<p align="center"><img src="public/brand/clearce-logo.png" width="360" alt="ClearCe — Local AI Image Enhancer — by UsainCe.dev"></p>

# ClearCe

**Local AI Image Enhancer · by UsainCe.dev**

A Windows desktop utility for enhancing images on your own hardware. ClearCe combines local Real-ESRGAN processing, a persistent batch queue and real before/after comparison in a focused dark workspace. English and Türkçe are supported, with live switching and saved preferences.

**Status:** Windows x64 MVP · version 0.1.0 · unsigned installer · AI engine installed separately.

![ClearCe Windows workspace](docs/screenshots/home.png)

## Features

- JPG, PNG and WebP input/output; originals remain untouched.
- **2x / 4x / 8x / 12x** using the general `realesrgan-x4plus` model.
- Batch files/folder import, sequential processing, pause after current, cancel and retry.
- Actual output comparison, bounded display previews, persistent history and output-folder actions.
- English/Turkish, first-run Windows language detection and persisted preferences.
- GPU selection, tile size, readiness checks and managed temporary-file cleanup.

[Screenshots](docs/SCREENSHOTS.md) · [Feature status](docs/MVP_FEATURE_STATUS.md)

## Install and configure

Build the installer below, or use a maintainer-provided `ClearCe_0.1.0_x64-setup.exe`. No hosted binary download is promised here. The installed app does not require Node.js, Rust or Python.

Requirements: **Windows x64**, Microsoft WebView2 and a compatible **Vulkan GPU/driver**. The unsigned installer may trigger a Windows warning. Installing WebView2 may need internet access if it is absent.

**Real-ESRGAN and model weights are not bundled or automatically downloaded.** Obtain a trusted Windows NCNN Vulkan distribution, extract it, then choose **Models → Select Local AI Engine** (Türkçe: **Modeller → Yerel AI Motorunu Seç**).

```text
trusted-engine/
  realesrgan-ncnn-vulkan.exe
  models/
    realesrgan-x4plus.param
    realesrgan-x4plus.bin
```

ClearCe checks files, executable launch and Vulkan readiness before publishing the managed installation. These checks do not authenticate a publisher: select only trusted software. Invalid imports preserve a working installation. If the engine disappears, History and Settings remain accessible; select a valid engine again. See [engine setup and troubleshooting](docs/ENGINE_SETUP.md).

## Enhance an image

1. Open or drop a local image on Home.
2. Select Photo, a scale and output format; optionally choose an existing writable output folder.
3. Choose **Enhance Image / Görseli İyileştir**.
4. Compare the actual result or open the output file/folder.

The model runs native 4x inference. 2x reduces that result; 8x/12x use two 4x passes and reduce the intermediate 16x output. High scales require more memory and disk space. There is no cloud inference or image upload in the processing pipeline.

Existing outputs are never overwritten. EXIF orientation is applied, then metadata is removed. Input limits are 100 MB and 6 megapixels; final output is limited to 96 MP and intermediate output to 128 MP, with additional resource checks.

## Develop and build

Install Node.js 20.19+ (or 22.12+), npm, Rust MSVC, Visual Studio C++ Build Tools and WebView2. From the repository root:

```powershell
npm ci
npm run tauri dev
```

This starts the Windows desktop app. `npm run dev` alone is a frontend development server and cannot perform native enhancement.

```powershell
npm test
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build -- --debug --no-bundle
npm run tauri build
```

Outputs: `src-tauri/target/debug/clearce.exe`, `src-tauri/target/release/clearce.exe`, and `src-tauri/target/release/bundle/nsis/ClearCe_0.1.0_x64-setup.exe`.

Real GPU tests are opt-in. Set `ENHANCECE_TEST_ENGINE_DIR` to a trusted managed runtime; see [engine setup](docs/ENGINE_SETUP.md). Legacy environment/storage identifiers intentionally remain compatible with earlier EnhanceCe installations; see [compatibility](docs/PHASE4_2.md).

## Limits and future work

No specialist portrait, face, anime, text or restoration model; no independent AI preview-patch generation; no adjustable quality/denoise/sharpen backend; no metadata retention, overwrite or automatic updater. Video, cloud and non-Windows releases are outside this MVP. Specialist models and release signing remain future work, without dates or promises.

## License

**Project license selection is pending.** Public source availability does not grant an open-source license. No MIT, Apache or other project license is implied. Approved ClearCe artwork is included as product branding; no separate reuse license is granted here.

[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) preserves dependency notices. These do not establish redistribution rights for external engine/model packages, which are intentionally excluded.

[Release notes](docs/RELEASE_0.1.0.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)
