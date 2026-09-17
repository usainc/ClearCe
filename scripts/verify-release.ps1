param([switch]$RequireSigned, [switch]$DefenderScan)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$version = (Get-Content -LiteralPath "$repo/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json).version
$release = Join-Path $repo 'src-tauri/target/release'
$installer = "ClearCe_${version}_x64-setup.exe"
$artifacts = @((Join-Path $release 'clearce.exe'),(Join-Path $release "bundle/nsis/$installer"))
$entries = @()
foreach ($path in $artifacts) {
    $item = Get-Item -LiteralPath $path
    $signature = Get-AuthenticodeSignature -LiteralPath $path
    if ($RequireSigned -and ($signature.Status -ne 'Valid' -or !$signature.TimeStamperCertificate)) { throw "Valid timestamped signature required: $($item.Name)" }
    $bytes = [IO.File]::ReadAllBytes($path)
    foreach ($encoding in @([Text.Encoding]::UTF8,[Text.Encoding]::Unicode)) {
        foreach ($sourcePath in @($repo, $repo.Replace('\','/'))) {
            if ($encoding.GetString($bytes).Contains($sourcePath)) { throw "Developer checkout path embedded in $($item.Name)" }
        }
    }
    $entries += [ordered]@{file=$item.Name;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower();signatureStatus="$($signature.Status)";signerSubject=$signature.SignerCertificate.Subject;timestampSubject=$signature.TimeStamperCertificate.Subject}
}
$scan = [ordered]@{status='not executed';reason='Opt in with -DefenderScan'}
if ($DefenderScan) {
    try {
        $status = Get-MpComputerStatus
        $scan = [ordered]@{status='not executed';engine=$status.AMEngineVersion;definitions=$status.AntivirusSignatureVersion;enabled=$status.AntivirusEnabled}
        if (!$status.AntivirusEnabled) { throw 'Defender antivirus unavailable or disabled by system policy' }
        try { Update-MpSignature -ErrorAction Stop; $scan.definitions=(Get-MpComputerStatus).AntivirusSignatureVersion } catch { $scan.update='unavailable' }
        $start = Get-Date
        foreach ($path in $artifacts) { Start-MpScan -ScanType CustomScan -ScanPath $path -ErrorAction Stop }
        $detected = @(Get-MpThreatDetection | Where-Object { $_.InitialDetectionTime -ge $start -and ($_.Resources | Where-Object { $resource=$_; $artifacts | Where-Object { $resource.Contains($_) } }) })
        $scan.status = if ($detected.Count) {'detection reported'} else {'scan completed; no matching detections reported'}
        $scan.threatIds = @($detected | Select-Object -ExpandProperty ThreatID)
    } catch { $scan.reason='Defender interface unavailable or permission denied; no protection settings changed' }
}
$manifest = [ordered]@{
    product='ClearCe'
    version=$version
    platform='Windows'
    architecture='x64'
    installer=$installer
    sha256=$entries[1].sha256
    signed=(@($entries | Where-Object signatureStatus -ne 'Valid').Count -eq 0)
    engineBundled=$false
    managedEngineDownload=$true
    languages=@('en','tr')
    artifacts=$entries
    defender=$scan
}
$dest = Join-Path $release 'release-integrity'
New-Item -ItemType Directory -Force $dest | Out-Null
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "$dest/release-manifest.json" -Encoding utf8
$entries | ForEach-Object { "$($_.sha256)  $($_.file)" } | Set-Content -LiteralPath "$dest/SHA256SUMS.txt" -Encoding ascii
$manifest | ConvertTo-Json -Depth 8
