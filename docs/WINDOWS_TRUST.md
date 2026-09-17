# Windows trust and release verification

## Current state

**SIGNING_READY=true; SIGNED=false.** The environment-driven signing workflow is prepared, but no trusted signing certificate was available for this phase. Current artifacts are **UNSIGNED TEST RELEASE**, not a signed public release. Debug artifacts remain DEBUG. No self-signed release certificate was created. A live CA-backed signing run remains unverified until a legitimate identity is provisioned.

ClearCe's intended official download channel is [GitHub Releases](https://github.com/usainc/ClearCe/releases). This phase prepares local artifacts; it does not publish a release or assert that an installer is already available there. Use only an explicitly published version with its release manifest and hashes. Project licensing must also be selected before claiming an open-source release.

## Signing workflow

Use a legitimate CA-issued Windows code-signing identity in `CurrentUser\My`, with its provider/hardware-backed private key available to SignTool. Install Microsoft's Windows SDK SignTool on the build host and set:

```powershell
$env:CLEARCE_SIGNING_THUMBPRINT = '<certificate thumbprint>'
$env:CLEARCE_SIGNING_TIMESTAMP_URL = '<CA-approved HTTPS RFC3161 service>'
powershell -NoProfile -File scripts/release.ps1 -Signed
```

No private keys/passwords enter configuration. The temporary thumbprint/timestamp config is under ignored `target` and removed in `finally`. Signing files (`.pfx`, `.p12`, private `.pem`, `.key`, keystores) are ignored and release guards reject tracked signing material. Protect the build account/CI environment; never commit or print credentials. Certificate subject, not the branding string `UsainCe.dev`, determines the cryptographically verified publisher. Branding does not claim a legally verified company identity.

Order: build → finalize VERSIONINFO → Tauri NSIS bundle marker → SHA-256 Authenticode signature → build/sign NSIS uninstaller and installer → signature verification → SHA-256 manifest. Tauri also signs its private copies of NSIS support plugins. No third-party AI engine is bundled or re-signed. Future ClearCe-owned helpers must be registered with the bundler and included in release validation before shipping.

Tauri restores the unpatched standalone main EXE after bundling; the workflow separately signs that final standalone artifact. It differs from the installed EXE's NSIS marker, so compare each artifact against its own hash. Do not apply either hash to the other file. No resource mutation occurs after signing. The metadata finalizer refuses an already signed PE. `Get-AuthenticodeSignature` must report `Valid`, with a timestamp certificate, for `-Signed` verification to pass. Inspect SignTool `/pa /v` output and the installed app/uninstaller when performing the first real signed-release qualification. The unsigned workflow does not claim to have tested provider credentials or timestamp longevity.

Microsoft Artifact Signing is a possible future provider integration, not configured in this phase. EV is not a SmartScreen bypass. Signing establishes publisher identity and continuity; new signed files may still be warned about. See [Microsoft's SmartScreen guidance](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation) and [Tauri signing lifecycle](https://v2.tauri.app/distribute/sign/windows/). Store/MSIX distribution can be evaluated later; Store-certified MSIX signing is a separate path and is not implemented here.

## Integrity and Defender

`scripts/verify-release.ps1` creates `src-tauri/target/release/release-integrity/SHA256SUMS.txt` and `release-manifest.json` for only the final standalone EXE and NSIS installer. The manifest records Windows/x64, version, hashes, signature status/subject, timestamp subject, `engineBundled:false`, `managedEngineDownload:true`, and EN/TR support. Check downloaded files with `Get-FileHash -Algorithm SHA256` against that release's manifest. Run `node scripts/check-release.mjs` to check current source secret patterns, ignored credential formats, setup policy and available manifest hashes. Pattern scanning is not proof that no possible secret exists.

Optional normal Defender validation:

```powershell
powershell -NoProfile -File scripts/verify-release.ps1 -DefenderScan
```

The script records Defender engine/definition versions, attempts a normal signature update, scans both final files and records reported detections. If unavailable/permission denied, it records that limitation. It creates no exclusions and changes no protection settings. A clean result is specific to those hashes, definitions and time, not a guarantee against all antivirus detections.

For a suspected false positive: confirm the exact artifact SHA-256 and signature; reproduce and record the detection name, Defender versions and affected release; report through [ClearCe issues](https://github.com/usainc/ClearCe/issues). The developer submits the exact affected file to [Microsoft Security Intelligence](https://www.microsoft.com/wdsi/filesubmission) as software developer/incorrect detection and tracks the result. Do not change code merely to conceal behavior. No automatic sample upload or external submission occurs during this build workflow.

## Release behavior review

* No UPX, custom PE wrapper, binary decryptor or obfuscator. Normal NSIS archive compression only.
* No normal installation/startup dependency on PowerShell, cmd, batch, VBScript or Windows Script Host. Developer build/QA scripts are separate and not packaged.
* Recorded-result opening uses the locked Tauri opener's `open::that_detached` with `shellexecute-on-windows`, which calls native ShellExecuteExW. The dependency's separate PowerShell-based command API is not the selected Windows code path.
* No telemetry, analytics, updater polling, Run/RunOnce writes, startup shortcuts, services or scheduled tasks. Network access is limited to an explicit managed-engine action with one exact catalog URL, bounded GitHub redirects and a pinned SHA-256; there is no dynamic source search or arbitrary remote URL.
* Manual Real-ESRGAN selection remains a trusted directory, not an arbitrary EXE picker. Fixed required filenames/layout, local integrity manifest, direct executable path/argv, health check, timeout, bounded output and Windows Job Object containment remain. External native code is not a security sandbox; only import software you trust.
* Managed downloads stay in an owned temporary directory, reject oversized or unsafe archives, and extract only expected files. Imported/downloaded files are staged as data; health-check execution happens only after stable-path publication. Failed checks roll back the managed installation and never replace the separate manual engine. Inference intermediates contain images, not executables.
* The selected engine's short health checks and requested inference suppress a console window. These are bounded child processes for an existing product function, not hidden persistence or a background service.
* Tauri capabilities retain window controls/events, the local settings store and native file/directory dialog. No shell permission or broad filesystem scope was added. Validated image files receive individual asset access; CSP restricts UI connectivity to IPC. Two fixed-path language handoff commands expose no arbitrary file access.
* WebView2/Windows/driver components can perform their own platform networking, certificate validation or runtime updates. Source review of ClearCe is not a packet capture or a claim of zero OS traffic. The installer now requires preinstalled WebView2 and does not bootstrap it.
