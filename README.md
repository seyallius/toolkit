# 🛠️ toolkit

> A unified FFmpeg workflow CLI for common media tasks.

A fast, single-binary replacement for mixed PowerShell/Go media scripts. Built with Rust, it provides safe,
cross-platform media conversion workflows with progress feedback and batch processing capabilities.

## ✨ Features

- **Unified CLI**: One binary for all media workflows (`ts2mp4`, `mkv2mp3`, `mp32mp4`, `vidwrap`)
- **Batch Processing**: Automatic directory scanning with skip/overwrite logic
- **Safe Defaults**: Prevents accidental data loss with explicit `--force` flags
- **Rich UX**: Spinners, progress bars, and colored output (disable with `--no-color`)
- **Testable Core**: Dependency-injected FFmpeg runner for deterministic testing
- **Zero Runtime Dependencies**: Single static binary, no PowerShell or Go required

## 📦 Installation

### From crates.io (Recommended)

```bash
cargo install toolkitrs
```

### From Source

```bash
git clone https://github.com/seyallius/toolkit.git
cd toolkit
cargo install --path .
```

### Prerequisites

- [FFmpeg](https://ffmpeg.org/download.html) must be installed and available in your PATH
- Optional: Use `--ffmpeg-path <PATH>` to specify a custom FFmpeg location

## 🚀 Usage

```bash
toolkitrs [OPTIONS] <COMMAND>
```

### Global Options

| Flag                    | Description                                           |
|-------------------------|-------------------------------------------------------|
| `--verbose`             | Print FFmpeg commands and diagnostic output to stderr |
| `--no-color`            | Disable ANSI color codes and styling                  |
| `--ffmpeg-path <PATH>`  | Explicit path to ffmpeg executable                    |
| `--ffprobe-path <PATH>` | Explicit path to ffprobe binary (reserved)            |
| `-h, --help`            | Print help information                                |
| `-V, --version`         | Print version information                             |

### Commands

#### `ts2mp4` — Remux TS to MP4

Convert Transport Stream files to MP4 via stream copy (no re-encoding).

```bash
# Convert all .ts files in current directory
toolkitrs ts2mp4

# Convert specific files to custom output directory
toolkitrs ts2mp4 --output-dir ./converted video1.ts video2.ts

# Overwrite existing outputs
toolkitrs ts2mp4 --force
```

#### `mkv2mp3` — Extract Audio from MKV

Convert MKV files to MP3 with optional cover art extracted from video frames.

```bash
# Default: 320kbps, 600px cover art
toolkitrs mkv2mp3

# Custom bitrate and cover size
toolkitrs mkv2mp3 --bitrate 256 --cover-size 800 movie.mkv

# Force overwrite existing MP3s
toolkitrs mkv2mp3 --force
```

#### `mp32mp4` — Create Video from MP3

Convert MP3 files to MP4 videos using embedded cover art as the video track.

```bash
# Use embedded cover art (black video fallback if missing)
toolkitrs mp32mp4

# Skip files without embedded cover art
toolkitrs mp32mp4 --no-cover-fallback

# Custom audio bitrate
toolkitrs mp32mp4 --bitrate 256 song.mp3
```

#### `vidwrap` — Wrap Video with Thumbnail

Combine a video with a companion image to create a new video with an embedded thumbnail. Supports interactive
post-processing.

```bash
# Requires a same-basename image (e.g., video.mp4 + video.jpg)
toolkitrs vidwrap video.mp4
```

Supported companion image formats (in priority order): `.png`, `.jpg`, `.jpeg`, `.bmp`, `.gif`, `.webp`

### Batch Arguments

All batch commands (`ts2mp4`, `mkv2mp3`, `mp32mp4`) share these options:

| Flag                 | Default | Description                                           |
|----------------------|---------|-------------------------------------------------------|
| `--output-dir <DIR>` | `./out` | Directory for converted files (created automatically) |
| `--force`            | `false` | Overwrite existing output files instead of skipping   |

## 🔧 Development

```bash
# Build debug binary
cargo build

# Run tests
cargo test

# Clippy linting
cargo clippy --all-targets -- -D warnings

# Format check
cargo fmt --check
```

# License

[MIT](./LICENSE)
