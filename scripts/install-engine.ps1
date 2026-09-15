param(
    [Parameter(Mandatory = $true)][string]$SourceDirectory,
    [string]$Version = 'user-provided (version not reported)'
)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path -LiteralPath $SourceDirectory).Path
$parent = Join-Path $env:LOCALAPPDATA 'dev.usaince.enhancece\engines'
$destination = Join-Path $parent 'realesrgan'
if (Test-Path -LiteralPath $destination) { throw "Engine destination already exists: $destination. Preserve it; this installer does not overwrite installations." }
$required = @('realesrgan-ncnn-vulkan.exe', 'models/realesrgan-x4plus.param', 'models/realesrgan-x4plus.bin')
foreach ($relative in $required) {
    if (!(Test-Path -LiteralPath (Join-Path $source $relative) -PathType Leaf)) { throw "Required engine file missing: $relative" }
}
$files = @($required)
foreach ($dll in @('vcomp140.dll', 'vcomp140d.dll')) {
    if (Test-Path -LiteralPath (Join-Path $source $dll) -PathType Leaf) { $files += $dll }
}
New-Item -ItemType Directory -Path $parent -Force | Out-Null
$staging = Join-Path $parent ('install-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path (Join-Path $staging 'models') -Force | Out-Null
try {
    $hashes = [ordered]@{}
    foreach ($relative in $files) {
        $target = Join-Path $staging $relative
        Copy-Item -LiteralPath (Join-Path $source $relative) -Destination $target
        $hashes[$relative] = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    $manifest = [ordered]@{ version = $Version; files = $hashes } | ConvertTo-Json -Depth 4
    [IO.File]::WriteAllText((Join-Path $staging 'manifest.json'), $manifest, [Text.UTF8Encoding]::new($false))
    # Both paths are explicit children of the managed engines directory.
    if ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($staging)) -ne [IO.Path]::GetFullPath($parent)) { throw 'Invalid staging path' }
    Move-Item -LiteralPath $staging -Destination $destination
    Write-Output "Installed engine: $destination"
} catch {
    # Leave an incomplete staging directory for inspection; never delete user input.
    throw
}
