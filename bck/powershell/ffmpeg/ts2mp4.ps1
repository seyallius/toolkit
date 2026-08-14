param(
    [string]$mode = "all"
)

# Ensure output directory exists
$outDir = Join-Path (Get-Location) "out"
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

if ($mode -eq "all") {
    Get-ChildItem -Filter *.ts | ForEach-Object {
        $inFile  = $_.FullName
        $outFile = Join-Path $outDir ($_.BaseName + ".mp4")
        ffmpeg -i "$inFile" -c copy "$outFile"
    }
} else {
    Write-Host "Unknown mode: $mode"
}
