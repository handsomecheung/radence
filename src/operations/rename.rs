use crate::common::{self, sanitize_filename};
use crate::metadata::get_audio_objects;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

fn get_random_string(length: usize) -> String {
    let chars = b"abcdefghijklmnopqrstuvwxyz";
    (0..length)
        .map(|_| chars[fastrand::usize(..chars.len())] as char)
        .collect()
}

pub fn get_metadata_hash(info: &HashMap<String, String>, force: bool) -> String {
    let mut content = format!(
        "{}{}{}{}{}{}{}",
        info.get("album").map(|s| s.as_str()).unwrap_or(""),
        info.get("title").map(|s| s.as_str()).unwrap_or(""),
        info.get("artist").map(|s| s.as_str()).unwrap_or(""),
        info.get("album_artist").map(|s| s.as_str()).unwrap_or(""),
        info.get("genre").map(|s| s.as_str()).unwrap_or(""),
        info.get("tracknumber").map(|s| s.as_str()).unwrap_or(""),
        info.get("discnumber").map(|s| s.as_str()).unwrap_or(""),
    );
    if force {
        content.push_str(&get_random_string(8));
    }
    let md5 = common::get_md5(content.as_bytes());
    if md5.len() >= 6 {
        md5[..6].to_string()
    } else {
        md5
    }
}

pub fn rename_files_by_metadata(
    folder: &Path,
    use_artist: bool,
    use_composer: bool,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    let objs = get_audio_objects(folder)?;
    let mut infos = Vec::new();
    let mut discs = HashSet::new();

    for obj in &objs {
        let mut tags = obj.get_tags();
        tags.insert("name".to_string(), obj.filename.clone());
        tags.insert("path".to_string(), obj.path.to_string_lossy().to_string());
        if let Some(disc) = tags.get("discnumber") {
            discs.insert(disc.clone());
        }
        infos.push(tags);
    }

    let use_disc = discs.len() > 1;

    for info in infos {
        rename_file_by_metadata(folder, &info, use_artist, use_composer, use_disc, force)?;
    }

    Ok(())
}

fn rename_file_by_metadata(
    folder: &Path,
    info: &HashMap<String, String>,
    use_artist: bool,
    use_composer: bool,
    use_disc: bool,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    let name = info.get("name").cloned().unwrap_or_default();
    let old_path = PathBuf::from(info.get("path").cloned().unwrap_or_default());

    let title = info.get("title").cloned().unwrap_or_default();
    let artist = info.get("artist").cloned().unwrap_or_default();
    let composer = info.get("composer").cloned().unwrap_or_default();
    let tracknumber = info.get("tracknumber").cloned().unwrap_or_default();
    let disc = info.get("discnumber").cloned().unwrap_or_default();

    let extname = old_path
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();

    let mut new_name = title;
    if use_artist && !artist.is_empty() {
        new_name = format!("{artist} ---- {new_name}");
    }

    if use_composer && !composer.is_empty() {
        new_name = format!("{composer} {new_name}");
    }

    if !tracknumber.is_empty() {
        new_name = format!("{tracknumber}.{new_name}");
    } else {
        println!("{name} has no tracknumber. discard it ...");
        return Ok(());
    }

    if use_disc && !disc.is_empty() {
        new_name = format!("{disc}-{new_name}");
    }

    new_name = sanitize_filename(&new_name);

    let digest = get_metadata_hash(info, force);
    let version = chrono::Local::now().format("%Y%m%d").to_string();
    let final_filename = format!("{new_name}.{version}.{digest}{extname}");
    let new_path = folder.join(&final_filename);

    if !new_path.exists() {
        println!("rename `{}` -> `{}`", old_path.display(), new_path.display());
        std::fs::rename(&old_path, &new_path)?;
    }

    Ok(())
}
