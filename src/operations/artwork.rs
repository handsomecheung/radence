use crate::metadata::get_audio_objects;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::process::Command;

pub fn show_artworks(folder: &Path) -> Result<(), Box<dyn Error>> {
    let objs = get_audio_objects(folder)?;
    let tmp_dir = tempfile::Builder::new()
        .prefix("radence-")
        .tempdir()?;
    let tmp_path = tmp_dir.path();

    let mut artworks = HashSet::new();
    for obj in &objs {
        let files = obj.write_artwork_files(tmp_path)?;
        for file in files {
            artworks.insert(file);
        }
    }

    if !artworks.is_empty() {
        let mut cmd = Command::new("imgcat");
        for file in &artworks {
            cmd.arg(file);
        }

        match cmd.status() {
            Ok(status) => {
                if !status.success() {
                    eprintln!("imgcat exited with status: {status}");
                }
            }
            Err(_) => {
                println!("imgcat is not available. Artworks saved in temporary directory:");
                for file in &artworks {
                    println!("  {}", file.display());
                }
            }
        }
    }

    println!("total: {} artworks", artworks.len());
    Ok(())
}

pub fn remove_artworks(folder: &Path) -> Result<(), Box<dyn Error>> {
    let mut objs = get_audio_objects(folder)?;
    for obj in &mut objs {
        obj.remove_artworks()?;
    }
    println!("Removed artworks from {} files", objs.len());
    Ok(())
}

pub fn reset_artworks(folder: &Path, cover_path: &Path) -> Result<(), Box<dyn Error>> {
    if !cover_path.is_file() {
        return Err(format!("Cover file `{}` does not exist", cover_path.display()).into());
    }

    let mut objs = get_audio_objects(folder)?;
    for obj in &mut objs {
        obj.remove_artworks()?;
        obj.set_artworks(cover_path)?;
    }
    println!("Reset artworks for {} files using `{}`", objs.len(), cover_path.display());
    Ok(())
}
