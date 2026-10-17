param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("dev", "insider")]
    [string]$Channel
)

$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Push-Location $workspace
$exitCode = 0
$target = Join-Path $workspace "target"
$targetLimitBytes = 4GB

function Get-DirectoryBytes([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        return [int64]0
    }
    return [int64]((Get-ChildItem -LiteralPath $Path -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum)
}

function Clear-TargetIfOverLimit {
    if (-not (Test-Path -LiteralPath $target)) {
        return
    }

    $targetBytes = Get-DirectoryBytes $target
    if ($targetBytes -le $targetLimitBytes) {
        Write-Output ("target size {0:N2} GiB; limit 4 GiB." -f ($targetBytes / 1GB))
        return
    }

    $buildProcesses = Get-Process -Name cargo, rustc, tauri-app -ErrorAction SilentlyContinue
    if ($buildProcesses) {
        throw "target exceeds 4 GiB while a build process is running; cleanup aborted."
    }

    $workspaceRoot = (Resolve-Path $workspace).Path.TrimEnd("\")
    $targetPath = (Resolve-Path $target).Path.TrimEnd("\")
    if (-not $targetPath.StartsWith($workspaceRoot + "\", [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "refusing to clean a target path outside the workspace."
    }

    Remove-Item -LiteralPath $targetPath -Recurse -Force
    Write-Output ("cleaned target at {0:N2} GiB before {1} build." -f ($targetBytes / 1GB), $Channel)
}

function Stop-PackagedExecutable([string]$ExecutablePath) {
    $fullPath = [System.IO.Path]::GetFullPath($ExecutablePath)
    foreach ($process in (Get-Process -ErrorAction SilentlyContinue)) {
        try { $processPath = [System.IO.Path]::GetFullPath($process.Path) } catch { continue }
        if ([string]::Equals($processPath, $fullPath, [System.StringComparison]::OrdinalIgnoreCase)) {
            Stop-Process -Id $process.Id -Force
            if (-not $process.WaitForExit(5000)) { throw "ReadUX.exe 未退出：$fullPath" }
        }
    }
}

function Create-InsiderPackage {
    $binary = Join-Path $workspace "target\release\ReadUX.exe"
    if (-not (Test-Path -LiteralPath $binary)) { throw "发行程序缺失：$binary" }
    Stop-PackagedExecutable $binary
    & python -X utf8 "scripts\package_portable.py" --target windows-x64 --binary "target/release/ReadUX.exe" --output "packages" --name "ReadUX-insider-portable-win64"
    if ($LASTEXITCODE -ne 0) { throw "便携包生成失败：$LASTEXITCODE" }
}

function Prepare-InstallerResources {
    $minimalPython = Join-Path $workspace "experiments\ginza-minimal\Scripts\python.exe"
    $minimalManifest = Join-Path $workspace "experiments\ginza-minimal-manifest.json"
    if (-not (Test-Path -LiteralPath $minimalPython) -or -not (Test-Path -LiteralPath $minimalManifest)) {
        & python -X utf8 "scripts\build_ginza_minimal_env.py" --source "experiments\ginza311" --output "experiments\ginza-minimal"
        if ($LASTEXITCODE -ne 0) { throw "GiNZA 环境准备失败：$LASTEXITCODE" }
    }
    & python -X utf8 "scripts\prepare_installer_resources.py"
    if ($LASTEXITCODE -ne 0) { throw "安装资源准备失败：$LASTEXITCODE" }
    & npm.cmd run verify:release-resources -- --require-generated
    if ($LASTEXITCODE -ne 0) { throw "安装资源校验失败：$LASTEXITCODE" }
}

try {
    Clear-TargetIfOverLimit

    $env:CARGO_TARGET_DIR = Join-Path $workspace "target"

    if ($Channel -eq "dev") {
        $env:CARGO_INCREMENTAL = "1"
        Remove-Item Env:VITE_BUILD_CHANNEL -ErrorAction SilentlyContinue
        & npm.cmd run tauri -- dev --config src-tauri/tauri.dev.conf.json
    } else {
        $env:CARGO_INCREMENTAL = "0"
        $env:VITE_BUILD_CHANNEL = "insider"
        Prepare-InstallerResources
        & npm.cmd run tauri -- build --no-bundle
        if ($LASTEXITCODE -eq 0) { Create-InsiderPackage }
    }

    $exitCode = $LASTEXITCODE
} finally {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:CARGO_INCREMENTAL -ErrorAction SilentlyContinue
    Remove-Item Env:VITE_BUILD_CHANNEL -ErrorAction SilentlyContinue
    Pop-Location
}

exit $exitCode
