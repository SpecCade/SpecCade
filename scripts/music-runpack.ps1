param(
  [int]$Cycles = 1,
  [int]$SleepSeconds = 0,
  [switch]$DeepGate,
  [switch]$CheckpointStaged,
  [string]$CheckpointPrefix = "music-runpack checkpoint"
)

$ErrorActionPreference = "Stop"

if ($Cycles -lt 1) {
  throw "Cycles must be >= 1"
}

if ($SleepSeconds -lt 0) {
  throw "SleepSeconds must be >= 0"
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
Set-Location $repoRoot

$pipelineArgs = @(
  "run",
  "-p",
  "speccade-cli",
  "--",
  "pipeline",
  "--profile",
  "music",
  "--spec-dir",
  "specs/music",
  "--cycles",
  $Cycles.ToString(),
  "--sleep-seconds",
  $SleepSeconds.ToString()
)

if ($DeepGate) {
  $pipelineArgs += "--deep-gate"
}

if ($CheckpointStaged) {
  $pipelineArgs += "--checkpoint-staged"
  if ($CheckpointPrefix) {
    $pipelineArgs += "--checkpoint-prefix"
    $pipelineArgs += $CheckpointPrefix
  }
}

Write-Host "Music runpack now delegates to the CLI pipeline command."
Write-Host ("Cycles: " + $Cycles)
Write-Host ("DeepGate: " + $DeepGate.IsPresent)
Write-Host ("CheckpointStaged: " + $CheckpointStaged.IsPresent)
Write-Host ("Running: cargo " + ($pipelineArgs -join " "))
Write-Host ""

& cargo @pipelineArgs
exit $LASTEXITCODE
