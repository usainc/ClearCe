param([switch]$Signed, [switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $repo
$output = Join-Path $repo 'src-tauri/target/release'
$config = Join-Path $output 'signing.local.json'
$oldFlags = $env:RUSTFLAGS
$oldEncodedFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    # Remap this checkout and Cargo's absolute source locations in panic strings.
    $env:CARGO_ENCODED_RUSTFLAGS = @("--remap-path-prefix=$repo=ClearCe", "--remap-path-prefix=$env:USERPROFILE/.cargo=cargo", "--remap-path-prefix=$env:USERPROFILE\.cargo=cargo") -join [char]31
    $argsList = @('run','tauri','--','build')
    if ($Signed) {
        if ($env:CLEARCE_SIGNING_THUMBPRINT -notmatch '^[A-Fa-f0-9]{40}$') { throw 'Set CLEARCE_SIGNING_THUMBPRINT to a trusted code-signing certificate in CurrentUser/My.' }
        $timestamp = [uri]$env:CLEARCE_SIGNING_TIMESTAMP_URL
        if (!$timestamp.IsAbsoluteUri -or $timestamp.Scheme -ne 'https') { throw 'Set an HTTPS RFC3161 CLEARCE_SIGNING_TIMESTAMP_URL approved by your CA.' }
        New-Item -ItemType Directory -Force $output | Out-Null
        @{bundle=@{windows=@{certificateThumbprint=$env:CLEARCE_SIGNING_THUMBPRINT;digestAlgorithm='sha256';timestampUrl=$timestamp.AbsoluteUri;tsp=$true}}} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $config -Encoding utf8
        $argsList += @('--config',$config)
    }
    if (!$SkipBuild) {
        & npm @argsList
        if ($LASTEXITCODE) { throw 'Tauri release build failed' }
    }
    # Tauri restores its unpatched main EXE after bundling. Sign that standalone
    # artifact separately; packaged NSS-marked app/uninstaller are signed by Tauri.
    if ($Signed) {
        $tool = Get-Command signtool.exe -ErrorAction Stop
        & $tool.Source sign /sha1 $env:CLEARCE_SIGNING_THUMBPRINT /fd sha256 /tr $timestamp.AbsoluteUri /td sha256 (Join-Path $output 'clearce.exe')
        if ($LASTEXITCODE) { throw 'Standalone executable signing failed' }
    }
    $windowsPowerShell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
    if (!(Test-Path -LiteralPath $windowsPowerShell -PathType Leaf)) { throw 'Windows PowerShell is required for Authenticode verification.' }
    $verifyArgs = @('-NoProfile','-NonInteractive','-File',"$PSScriptRoot/verify-release.ps1")
    if ($Signed) { $verifyArgs += '-RequireSigned' }
    & $windowsPowerShell @verifyArgs
    if ($LASTEXITCODE) { throw 'Release verification failed' }
} finally {
    $env:RUSTFLAGS = $oldFlags
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedFlags
    if (Test-Path -LiteralPath $config) { Remove-Item -LiteralPath $config }
}
