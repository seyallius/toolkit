# 🧰 toolkit

> scripts that do things. they just work. mostly.

A growing collection of scripts I use to make computers do what I want. PowerShell, Bash, whatever gets the job done.

## 📦 What's in here

### 🎬 FFmpeg Scripts (`powershell/ffmpeg/`)

Convert media files without losing your mind.

| Script    | What it does                                              | Usage                                                                   |
|-----------|-----------------------------------------------------------|-------------------------------------------------------------------------|
| `mkv2mp3` | Extract audio from MKV files to MP3, preserving cover art | Drag `.mkv` files onto `mkv2mp3.cmd` or run in folder with `.mkv` files |
| `mp32mp4` | Convert MP3 to MP4 with cover art as video                | Drag `.mp3` files onto `mp32mp4.cmd` or run in folder with `.mp3` files |
| `ts2mp4`  | Remux `.ts` files to `.mp4` (no re-encode)                | `ts2mp4.cmd` in folder with `.ts` files                                 |

**Common flags for mkv2mp3 and mp32mp4:**

- `-OutputDir <path>` — where to put converted files (default: `./out`)
- `-Force` — overwrite existing output files
- `-ShowVerbose` — see what ffmpeg is actually doing

### 🤖 Just Commands (`justfile`)

Development shortcuts for my projects. Run `just` to see all available commands.

| Group        | What's in there                                            |
|--------------|------------------------------------------------------------|
| Development  | `build`, `dev`, `check`, `treeclip`, dependency management |
| Docker       | `db-up`, `db-down`                                         |
| Code Quality | `lint`, `audit`                                            |
| Dependency   | `vendor`, `vendor-clean`                                   |
| Git          | `amend`, `rebase`, `diff-cp`, `today`                      |

## 🚀 Quick Start

```bash
# Clone it wherever you keep your tools
git clone https://github.com/your-username/toolkit.git

# For PowerShell scripts: just double-click the .cmd file
# or run from terminal:
.\powershell\ffmpeg\mkv2mp3.cmd my-video.mkv

# For just commands: install just, then:
just build
just dev
```

## 📝 Adding New Scripts

When adding a new script, update the table above:

```markdown
| `script-name` | One-line description of what it does | How to use it |
```

## ⚠️ Disclaimer

Some of these were written at 3 AM. They work on my machine. Your mileage may vary.

## 📜 License

Do whatever you want. If it breaks, you get to keep both pieces.
