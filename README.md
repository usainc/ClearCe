<p align="center"><img src="public/brand/clearce-logo.png" width="320" alt="ClearCe — Local AI Image Enhancer — by UsainCe.dev"></p>

# ClearCe

**Local AI Image Enhancer for Windows**  
by UsainCe.dev

[![Release](https://img.shields.io/badge/release-v0.1.0-blue)](https://github.com/usainc/ClearCe/releases/tag/v0.1.0)
![Windows](https://img.shields.io/badge/platform-Windows%20x64-0078D6)
![Public Beta](https://img.shields.io/badge/status-Public%20Beta-orange)

ClearCe is a native Windows desktop application for enhancing images locally with a Real-ESRGAN NCNN Vulkan engine. Images stay on your computer during inference; there is no cloud image upload in the enhancement pipeline.

## Download

**Latest public beta:** [ClearCe 0.1.0](https://github.com/usainc/ClearCe/releases/tag/v0.1.0)

Download:

`ClearCe_0.1.0_x64-setup.exe`

Expected installer SHA-256:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

> ClearCe 0.1.0 is currently distributed with an **unsigned Windows installer**. Windows SmartScreen may show a warning because this release does not yet have an Authenticode publisher signature. Download only from this repository and verify the SHA-256 if desired. Do **not** disable Windows Defender or SmartScreen for ClearCe.

## Quick Installation

1. Download `ClearCe_0.1.0_x64-setup.exe` from the [official GitHub Release](https://github.com/usainc/ClearCe/releases/tag/v0.1.0).
2. Run the installer.
3. Select **English** or **Türkçe**.
4. Choose the installation directory and shortcut preferences.
5. Launch ClearCe.
6. Open **Models / Modeller**.
7. Keep **Auto (Recommended) / Otomatik (Önerilen)** enabled.
8. Select **Download Recommended / Önerileni İndir** to let ClearCe install the verified managed Real-ESRGAN package, or choose an existing compatible local engine.
9. Open an image and start enhancing.

The installed application does **not** require Node.js, Rust, Python or CUDA for normal use.

For the complete bilingual walkthrough, troubleshooting and reinstall/uninstall behavior, see **[Installation Guide / Kurulum Rehberi](docs/INSTALLATION.md)**.

## Features

- 2x, 4x, 8x and 12x enhancement
- GPU/Vulkan processing on compatible NVIDIA, AMD and Intel hardware
- Automatic GPU and model recommendation
- One-click managed Real-ESRGAN installation with pinned-source and SHA-256 verification
- Manual local engine support
- Persistent batch queue with pause, cancel and retry controls
- Before / After comparison of processed results
- PNG, JPG and WebP input/output
- English and Türkçe interface
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

## First Launch & AI Engine

**Real-ESRGAN is not bundled inside the ClearCe installer.**

When you open **Models**, ClearCe detects available Vulkan devices and shows a recommendation for your computer. If no managed engine is installed, you can explicitly select **Download Recommended**.

ClearCe then:

1. downloads only the pinned official Real-ESRGAN package allowed by its internal catalog;
2. verifies the package SHA-256;
3. validates the archive layout and expected files;
4. extracts the approved runtime into a ClearCe-managed directory;
5. performs engine/Vulkan health checks;
6. activates the supported general or anime model when ready.

A failed managed install does not replace a previously working engine. After a healthy managed engine is installed, normal image inference runs locally and does not require internet access.

Advanced users can instead choose **Select Local AI Engine / Yerel AI Motorunu Seç** and provide an extracted compatible Real-ESRGAN NCNN Vulkan directory.

More detail:

- [Installation Guide / Kurulum Rehberi](docs/INSTALLATION.md)
- [AI Engine Setup](docs/ENGINE_SETUP.md)
- [Engine & Model Policy](docs/ENGINE_MODELS.md)

## Auto Model Recommendation

ClearCe 0.1.0 uses a deterministic recommendation system based on the detected Vulkan device and available resources.

Current supported catalog:

| Use | Model |
| --- | --- |
| General photo | `realesrgan-x4plus` |
| Anime / illustration | `realesrgan-x4plus-anime` |

If a recommended supported model is missing, ClearCe can offer the approved managed package. Unsupported or deferred specialist models are not presented as active features.

## Supported Formats & Limits

- Input: JPG, PNG, WebP
- Output: PNG, JPG, WebP
- Maximum input file size: 100 MB
- Maximum input resolution: 6 megapixels
- Maximum final output: 96 megapixels

The native model produces a 4x result. ClearCe downsamples that result for 2x; 8x and 12x use two 4x passes and downsample the intermediate result. Higher scales require substantially more GPU memory, RAM, temporary disk space and processing time.

## Requirements

- Windows x64
- Microsoft Edge WebView2 Runtime
- Vulkan-compatible GPU and current graphics driver
- Compatible Real-ESRGAN NCNN Vulkan engine, installed through ClearCe or selected manually

ClearCe is not NVIDIA-only. Compatible AMD and Intel Vulkan devices are supported, subject to their drivers and available resources.

## Verify the Download

The GitHub Release includes `SHA256SUMS.txt` and `release-manifest.json`.

PowerShell:

```powershell
Get-FileHash .\ClearCe_0.1.0_x64-setup.exe -Algorithm SHA256
```

Expected:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

## Reinstall / Uninstall

Reinstalling the current ClearCe release preserves the existing user configuration where applicable, including settings, history and healthy engine configuration.

Uninstall removes installer-owned application resources and shortcuts. ClearCe does not delete your source images or completed output images. In 0.1.0, application settings/history/engine data are preserved so a later reinstall can reuse them.

## Troubleshooting

**No compatible Vulkan device detected**  
Install or update the GPU driver from NVIDIA, AMD or Intel's official source and restart ClearCe.

**WebView2 is missing**  
Install Microsoft Edge WebView2 Runtime from Microsoft's official WebView2 page, then run Setup again.

**Managed engine download fails**  
Check internet/proxy configuration and retry. The current working engine is left untouched on failure.

**Engine is invalid or incomplete**  
Use **Reinstall Managed Engine** or select a complete compatible local engine directory again.

**8x/12x cannot run**  
Try a smaller source image or a lower scale and ensure sufficient temporary disk space is available.

For the full bilingual troubleshooting guide, see [docs/INSTALLATION.md](docs/INSTALLATION.md).

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

## Security & Feedback

- Reproducible application bugs: [GitHub Issues](https://github.com/usainc/ClearCe/issues)
- Security vulnerabilities: [Private vulnerability reporting](https://github.com/usainc/ClearCe/security/advisories/new)

Do not upload private images, credentials or security-sensitive information to public issues.

## License

**A project license has not been selected.** Public source availability does not grant permission to use, modify, redistribute or sublicense ClearCe source code or artwork. No MIT, Apache, GPL or other project license is implied.

[Download v0.1.0](https://github.com/usainc/ClearCe/releases/tag/v0.1.0) · [Installation Guide](docs/INSTALLATION.md) · [Release Notes](docs/RELEASE_0.1.0.md) · [Security](SECURITY.md) · [Feature Status](docs/MVP_FEATURE_STATUS.md)
