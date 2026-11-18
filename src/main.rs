mod cli;
mod common;
mod metadata;
mod operations;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Rename {
            artist,
            composer,
            force,
            folder,
        } => operations::rename::rename_files_by_metadata(&folder, artist, composer, force),

        Commands::Tag { folder } => operations::tags::tag_interactive(&folder),

        Commands::ShowTags { folder } => operations::tags::show_tags_dir(&folder),

        Commands::ShowTag { tag, folder } => operations::tags::show_tag_dir(&folder, &tag),

        Commands::CheckTags { folder } => operations::tags::check_tags(&folder),

        Commands::Autofix {
            artist,
            composer,
            force,
            folder,
        } => operations::tags::autofix(&folder, force, artist, composer),

        Commands::RemoveInvalidKeywords {
            artist,
            composer,
            force,
            folder,
        } => operations::tags::remove_invalid_keywords(&folder, force, artist, composer),

        Commands::AddField {
            key,
            value,
            file,
            folder,
        } => operations::tags::add_field(&folder, &key, &value, file.as_deref()),

        Commands::ReplaceTrack {
            target,
            source,
            folder,
        } => operations::tags::replace_track(&folder, &target, &source),

        Commands::SetTracknumber { file, folder } => {
            operations::tags::set_tracknumber(&folder, &file)
        }

        Commands::ShowArtworks { folder } => operations::artwork::show_artworks(&folder),

        Commands::RemoveArtworks { folder } => operations::artwork::remove_artworks(&folder),

        Commands::ResetArtworks { file, folder } => {
            operations::artwork::reset_artworks(&folder, &file)
        }

        Commands::ShowQuality { detail, folders } => {
            operations::quality::show_quality(&folders, detail)
        }

        Commands::RenameByQuality { folders } => {
            operations::quality::rename_by_quality(&folders)
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
