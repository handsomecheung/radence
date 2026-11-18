# Radence

Radence is a fast audio metadata and library management CLI tool implemented in Rust. It provides batch tag inspection, cleaning, autofixing, renaming, and quality probing for audio libraries.

## Features

- **Tag Inspection & Editing**: View tags (`show-tags`, `show-tag`), interactive tagging (`tag`), add or replace fields (`add-field`, `replace-track`).
- **Tag Validation & Autofix**: Detect watermark/promo keywords (`check-tags`, `remove-invalid-keywords`), normalize track/disc numbering slashes, trim whitespace, and infer metadata from filenames or folder structures (`autofix`).
- **Batch Renaming**: Format filenames based on metadata with MD5 content hashing and character sanitization (`rename`). Prefix tracks matching an external tracklist file (`set-tracknumber`).
- **Cover Artwork**: View embedded cover art via `imgcat` (`show-artworks`), strip artworks (`remove-artworks`), or embed new covers (`reset-artworks`).
- **Audio Quality**: Probe audio stream properties via `ffprobe` (`show-quality`) and append format tags (e.g. `[FLAC(96.0-24)]`, `[DSD64]`) to folder names (`rename-by-quality`).

## Supported Formats

Supports common audio formats via [lofty](https://crates.io/crates/lofty), including:
- FLAC (`.flac`)
- MP3 (`.mp3`)
- DSF (`.dsf`)
- MP4 / AAC (`.m4a`)
- WAV (`.wav`)
- APE (`.ape`)

## Requirements

- Rust 1.85+
- (Optional) `ffprobe` for `show-quality` and `rename-by-quality`
- (Optional) `imgcat` for `show-artworks` terminal preview

## Installation

```bash
cargo build --release
```

The compiled binary will be located at `target/release/Radence`.

## Usage

```text
Radence <COMMAND> [OPTIONS]
```

### Commands

| Command | Description |
| :--- | :--- |
| `rename` | Rename audio files based on metadata (`[disc-][track].[composer ][artist ---- ]title.date.hash.ext`) |
| `show-tags` | Print common tags for all audio files in directory |
| `show-tag <TAG>` | Display a specific tag across all files |
| `check-tags` | Validate required tags, consistency across album, and watermark keywords |
| `tag` | Interactive terminal prompt to set album-wide metadata |
| `add-field <KEY> [VALUE]` | Add or update a metadata tag field |
| `replace-track <TARGET> [SOURCE]` | Find and replace substring in track titles |
| `remove-invalid-keywords` | Strip promotional watermark strings from titles and rename files |
| `set-tracknumber --file <FILE>` | Parse a tracklist text file and prefix tracks with numbers |
| `autofix` | Automatically fix common tag issues and rename files |
| `show-artworks` | Extract and view embedded cover art |
| `remove-artworks` | Remove embedded cover art |
| `reset-artworks --file <FILE>` | Embed specified cover image into all audio files |
| `show-quality` | Probe and display audio stream quality |
| `rename-by-quality` | Append audio quality tag to directory name |
