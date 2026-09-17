# Phase 4.7 — GitHub Public Release

Phase 4.7 publishes the approved ClearCe 0.1.0 release candidate without rebuilding or changing application code. The public repository continues from its reviewed snapshot history so private development branches and historical machine-specific paths are not pushed.

## Public release identity

- Repository: `https://github.com/usainc/ClearCe`
- Stable branch: `main`
- Tag: `v0.1.0`
- Release title: `ClearCe 0.1.0 — Public Beta`
- Release page: `https://github.com/usainc/ClearCe/releases/tag/v0.1.0`

## Approved artifacts

Only these files may be attached to the GitHub release:

- `ClearCe_0.1.0_x64-setup.exe`
- `SHA256SUMS.txt`
- `release-manifest.json`

Expected installer SHA-256:

```text
32c88d6dcce5f4d4774f9b7c59fa2f9222c05a3d594914b41d60947afe0ba116
```

The approved manifest records an unsigned installer and `engineBundled: false`. Real-ESRGAN runtime/model files are not release assets.

## Publication checks

Before tagging, the public tree is scanned for common credential material, private keys/certificates, local user paths, databases, logs, downloaded engines/models, archives and build/debug output. Immediately before upload, the installer, checksum file and manifest must be checked again against the approved SHA-256 value.

After publication, the installer is downloaded from the GitHub release and hashed independently. That post-publication result is part of the release handoff because it occurs after this immutable tag is created.

## Scope

This phase changes public documentation and screenshots only. It does not rebuild ClearCe, modify product behavior, add a service, bundle an AI engine, select a project license or begin 1.1 development.
