<p align="center"><img src="public/brand/clearce-logo.png" width="320" alt="ClearCe — Local AI Image Enhancer — by UsainCe.dev"></p>

# ClearCe

**Local AI Image Enhancer for Windows**
by UsainCe.dev

## Overview

ClearCe is a native Windows desktop application for enhancing images with a local Real-ESRGAN NCNN Vulkan engine. Images stay on the computer during inference; there is no cloud processing or image upload in the enhancement pipeline.

ClearCe 0.1.0 is a public beta for Windows x64. Its installer is currently unsigned.

## Features

- 2x, 4x, 8x and 12x enhancement
- GPU/Vulkan processing on compatible NVIDIA, AMD and Intel hardware
- Persistent batch queue with pause, cancel and retry controls
- Before / After comparison of processed results
- Automatic GPU and model recommendation
- One-click managed Real-ESRGAN installation with SHA-256 verification
- Manual local engine support
- English and Türkçe interface
- PNG, JPG and WebP input/output
- Dark, Light, System, Midnight, Graphite and Forest themes

## Screenshots

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/home.png" alt="ClearCe main enhancement screen with Before and After comparison"><br><sub>Main enhancement workspace and Before / After comparison</sub></td>
    <td width="50%"><img src="docs/screenshots/models-managed.png" alt="ClearCe managed Real-ESRGAN setup and automatic recommendation"><br><sub>Managed engine setup and automatic recommendation</sub></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/screenshots/batch.png" alt="ClearCe batch queue"><br><sub>Persistent batch queue</sub></td>
    <td width="50%"><img src="docs/screenshots/settings-tr.png" alt="ClearCe settings in Turkish"><br><sub>Settings in Türkçe</sub></td>
  </tr>
</table>

The Home screenshot uses a clearly labeled demonstration comparison. Processed files use the actual generated output in the comparison view.

## Installation

1. Download `ClearCe_0.1.0_x64-setup.exe` from the [official GitHub Release](https://github.com/usainc/ClearCe/releases/tag/v0.1.0).
2. Run the installer.
3. Select English or Türkçe.
4. Launch ClearCe.
5. Open **Models**.
6. Choose the recommended managed AI engine installation, or select an existing compatible local engine.
7. Open an image and start enhancing.

The installed application does not require Node.js, Rust or Python.

ClearCe 0.1.0 is distributed with an **unsigned Windows installer**. Windows SmartScreen may display a warning because this release does not yet have an Authenticode publisher signature. Verify the installer hash below and download releases only from this repository.

## AI Engine Setup

**Real-ESRGAN is not bundled with ClearCe.**

After explicit user action, ClearCe can download a pinned official Real-ESRGAN NCNN Vulkan package. ClearCe verifies the package SHA-256 and expected archive layout before activation. It does not silently download an engine during installation or startup.

Manual setup remains available for users who already have a compatible local engine. See [AI engine setup](docs/ENGINE_SETUP.md) and the [supported model catalog](docs/ENGINE_MODELS.md).

## Auto Model Recommendation

The Models screen detects the local Vulkan devices and recommends a supported model/profile from the pinned ClearCe catalog. Recommendations are computed locally. The 0.1.0 catalog supports the general `realesrgan-x4plus` model and the anime/illustration `realesrgan-x4plus-anime` model.

## Local Processing

Normal image inference runs locally after a healthy engine is installed. Internet access is not required for normal inference. Network access is used only when the user explicitly starts the managed engine download.

Existing output files are preserved with a numeric suffix. EXIF orientation is applied and other metadata is removed in 0.1.0.

## Supported Formats

- Input: JPG, PNG, WebP
- Output: PNG, JPG, WebP
- Maximum input file size: 100 MB
- Maximum input resolution: 6 megapixels
- Maximum final output: 96 megapixels

The native model produces a 4x result. ClearCe downsamples that result for 2x; 8x and 12x use two 4x passes and downsample the intermediate result.

## Languages

- English
- Türkçe

The installer language seeds a fresh profile. The language can be changed immediately from Settings and is saved on the device.

## Requirements

- Windows x64
- Microsoft WebView2 Runtime
- Vulkan-compatible GPU and current driver
- Compatible Real-ESRGAN NCNN Vulkan engine, installed through ClearCe or selected manually

ClearCe is not NVIDIA-only. Compatible AMD and Intel Vulkan devices are supported, subject to their drivers and available resources.

## Verification / SHA-256

The release includes `SHA256SUMS.txt` and `release-manifest.json`. Verify the downloaded installer in PowerShell:

```powershell
Get-FileHash .\ClearCe_0.1.0_x64-setup.exe -Algorithm SHA256
```

Expected installer SHA-256:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

## Known Limitations

- The 0.1.0 installer is unsigned and may trigger SmartScreen.
- Windows x64 is the only supported platform.
- There is no dedicated Face Restore model yet.
- There is no automatic updater.
- Image metadata preservation is deferred.
- Portrait, screenshot/text and restoration specialist models are not available.
- 8x and 12x require substantially more memory, disk space and processing time.

## Development

Install Node.js 20.19+ (or 22.12+), npm, Rust MSVC, Visual Studio C++ Build Tools and WebView2. From the repository root:

```powershell
npm ci
npm run tauri dev
```

Run the focused checks before a pull request:

```powershell
npm test
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

`npm run dev` starts only the frontend development server and cannot perform native image enhancement. See [Contributing](CONTRIBUTING.md) for repository rules.

## Third-Party Software

Real-ESRGAN, NCNN runtime files and model weights are obtained separately and remain under their upstream terms. ClearCe does not redistribute them in its installer. Managed setup downloads the pinned official upstream package only after user action.

Dependency notices are preserved in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## License

**A project license has not been selected.** Public source availability does not grant permission to use, modify, redistribute or sublicense ClearCe source code or artwork. No MIT, Apache, GPL or other project license is implied.

[Release notes](docs/RELEASE_0.1.0.md) · [Security](SECURITY.md) · [Feature status](docs/MVP_FEATURE_STATUS.md)
