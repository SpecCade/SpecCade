<#
.SYNOPSIS
    Generate all speccade golden specs

.DESCRIPTION
    Executes all golden specs using the CLI's generate-all command.
    Single-file outputs go directly to category folder.
    Multi-file outputs get their own subdirectory.

.PARAMETER OutputDir
    Output directory for generated assets (default: ./test-outputs)

.PARAMETER IncludeBlender
    Include Blender-based assets (static_mesh, skeletal_mesh, skeletal_animation)

.PARAMETER Release
    Build and run in release mode (faster generation)

.EXAMPLE
    .\generate_all.ps1

.EXAMPLE
    .\generate_all.ps1 -OutputDir .\my-outputs

.EXAMPLE
    .\generate_all.ps1 -IncludeBlender -Release
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$OutputDir = ".\test-outputs",

    [switch]$IncludeBlender,

    [switch]$Release
)

$ErrorActionPreference = "Stop"

# Paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$SpeccadeRoot = Split-Path -Parent $ScriptDir
$SpecDir = Join-Path $SpeccadeRoot "golden\speccade\specs"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$ForegroundColor = "White"
    )
    Write-Host $Message -ForegroundColor $ForegroundColor
}

Write-ColorOutput "======================================" -ForegroundColor Cyan
Write-ColorOutput "  SpecCade Golden Spec Generator" -ForegroundColor Cyan
Write-ColorOutput "======================================" -ForegroundColor Cyan
Write-Host ""
Write-ColorOutput "Spec directory: $SpecDir" -ForegroundColor Blue
Write-ColorOutput "Output directory: $OutputDir" -ForegroundColor Blue
Write-ColorOutput "Include Blender: $IncludeBlender" -ForegroundColor Blue
Write-ColorOutput "Release mode: $Release" -ForegroundColor Blue
Write-Host ""

# Build speccade-cli
$buildMode = if ($Release) { "--release" } else { "" }
$targetDir = if ($Release) { "release" } else { "debug" }

Write-ColorOutput "Building speccade-cli ($targetDir)..." -ForegroundColor Yellow
Push-Location $SpeccadeRoot
try {
    if ($Release) {
        $buildOutput = cargo build -p speccade-cli --release 2>&1
    } else {
        $buildOutput = cargo build -p speccade-cli 2>&1
    }
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "Build failed" -ForegroundColor Red
        Write-Host $buildOutput
        exit 1
    }
    Write-ColorOutput "Build successful" -ForegroundColor Green
}
finally {
    Pop-Location
}
Write-Host ""

# Find the binary
$SpeccadeBin = Join-Path $SpeccadeRoot "target\$targetDir\speccade.exe"
if (-not (Test-Path $SpeccadeBin)) {
    $SpeccadeBin = Join-Path $SpeccadeRoot "target\$targetDir\speccade"
}

if (-not (Test-Path $SpeccadeBin)) {
    Write-ColorOutput "Could not find speccade binary at $SpeccadeBin" -ForegroundColor Red
    exit 1
}

Write-ColorOutput "Using binary: $SpeccadeBin" -ForegroundColor Blue
Write-Host ""

# Build CLI arguments
$cliArgs = @("generate-all", "--spec-dir", $SpecDir, "--out-root", $OutputDir)

if ($IncludeBlender) {
    $cliArgs += "--include-blender"
}

Write-ColorOutput "Running: speccade $($cliArgs -join ' ')" -ForegroundColor Yellow
Write-Host ""

# Run the generate-all command
& $SpeccadeBin @cliArgs

$exitCode = $LASTEXITCODE

Write-Host ""
if ($exitCode -eq 0) {
    Write-ColorOutput "Generation completed successfully!" -ForegroundColor Green
} else {
    Write-ColorOutput "Generation completed with errors (exit code: $exitCode)" -ForegroundColor Yellow
}

exit $exitCode
