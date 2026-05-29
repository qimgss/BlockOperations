# Android NDK 交叉编译脚本 (Windows)
# 用法: .\cross-compile.ps1

param(
    [string]$NdkPath = "C:\Android\NDK",
    [string]$TargetApi = "28",
    [string]$TargetArch = "aarch64-linux-android"
)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  blkops Android build script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 检查 NDK 路径
$NdkPrebuilt = Join-Path $NdkPath "toolchains\llvm\prebuilt\windows-x86_64"
$ClangPath = Join-Path $NdkPrebuilt "bin\${TargetArch}${TargetApi}-clang++.cmd"

if (-not (Test-Path $ClangPath)) {
    Write-Host "error: Cannot find the compiler: $ClangPath" -ForegroundColor Red
    Write-Host "Please confirm whether the NDK path is correct.: $NdkPath" -ForegroundColor Yellow
    exit 1
}

Write-Host "NDK Path: $NdkPath" -ForegroundColor Green
Write-Host "Compiler: $ClangPath" -ForegroundColor Green
Write-Host "Target API: $TargetApi" -ForegroundColor Green
Write-Host "Target Architecture: $TargetArch" -ForegroundColor Green

# Set environment variable
$env:CC = $ClangPath
$env:CXX = $ClangPath
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $ClangPath
$env:ANDROID_NDK_HOME = $NdkPath

Write-Host "`nThe environment variable has been set:" -ForegroundColor Yellow
Write-Host "  CC=$env:CC"
Write-Host "  CXX=$env:CXX"
Write-Host "  CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"

# 检查 Rust 目标
Write-Host "`nCheck if Rust target is installed..." -ForegroundColor Yellow
$targets = rustup target list | Select-String "aarch64-linux-android"

if (-not $targets) {
    Write-Host "Add aarch64-linux-android target..." -ForegroundColor Yellow
    rustup target add aarch64-linux-android
} else {
    Write-Host "aarch64-linux-android target is installed" -ForegroundColor Green
}

# Clean and comiling
Write-Host "`nStart compiling..." -ForegroundColor Yellow

# Clean old builds
if (Test-Path "target") {
    Write-Host "Clean old builds..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force "target" -ErrorAction SilentlyContinue
}

# Compiling release version
Write-Host "Compiling release version..." -ForegroundColor Yellow
cargo build --release --target aarch64-linux-android

if ($LASTEXITCODE -eq 0) {
    $outputPath = "target\aarch64-linux-android\release\blkops"
    if (Test-Path $outputPath) {
        Write-Host "`n✅ Compiling success!" -ForegroundColor Green
        Write-Host "Output file: $outputPath" -ForegroundColor Green
        
        $fileInfo = Get-Item $outputPath
        Write-Host "File size: $([math]::Round($fileInfo.Length / 1KB, 2)) KB" -ForegroundColor Cyan
        
        $destPath = ".\blkops-android-arm64"
        Copy-Item $outputPath $destPath -Force
        Write-Host "Copied as: $destPath" -ForegroundColor Green
    } else {
        Write-Host "`n❌ cannot find output file!" -ForegroundColor Red
    }
} else {
    Write-Host "`n❌ Compiling failed!" -ForegroundColor Red
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Compiling end" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan