$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
Set-Location $repoRoot

Write-Host "==> cargo fmt --check"
cargo fmt --all -- --check

Write-Host "==> cargo test (workspace, all features)"
if (Test-Path (Join-Path $repoRoot "editor\dist")) {
  cargo test --workspace --all-features
} else {
  Write-Host "==> editor/dist missing; excluding speccade-editor-app"
  cargo test --workspace --all-features --exclude speccade-editor-app
}

Write-Host "==> cargo clippy"
if (Test-Path (Join-Path $repoRoot "editor\dist")) {
  cargo clippy --workspace --all-features
} else {
  Write-Host "==> editor/dist missing; excluding speccade-editor-app"
  cargo clippy --workspace --all-features --exclude speccade-editor-app
}

Write-Host "==> cargo doc"
if (Test-Path (Join-Path $repoRoot "editor\dist")) {
  cargo doc --workspace --no-deps --all-features
} else {
  Write-Host "==> editor/dist missing; excluding speccade-editor-app"
  cargo doc --workspace --no-deps --all-features --exclude speccade-editor-app
}

Write-Host "==> coverage generate"
cargo run -p speccade-cli -- coverage generate

Write-Host "==> feature coverage test"
cargo test -p speccade-tests --test feature_coverage -- --nocapture

Write-Host "==> golden: build release CLI"
cargo build --release -p speccade-cli

$goldenDir = Join-Path $repoRoot "golden\speccade\specs"
if (Test-Path $goldenDir) {
  Write-Host "==> golden: validate specs"
  $exe = Join-Path $repoRoot "target\release\speccade.exe"
  # Only validate Spec JSON fixtures; ignore report JSONs and other artifacts.
  Get-ChildItem -Path $goldenDir -Recurse -Filter "*.spec.json" -File | ForEach-Object {
    Write-Host ("Validating: " + $_.FullName)
    & $exe validate --spec $_.FullName
    if ($LASTEXITCODE -ne 0) {
      throw "golden spec validation failed: $($_.FullName)"
    }
  }
} else {
  Write-Host "==> golden: no specs directory found, skipping"
}

Write-Host "==> golden: hash verification test"
cargo test -p speccade-tests --test golden_hash_verification -- --nocapture

Write-Host "OK"
