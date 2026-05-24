#!/usr/bin/env pwsh
# mp32mp4.ps1 - Convert MP3 files to MP4 with cover art as video

param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$Files = @(),
    
    [int]$Bitrate = 320,
    
    [string]$OutputDir = "out",
    
    [switch]$Force,
    
    [switch]$ShowVerbose,
    
    [switch]$NoCoverFallback
)

# Function for verbose output
function Write-VerboseMsg {
    param([string]$Message)
    if ($ShowVerbose) {
        Write-Host "[VERBOSE] $Message" -ForegroundColor Cyan
    }
}

# Function to process a single file
function Convert-Mp3ToMp4 {
    param([string]$InputFile, [string]$OutputFile, [int]$Bitrate, [bool]$NoCoverFallback, [bool]$ShowVerbose)
    
    $fileName = [System.IO.Path]::GetFileNameWithoutExtension($InputFile)
    
    # Create temp file for cover art
    $tempCover = [System.IO.Path]::GetTempFileName() + ".jpg"
    
    try {
        # Extract cover art
        Write-VerboseMsg "Extracting cover art from: $fileName"
        $extractArgs = @(
            "-i", $InputFile,
            "-an",
            "-vcodec", "copy",
            "-y",
            $tempCover
        )
        
        $null = & ffmpeg $extractArgs -hide_banner -loglevel error 2>&1
        $extractSuccess = $LASTEXITCODE -eq 0 -and (Test-Path $tempCover) -and ((Get-Item $tempCover).Length -gt 0)
        
        if (-not $extractSuccess) {
            if ($NoCoverFallback) {
                Write-Host "WARNING: No cover art found in $fileName, creating with black video" -ForegroundColor Yellow
                $tempCover = $null
            } else {
                Write-Host "SKIPPED: No cover art in $fileName" -ForegroundColor Yellow
                return $false
            }
        }
        
        # Convert to MP4 with cover art as video
        Write-VerboseMsg "Converting to MP4: $fileName"
        
        if ($tempCover -and (Test-Path $tempCover)) {
            $convertArgs = @(
                "-loop", "1",
                "-i", $tempCover,
                "-i", $InputFile,
                "-threads", "0",
                "-filter_threads", "0",
                "-filter_complex_threads", "0",
                "-c:v", "libx264",
                "-preset", "ultrafast",
                "-tune", "stillimage",
                "-pix_fmt", "yuv420p",
                "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2,setsar=1,format=yuv420p",
                "-c:a", "aac",
                "-b:a", "${Bitrate}k",
                "-shortest",
                "-movflags", "+faststart",
                "-y",
                $OutputFile
            )
        } else {
            # Fallback: generate black video
            $convertArgs = @(
                "-f", "lavfi",
                "-i", "color=c=black:s=1280x720:r=1",
                "-i", $InputFile,
                "-threads", "0",
                "-filter_threads", "0",
                "-filter_complex_threads", "0",
                "-c:v", "libx264",
                "-preset", "ultrafast",
                "-tune", "stillimage",
                "-pix_fmt", "yuv420p",
                "-c:a", "aac",
                "-b:a", "${Bitrate}k",
                "-shortest",
                "-movflags", "+faststart",
                "-y",
                $OutputFile
            )
        }
        
        $null = & ffmpeg $convertArgs -hide_banner -loglevel error
        
        if ($LASTEXITCODE -eq 0 -and (Test-Path -LiteralPath $OutputFile)) {
            Write-Host "SUCCESS: $OutputFile" -ForegroundColor Green
            return $true
        } else {
            Write-Host "FAILED: $fileName" -ForegroundColor Red
            if (Test-Path -LiteralPath $OutputFile) {
                Remove-Item -LiteralPath $OutputFile -Force -ErrorAction SilentlyContinue
            }
            return $false
        }
    }
    finally {
        # Clean up temp file
        if ($tempCover -and (Test-Path $tempCover)) {
            Remove-Item $tempCover -Force -ErrorAction SilentlyContinue
        }
    }
}

