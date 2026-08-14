#!/usr/bin/env pwsh
# mkv2mp3.ps1 - Convert MKV files to MP3 with cover art

param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$Files = @(),
    
    [int]$CoverSize = 600,
    
    [int]$Bitrate = 320,
    
    [string]$OutputDir = "out",
    
    [switch]$Force,
    
    [switch]$ShowVerbose
)

# Function for verbose output
function Write-VerboseMsg {
    param([string]$Message)
    if ($ShowVerbose) {
        Write-Host "[VERBOSE] $Message" -ForegroundColor Cyan
    }
}

# Function to process a single file
function Convert-MkvToMp3 {
    param([string]$InputFile, [string]$OutputFile, [int]$CoverSize, [int]$Bitrate, [bool]$ShowVerbose)
    
    $fileName = [System.IO.Path]::GetFileNameWithoutExtension($InputFile)
    
    # Double-check output doesn't exist right before processing
	if ((Test-Path -LiteralPath $OutputFile) -and (-not $Force)) {
		Write-Host "SKIPPED (pre-check): $OutputFile already exists" -ForegroundColor Yellow
		return $false
	}
    
    # Create temp file for cover art
    $tempCover = [System.IO.Path]::GetTempFileName() + ".jpg"
    
    try {
        # Extract cover art (first frame at 1 second)
        Write-VerboseMsg "Extracting cover art from: $fileName"
        $extractArgs = @(
            "-ss", "00:00:01",
            "-i", $InputFile,
            "-frames:v", "1",
            "-vf", "scale=${CoverSize}:${CoverSize}:force_original_aspect_ratio=decrease,format=rgb24",
            "-y",
            $tempCover
        )
        
        $null = & ffmpeg $extractArgs -hide_banner -loglevel error 2>&1
        $extractSuccess = $LASTEXITCODE -eq 0 -and (Test-Path $tempCover) -and ((Get-Item $tempCover).Length -gt 0)
        
        if (-not $extractSuccess) {
            if ($ShowVerbose) {
                Write-Host "WARNING: Failed to extract cover art from $fileName, continuing without cover" -ForegroundColor Yellow
            }
            $tempCover = $null
        }
        
        # Convert audio to MP3 with cover art if available
        Write-VerboseMsg "Converting audio to MP3: $fileName"
        $convertArgs = @(
            "-threads", "auto",
            "-i", $InputFile
        )
        
        $hasCover = $tempCover -and (Test-Path $tempCover)
        if ($hasCover) {
            $convertArgs += @("-i", $tempCover)
        }
        
        $convertArgs += @(
            "-map", "0:a:0",
            "-c:a", "libmp3lame",
            "-b:a", "${Bitrate}k",
            "-id3v2_version", "3",
            "-write_id3v1", "1"
        )
        
        if ($hasCover) {
            $convertArgs += @("-map", "1:v:0", "-c:v", "copy", "-disposition:v:0", "attached_pic")
        }
        
        $convertArgs += @(
            "-y",
            $OutputFile
        )
        
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
    $mkvFiles = @()
    
    if ($Files.Count -eq 0) {
        # No arguments - process all MKV files in current directory
        Write-VerboseMsg "No files specified, scanning for all .mkv files"
        $mkvFiles = Get-ChildItem -Filter "*.mkv" -File
    } else {
        # Process specified files
        foreach ($file in $Files) {
            if (Test-Path $file) {
                if ($file -like "*.mkv") {
                    $mkvFiles += Get-Item $file
                } else {
                    Write-Host "WARNING: Skipping non-MKV file: $file" -ForegroundColor Yellow
                }
            } else {
                Write-Host "WARNING: File not found: $file" -ForegroundColor Yellow
            }
        }
    }
    
    if ($mkvFiles.Count -eq 0) {
        Write-Host "No MKV files found to process." -ForegroundColor Yellow
        exit 0
    }
    
    # Filter out files that already have output (unless Force is used)
	$filesToProcess = @()
	$skippedFiles = @()

	foreach ($file in $mkvFiles) {
		$outputFile = Join-Path (Resolve-Path $OutputDir) "$($file.BaseName).mp3"
		
		# Use -LiteralPath to handle special characters like [ and ]
		if ((Test-Path -LiteralPath $outputFile) -and (-not $Force)) {
			$skippedFiles += $file
		} else {
			$filesToProcess += @{
				InputFile = $file.FullName
				OutputFile = $outputFile
				Name = $file.Name
			}
		}
	}
    
    # Display summary before processing
    Write-Host "Found $($mkvFiles.Count) MKV file(s) total" -ForegroundColor Cyan
    Write-Host "Output directory: $OutputDir" -ForegroundColor Cyan
    Write-Host "Cover art size: ${CoverSize}x${CoverSize}" -ForegroundColor Cyan
    Write-Host "MP3 bitrate: ${Bitrate}kbps" -ForegroundColor Cyan
    
    if ($Force) {
        Write-Host "Force mode: ON (will overwrite existing files)" -ForegroundColor Magenta
    }
    
    if ($skippedFiles.Count -gt 0) {
        Write-Host "`nSkipping $($skippedFiles.Count) file(s) with existing output:" -ForegroundColor Yellow
        foreach ($fileName in $skippedFiles) {
            Write-Host "  - $fileName" -ForegroundColor Yellow
        }
    }
    
    if ($filesToProcess.Count -eq 0) {
        Write-Host "`nNo files to process. All MKV files have been converted!" -ForegroundColor Green
        exit 0
    }
    
    Write-Host "`nProcessing $($filesToProcess.Count) file(s):" -ForegroundColor Green
    Write-Host ("-" * 50)
    
    # Process each file
    $successCount = 0
    $failCount = 0
    
    foreach ($item in $filesToProcess) {
        $result = Convert-MkvToMp3 -InputFile $item.InputFile -OutputFile $item.OutputFile -CoverSize $CoverSize -Bitrate $Bitrate -ShowVerbose $ShowVerbose
        
        if ($result) {
            $successCount++
        } else {
            $failCount++
        }
    }
    
    Write-Host ("-" * 50)
    Write-Host "SUMMARY: $successCount succeeded, $($skippedFiles.Count) skipped, $failCount failed" -ForegroundColor Cyan
    
    if ($failCount -gt 0) {
        exit 1
    }
}
catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
    exit 1
}