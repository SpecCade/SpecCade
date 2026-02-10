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

$runId = Get-Date -Format "yyyyMMdd-HHmmss"
$logRoot = Join-Path $repoRoot ("target/runpack/music/" + $runId)
New-Item -ItemType Directory -Path $logRoot -Force | Out-Null

function Invoke-Step {
  param(
    [string]$Name,
    [string]$Command,
    [string]$LogFile
  )

  Write-Host ("==> " + $Name)
  Add-Content -Path $LogFile -Value ("`n==> " + $Name) -Encoding utf8
  Add-Content -Path $LogFile -Value ("$ " + $Command) -Encoding utf8
  cmd /c "$Command >> `"$LogFile`" 2>&1"
  if ($LASTEXITCODE -ne 0) {
    throw ("Step failed: " + $Name)
  }
  Write-Host ("    ok: " + $Name)
}

function Invoke-CheckpointCommit {
  param(
    [int]$CycleNumber,
    [string]$Prefix
  )

  git diff --cached --quiet
  $hasStagedChanges = ($LASTEXITCODE -ne 0)

  if (-not $hasStagedChanges) {
    Write-Host "==> no staged changes to checkpoint"
    return
  }

  $msg = "{0} (cycle {1}, {2})" -f $Prefix, $CycleNumber, (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
  Write-Host ("==> git commit -m """ + $msg + """")
  git commit -m $msg
  if ($LASTEXITCODE -ne 0) {
    throw "Checkpoint commit failed"
  }
}

$baseSteps = @(
  @{
    Name = "spec semantic validation tests"
    Command = "cargo test -p speccade-spec validation::tests::music_tests"
  },
  @{
    Name = "backend music tests"
    Command = "cargo test -p speccade-backend-music"
  },
  @{
    Name = "cli tests"
    Command = "cargo test -p speccade-cli"
  },
  @{
    Name = "integration music parity tests"
    Command = "cargo test -p speccade-tests --test music_parity"
  },
  @{
    Name = "integration compose tests"
    Command = "cargo test -p speccade-tests --test compose"
  }
)

if ($DeepGate) {
  $baseSteps += @{
    Name = "integration golden hash verification"
    Command = "cargo test -p speccade-tests --test golden_hash_verification -- --nocapture"
  }
}

Write-Host ("Runpack id: " + $runId)
Write-Host ("Logs: " + $logRoot)
Write-Host ("Cycles: " + $Cycles)
Write-Host ("DeepGate: " + $DeepGate.IsPresent)
Write-Host ("CheckpointStaged: " + $CheckpointStaged.IsPresent)

for ($cycle = 1; $cycle -le $Cycles; $cycle++) {
  $cycleLog = Join-Path $logRoot ("cycle-{0:D2}.log" -f $cycle)
  Write-Host ""
  Write-Host ("========== Cycle {0}/{1} ==========" -f $cycle, $Cycles)
  Add-Content -Path $cycleLog -Value ("Cycle {0} start: {1}" -f $cycle, (Get-Date -Format "o")) -Encoding utf8

  foreach ($step in $baseSteps) {
    Invoke-Step -Name $step.Name -Command $step.Command -LogFile $cycleLog
  }

  Add-Content -Path $cycleLog -Value "==> git status --short" -Encoding utf8
  $status = git status --short
  $status | Out-File -FilePath $cycleLog -Append -Encoding utf8
  $status | ForEach-Object { Write-Host $_ }

  if ($CheckpointStaged) {
    Invoke-CheckpointCommit -CycleNumber $cycle -Prefix $CheckpointPrefix
  }

  Add-Content -Path $cycleLog -Value ("Cycle {0} complete: {1}" -f $cycle, (Get-Date -Format "o")) -Encoding utf8

  if ($cycle -lt $Cycles -and $SleepSeconds -gt 0) {
    Write-Host ("Sleeping for {0}s before next cycle..." -f $SleepSeconds)
    Start-Sleep -Seconds $SleepSeconds
  }
}

Write-Host ""
Write-Host "Runpack completed."
Write-Host ("Logs written to: " + $logRoot)
