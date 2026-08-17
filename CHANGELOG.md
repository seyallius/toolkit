# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/seyallius/toolkit/compare/v0.1.2...v0.1.3) - 2026-08-17

### Other

- adopt standard Rust target triple naming for cargo-binstall

## [0.1.2](https://github.com/seyallius/toolkit/compare/v0.1.1...v0.1.2) - 2026-08-17

### Other

- *(license)* update README to reflect dual MIT/Apache-2.0 license

## [0.1.1](https://github.com/seyallius/toolkit/compare/v0.1.0...v0.1.1) - 2026-08-17

### Added

- *(ui)* add extensive spinner styles and tune animation speed
- *(toolkit)* add Rust FFmpeg CLI toolkit for media workflows
- *(vidwrap)* add tool for embedding images as video thumbnails
- *(scripts)* add bash train monitor and test notification scripts
- *(ffmpeg)* add MP3-to-MP4 converter with cover art extraction
- *(mkv2mp3)* add PowerShell script and batch wrapper for MKV to MP3 conversion
- *(conv)* add TS to MP4 batch converter script

### Fixed

- *(ffmpeg)* encode_loop argument order for correct image looping (wip)
- *(justfile)* include untracked files in diff-cp recipes
- *(justfile)* include staged changes in diff-cp commands

### Other

- rename package to toolkitrs and prepare for crates.io publication
- *(readme)* rewrite README to document unified Rust FFmpeg CLI
- add agent contribution guide
- enhance doc comments and rename spinner stop field for clarity
- *(backup)* archive legacy scripts and Go vidwrap implementation
- *(commands)* extract generic batch processor to eliminate DRY
- *(toolkit)* add module-level documentation and refactor constants
- *(vidwrap)* add comprehensive table-driven tests for all packages
- *(idea)* add JetBrains IDE workspace configuration and format README table
- add README with project overview and usage guide
- *(git)* add .gitignore and extend .treeclipignore
- add .treeclipignore with default ignore patterns
- *(ffmpeg)* reorganize conversion scripts into powershell/ffmpeg directory
- Add basic justfile