# Main execution
try {
    # Create output directory if it doesn't exist
    if (-not (Test-Path $OutputDir)) {
        Write-VerboseMsg "Creating output directory: $OutputDir"
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    }
    
    # Determine which files to process
    $mp3Files = @()
    
    if ($Files.Count -eq 0) {
        # No arguments - process all MP3 files in current directory
        Write-VerboseMsg "No files specified, scanning for all .mp3 files"
        $mp3Files = Get-ChildItem -Filter "*.mp3" -File
    } else {
        # Process specified files
        foreach ($file in $Files) {
            if (Test-Path $file) {
                if ($file -like "*.mp3") {
                    $mp3Files += Get-Item $file
                } else {
                    Write-Host "WARNING: Skipping non-MP3 file: $file" -ForegroundColor Yellow
                }
            } else {
                Write-Host "WARNING: File not found: $file" -ForegroundColor Yellow
            }
        }
    }
    
    if ($mp3Files.Count -eq 0) {
        Write-Host "No MP3 files found to process." -ForegroundColor Yellow
        exit 0
    }
    
    # Filter out files that already have output (unless Force is used)
    $filesToProcess = @()
    $skippedFiles = @()
    
    foreach ($file in $mp3Files) {
        $outputFile = Join-Path (Resolve-Path $OutputDir) "$($file.BaseName).mp4"
        
        # Use -LiteralPath to handle special characters like [ and ]
        if ((Test-Path -LiteralPath $outputFile) -and (-not $Force)) {
            $skippedFiles += @{
                File = $file
                Reason = "Output already exists"
            }
        } else {
            $filesToProcess += @{
                InputFile = $file.FullName
                OutputFile = $outputFile
                Name = $file.Name
            }
        }
    }
    
    # Display summary before processing
    Write-Host "Found $($mp3Files.Count) MP3 file(s) total" -ForegroundColor Cyan
    Write-Host "Output directory: $OutputDir" -ForegroundColor Cyan
    Write-Host "MP4 audio bitrate: ${Bitrate}kbps" -ForegroundColor Cyan
    Write-Host "Skip files without cover: $(if($NoCoverFallback){'No (will use black video)'}else{'Yes'})" -ForegroundColor Cyan
    
    if ($Force) {
        Write-Host "Force mode: ON (will overwrite existing files)" -ForegroundColor Magenta
    }
    
    if ($NoCoverFallback) {
        Write-Host "NoCoverFallback: ON (will process files even without cover art)" -ForegroundColor Magenta
    }
    
    if ($skippedFiles.Count -gt 0) {
        Write-Host "`nSkipping $($skippedFiles.Count) file(s) with existing output:" -ForegroundColor Yellow
        foreach ($item in $skippedFiles) {
            Write-Host "  - $($item.File.Name) ($($item.Reason))" -ForegroundColor Yellow
        }
    }
    
    if ($filesToProcess.Count -eq 0) {
        Write-Host "`nNo files to process. All MP3 files have been converted!" -ForegroundColor Green
        exit 0
    }
    
    Write-Host "`nProcessing $($filesToProcess.Count) file(s):" -ForegroundColor Green
    Write-Host ("-" * 50)
    
    # Process each file
    $successCount = 0
    $failCount = 0
    $skippedNoCoverCount = 0
    
    foreach ($item in $filesToProcess) {
        $result = Convert-Mp3ToMp4 -InputFile $item.InputFile -OutputFile $item.OutputFile -Bitrate $Bitrate -NoCoverFallback $NoCoverFallback -ShowVerbose $ShowVerbose
        
        if ($result) {
            $successCount++
        } elseif ($LASTEXITCODE -eq 0 -and -not $NoCoverFallback) {
            # File was skipped due to no cover art
            $skippedNoCoverCount++
        } else {
            $failCount++
        }
    }
    
    Write-Host ("-" * 50)
    Write-Host "SUMMARY: $successCount succeeded, $($skippedFiles.Count) skipped (exists), $skippedNoCoverCount skipped (no cover), $failCount failed" -ForegroundColor Cyan
    
    if ($failCount -gt 0) {
        exit 1
    }
}
catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
    exit 1
}