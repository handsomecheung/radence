# Project Context

Before starting work, ensure you have read the `README.md` in the root directory to understand the project's background, objectives, and overall architecture.

## Radence Development Rules

Radence is a high-performance audio metadata inspection, tagging, and library organization CLI tool implemented in Rust, migrated from the Python tool `mtool`.

### 1. Core Instructions & Policies

*   **Modification Policy:** Do not modify the code unless the user explicitly asks you to. If you believe a modification is necessary, explain the reason and obtain consent first.
*   **Git Policy:** Unless explicitly instructed otherwise, do not change the Git state (do not stage changes, create commits, or push). When committing, never add `Co-Authored-By`, `Claude-Session`, or any other Claude/Anthropic metadata trailers.
*   **Language & Comments:** When comments are needed in code or config files, write them in English.
*   **Bash Shebang:** When creating Bash scripts, use `#!/usr/bin/env bash` as the shebang.
*   **File Links:** When referencing files, create clickable markdown links using the `file://` scheme.

### 2. Architecture & Directory Structure

```
radence/
├── Cargo.toml            # Dependencies: clap, lofty, zhconv, deunicode, md-5, etc.
├── README.md             # Project documentation and CLI usage
├── AGENTS-CONTEXT.md     # Shared guidelines and context for AI agents
├── CLAUDE.md             # Claude Code entry point
├── GEMINI.md             # Antigravity CLI / Gemini entry point
├── AGENTS.md             # Codex entry point
└── src/
    ├── main.rs           # CLI entry point, exit code handling, and dispatch
    ├── cli.rs            # Clap subcommand and flag definitions
    ├── common.rs         # Text unification (zhconv), sanitization (deunicode), MD5, keywords
    ├── metadata.rs       # Unified Lofty abstraction for FLAC, MP3, DSF, M4A, etc.
    └── operations/
        ├── mod.rs        # Module exports
        ├── tags.rs       # Tag inspection, editing, check-tags, autofix, set-tracknumber
        ├── rename.rs     # Metadata-based file renaming with MD5 hash
        ├── artwork.rs    # Embedded cover artwork display, remove, and reset
        └── quality.rs    # Audio format/stream property probing via ffprobe
```

### 3. Environment & Tooling

*   **Rust Edition:** Rust 2024 edition (Rust 1.85+ / 1.91+).
*   **Package Manager:** This environment uses the **Nix package manager** for system packages.
*   **External Command Dependencies:**
    - `ffprobe`: Used for `show-quality` and `rename-by-quality` audio stream inspection.
    - `imgcat`: Used for `show-artworks` terminal preview (falls back gracefully if missing).

### 4. Build & Test Commands

*   Check: `cargo check`
*   Build: `cargo build`
*   Test: `cargo test`
*   Release build: `cargo build --release`
