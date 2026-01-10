<#
.SYNOPSIS
    Generate all speccade golden specs

.DESCRIPTION
    Executes all golden specs and organizes outputs for validation and visualization.

.PARAMETER OutputDir
    Output directory for generated assets (default: ./test-outputs)

.PARAMETER IncludeBlender
    Include Blender-based assets (static_mesh, skeletal_mesh, skeletal_animation)

.PARAMETER Verbose
    Show verbose output

.EXAMPLE
    .\generate_all.ps1

.EXAMPLE
    .\generate_all.ps1 -OutputDir .\my-outputs

.EXAMPLE
    .\generate_all.ps1 -IncludeBlender
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$OutputDir = ".\test-outputs",

    [switch]$IncludeBlender,

    [switch]$VerboseOutput
)

$ErrorActionPreference = "Stop"

# Paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$SpeccadeRoot = Split-Path -Parent $ScriptDir
$SpecDir = Join-Path $SpeccadeRoot "golden\speccade\specs"

# Asset types to process
$AssetTypes = @("audio_sfx", "audio_instrument", "music", "texture_2d", "normal_map")

if ($IncludeBlender) {
    $AssetTypes += @("static_mesh", "skeletal_mesh", "skeletal_animation")
}

# Statistics
$TotalSpecs = 0
$SuccessCount = 0
$FailureCount = 0
$SkippedCount = 0
$FailedSpecs = @()
$SuccessHashes = @()

# Start timer
$StartTime = Get-Date

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
Write-Host ""

# Create output directories
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
foreach ($assetType in $AssetTypes) {
    $typeDir = Join-Path $OutputDir $assetType
    New-Item -ItemType Directory -Force -Path $typeDir | Out-Null
}

# Build speccade-cli first
Write-ColorOutput "Building speccade-cli..." -ForegroundColor Yellow
Push-Location $SpeccadeRoot
try {
    $buildOutput = cargo build -p speccade-cli --release 2>&1
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
$SpeccadeBin = Join-Path $SpeccadeRoot "target\release\speccade.exe"
if (-not (Test-Path $SpeccadeBin)) {
    $SpeccadeBin = Join-Path $SpeccadeRoot "target\release\speccade"
}

if (-not (Test-Path $SpeccadeBin)) {
    Write-ColorOutput "Could not find speccade binary" -ForegroundColor Red
    exit 1
}

# Process each asset type
foreach ($assetType in $AssetTypes) {
    $AssetDir = Join-Path $SpecDir $assetType

    if (-not (Test-Path $AssetDir)) {
        Write-ColorOutput "Skipping $assetType (directory not found)" -ForegroundColor Yellow
        continue
    }

    Write-ColorOutput "Processing $assetType..." -ForegroundColor Cyan

    $specFiles = Get-ChildItem -Path $AssetDir -Filter "*.json" -ErrorAction SilentlyContinue

    foreach ($spec in $specFiles) {
        $TotalSpecs++
        $name = [System.IO.Path]::GetFileNameWithoutExtension($spec.Name)
        $outDir = Join-Path $OutputDir "$assetType\$name"

        if ($VerboseOutput) {
            Write-Host "  Generating: $name" -ForegroundColor Blue
        }

        New-Item -ItemType Directory -Force -Path $outDir | Out-Null

        $logFile = Join-Path $outDir "generation.log"

        # Run generation
        try {
            $result = & $SpeccadeBin generate --spec $spec.FullName --out-root $outDir 2>&1
            $result | Out-File -FilePath $logFile -Encoding utf8

            if ($LASTEXITCODE -eq 0) {
                $SuccessCount++

                # Extract hash from report if available
                $reportFiles = Get-ChildItem -Path $outDir -Filter "*.report.json" -ErrorAction SilentlyContinue
                if ($reportFiles) {
                    $reportContent = Get-Content $reportFiles[0].FullName -Raw | ConvertFrom-Json -ErrorAction SilentlyContinue
                    if ($reportContent -and $reportContent.spec_hash) {
                        $SuccessHashes += "$assetType/$name`: $($reportContent.spec_hash)"
                    }
                }

                if ($VerboseOutput) {
                    Write-ColorOutput "    SUCCESS" -ForegroundColor Green
                }
                else {
                    Write-Host "." -NoNewline -ForegroundColor Green
                }
            }
            else {
                $FailureCount++
                $FailedSpecs += "$assetType/$name"

                if ($VerboseOutput) {
                    Write-ColorOutput "    FAILED" -ForegroundColor Red
                    $result | Select-Object -First 10 | ForEach-Object { Write-Host "      $_" }
                }
                else {
                    Write-Host "x" -NoNewline -ForegroundColor Red
                }
            }
        }
        catch {
            $FailureCount++
            $FailedSpecs += "$assetType/$name"
            $_.Exception.Message | Out-File -FilePath $logFile -Encoding utf8 -Append

            if ($VerboseOutput) {
                Write-ColorOutput "    FAILED: $($_.Exception.Message)" -ForegroundColor Red
            }
            else {
                Write-Host "x" -NoNewline -ForegroundColor Red
            }
        }
    }

    if (-not $VerboseOutput) {
        Write-Host ""  # Newline after dots
    }
}

# Calculate elapsed time
$EndTime = Get-Date
$Elapsed = ($EndTime - $StartTime).TotalSeconds

# Print summary
Write-Host ""
Write-ColorOutput "======================================" -ForegroundColor Cyan
Write-ColorOutput "  Generation Summary" -ForegroundColor Cyan
Write-ColorOutput "======================================" -ForegroundColor Cyan
Write-Host ""
Write-ColorOutput "Total specs processed: $TotalSpecs" -ForegroundColor Blue
Write-ColorOutput "Successful: $SuccessCount" -ForegroundColor Green
Write-ColorOutput "Failed: $FailureCount" -ForegroundColor Red
Write-ColorOutput "Skipped: $SkippedCount" -ForegroundColor Yellow
Write-ColorOutput "Total runtime: $([math]::Round($Elapsed, 2))s" -ForegroundColor Blue
Write-Host ""

# List successful specs with hashes
if ($SuccessHashes.Count -gt 0) {
    Write-ColorOutput "Generated specs with BLAKE3 hashes:" -ForegroundColor Green
    foreach ($entry in $SuccessHashes) {
        Write-Host "  $entry"
    }
    Write-Host ""
}

# List failed specs
if ($FailedSpecs.Count -gt 0) {
    Write-ColorOutput "Failed specs:" -ForegroundColor Red
    foreach ($spec in $FailedSpecs) {
        Write-Host "  - $spec"
    }
    Write-Host ""
}

Write-ColorOutput "Outputs saved to: $OutputDir" -ForegroundColor Blue

# Write summary report
$SummaryFile = Join-Path $OutputDir "generation_summary.json"
$Summary = @{
    timestamp        = (Get-Date -Format "o")
    total_specs      = $TotalSpecs
    successful       = $SuccessCount
    failed           = $FailureCount
    skipped          = $SkippedCount
    runtime_seconds  = [math]::Round($Elapsed, 2)
    include_blender  = $IncludeBlender.IsPresent
    failed_specs     = $FailedSpecs
}
$Summary | ConvertTo-Json -Depth 10 | Out-File -FilePath $SummaryFile -Encoding utf8

Write-ColorOutput "Summary report: $SummaryFile" -ForegroundColor Blue

# Exit with error if any specs failed
if ($FailureCount -gt 0) {
    exit 1
}

exit 0
