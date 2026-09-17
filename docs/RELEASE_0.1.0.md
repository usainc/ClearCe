# ClearCe 0.1.0 — Public Beta

ClearCe 0.1.0 is the first public beta of the local AI image enhancement desktop application for Windows.

## Highlights

- Local GPU-powered AI image enhancement
- 2x, 4x, 8x and 12x output
- Batch processing
- Before / After comparison
- Automatic GPU/model recommendation
- One-click managed Real-ESRGAN setup
- Manual local engine support
- English and Turkish UI
- Six saved workspace themes

## AI engine

Real-ESRGAN is not bundled with the installer. After explicit user action, ClearCe can download a pinned official Real-ESRGAN NCNN Vulkan package and verifies its SHA-256 plus expected archive layout before activation. Manual local engine configuration remains supported.

Normal image inference runs locally after the engine is installed. Internet access is not required for normal inference.

## Installation

1. Download `ClearCe_0.1.0_x64-setup.exe` from this release.
2. Run the installer and select English or Türkçe.
3. Launch ClearCe and open **Models / Modeller**.
4. Keep **Auto (Recommended) / Otomatik (Önerilen)** enabled.
5. Install the recommended managed engine or select an existing compatible local engine.
6. Open an image and begin enhancing.

For a complete English/Türkçe walkthrough, SmartScreen guidance, AI-engine setup and troubleshooting, see the [Installation Guide / Kurulum Rehberi](INSTALLATION.md).

## Windows trust

The ClearCe 0.1.0 installer is currently unsigned. Windows SmartScreen may show a warning because the release does not yet have an Authenticode publisher signature. Download only from the official ClearCe repository and verify the SHA-256 value. Do not disable Windows Defender or SmartScreen for ClearCe.

## Integrity

This release includes:

- `ClearCe_0.1.0_x64-setup.exe`
- `SHA256SUMS.txt`
- `release-manifest.json`

Expected installer SHA-256:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

The manifest records `signed: false` and `engineBundled: false`.

## Known limitations

- Unsigned installer
- No dedicated Face Restore model yet
- No automatic updater
- Metadata preservation is deferred
- Windows x64 only
- High scale factors require substantially more memory, storage and processing time

This is a public beta. Please use [GitHub Issues](https://github.com/usainc/ClearCe/issues) for reproducible bugs that do not contain private images, credentials or security details. Report vulnerabilities through [private vulnerability reporting](https://github.com/usainc/ClearCe/security/advisories/new).
