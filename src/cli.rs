use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "radence",
    about = "Radence - High-Performance Audio Metadata & Library Organizer",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "kebab-case")]
pub enum Commands {
    /// Rename audio files based on metadata
    Rename {
        #[arg(long, help = "Include artist name in filename")]
        artist: bool,

        #[arg(long, help = "Include composer name in filename")]
        composer: bool,

        #[arg(long, help = "Force rename with random suffix")]
        force: bool,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Interactive tagging prompt for album metadata
    Tag {
        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Show common tags for all audio files in directory
    ShowTags {
        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Show specific tag value across all audio files in directory
    ShowTag {
        #[arg(help = "Tag key (e.g. artist, title, album)")]
        tag: String,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Validate tags for completeness, consistency, and watermark keywords
    CheckTags {
        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Automatically fix common tag issues and rename files
    Autofix {
        #[arg(long, help = "Include artist name during rename")]
        artist: bool,

        #[arg(long, help = "Include composer name during rename")]
        composer: bool,

        #[arg(long, help = "Force rename with random suffix")]
        force: bool,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Remove watermark promo keywords from track titles and rename
    RemoveInvalidKeywords {
        #[arg(long, help = "Include artist name during rename")]
        artist: bool,

        #[arg(long, help = "Include composer name during rename")]
        composer: bool,

        #[arg(long, help = "Force rename with random suffix")]
        force: bool,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Add or update a metadata tag field
    AddField {
        #[arg(help = "Field key (e.g. composer, album, genre)")]
        key: String,

        #[arg(default_value = "placeholder", help = "Field value")]
        value: String,

        #[arg(long, help = "Specific file to modify")]
        file: Option<PathBuf>,

        #[arg(default_value = ".", help = "Target directory if --file is omitted")]
        folder: PathBuf,
    },

    /// Replace substring in track titles
    ReplaceTrack {
        #[arg(help = "Substring to replace")]
        target: String,

        #[arg(default_value = "", help = "Replacement string")]
        source: String,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Parse tracklist file and prefix tracks with numbers
    SetTracknumber {
        #[arg(long, required = true, help = "Tracklist text file")]
        file: PathBuf,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Show embedded cover artworks using imgcat
    ShowArtworks {
        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Remove embedded artworks from audio files
    RemoveArtworks {
        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Remove existing artwork and embed new cover image
    ResetArtworks {
        #[arg(long, required = true, help = "Cover image file path")]
        file: PathBuf,

        #[arg(default_value = ".", help = "Target directory")]
        folder: PathBuf,
    },

    /// Show audio quality via ffprobe
    ShowQuality {
        #[arg(long, help = "Show detailed per-track information")]
        detail: bool,

        #[arg(help = "Target directories (defaults to current directory)")]
        folders: Vec<PathBuf>,
    },

    /// Rename directory with quality tag suffix (e.g. [FLAC(96.0-24)])
    RenameByQuality {
        #[arg(help = "Target directories (defaults to current directory)")]
        folders: Vec<PathBuf>,
    },
}
