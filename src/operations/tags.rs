use crate::common::{name_unify, pad_string, INVALID_KEYWORDS};
use crate::metadata::{get_audio_objects, AudioTrack};
use crate::operations::rename::rename_files_by_metadata;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn show_tags_dir(folder: &Path) -> Result<(), Box<dyn Error>> {
    let objs = get_audio_objects(folder)?;
    for obj in &objs {
        let tags = obj.get_tags();
        println!("{}: ", obj.filename);
        let mut keys: Vec<&String> = tags.keys().collect();
        keys.sort();
        for key in keys {
            let val = &tags[key];
            println!("{key}: {val}");
        }
        println!();
    }
    Ok(())
}

pub fn show_tag_dir(folder: &Path, tag: &str) -> Result<(), Box<dyn Error>> {
    let objs = get_audio_objects(folder)?;
    for obj in &objs {
        let val = obj.get_tag(tag);
        println!("{}: {val}", pad_string(&obj.filename, 40));
    }
    Ok(())
}

pub fn check_tags(folder: &Path) -> Result<(), Box<dyn Error>> {
    let objs = get_audio_objects(folder)?;
    let mut all_tags = HashMap::new();
    for obj in &objs {
        all_tags.insert(obj.filename.clone(), obj.get_tags());
    }

    // 1. Check watermark keywords
    let mut invalid_infos = HashMap::new();
    for obj in &objs {
        let title = obj.get_tag("title");
        for keyword in INVALID_KEYWORDS.iter() {
            if title.contains(keyword) {
                invalid_infos.insert(obj.filename.clone(), *keyword);
                break;
            }
        }
    }
    let name_status = invalid_infos.is_empty();

    // 2. Check completeness
    let required_tags = [
        "title",
        "artist",
        "album",
        "album_artist",
        "genre",
        "tracknumber",
        "discnumber",
    ];
    let mut all_missing_tags = HashMap::new();
    for (name, tags) in &all_tags {
        let mut missing = Vec::new();
        for req in required_tags {
            if tags.get(req).map(|s| s.as_str()).unwrap_or("") == "" {
                missing.push(req);
            }
        }
        if !missing.is_empty() {
            all_missing_tags.insert(name.clone(), missing);
        }
    }
    let completeness_status = all_missing_tags.is_empty();

    // 3. Check consistency
    let ctags = ["album", "album_artist"];
    let mut all_consistency_tags: HashMap<&str, HashMap<String, Vec<String>>> = HashMap::new();
    for (name, tags) in &all_tags {
        for &ctag in &ctags {
            let val = tags.get(ctag).cloned().unwrap_or_default();
            all_consistency_tags
                .entry(ctag)
                .or_default()
                .entry(val)
                .or_default()
                .push(name.clone());
        }
    }

    let mut consistency_status = true;
    for info in all_consistency_tags.values() {
        if info.len() > 1 {
            consistency_status = false;
        }
    }

    if name_status && completeness_status && consistency_status {
        println!("status: ok");
        return Ok(());
    }

    if !name_status {
        for (name, keyword) in &invalid_infos {
            println!("\"{name}\" contains invalid keywords: {keyword}");
        }
        println!();
    }

    if !completeness_status {
        for (name, missing) in &all_missing_tags {
            println!("\"{name}\" missing tags: {missing:?}");
            for tag in missing {
                println!("{tag} : radence add-field --file '{name}' {tag} ''");
            }
            println!();
        }
    }

    if !consistency_status {
        for (&ctag, info) in &all_consistency_tags {
            if info.len() > 1 {
                let mut max_tag = String::new();
                let mut max_count = 0;
                for (tag, names) in info {
                    if names.len() > max_count {
                        max_tag = tag.clone();
                        max_count = names.len();
                    }
                }
                for (tag, names) in info {
                    if tag != &max_tag {
                        for name in names {
                            println!("name {name} tag {ctag} not consistent. max_tag: {max_tag}, tag: {tag}");
                        }
                    }
                }
                println!();
            }
        }
    }

    Err("Tag verification failed".into())
}

pub fn tag_interactive(folder: &Path) -> Result<(), Box<dyn Error>> {
    let mut objs = get_audio_objects(folder)?;
    let fields = [
        "album",
        "album_artist",
        "composer",
        "conductor",
        "genre",
        "date",
    ];

    let mut album_common: HashMap<&str, HashSet<String>> = HashMap::new();
    for obj in &objs {
        let tags = obj.get_tags();
        for &field in &fields {
            if let Some(val) = tags.get(field) {
                if !val.is_empty() {
                    album_common
                        .entry(field)
                        .or_default()
                        .insert(val.clone());
                }
            }
        }
    }

    for &field in &fields {
        if let Some(values) = album_common.get(field) {
            let list: Vec<String> = values.iter().cloned().collect();
            println!("{}: {}", field, list.join(" || "));
        }
    }
    println!("\n");

    let keys = [
        "album",
        "artist",
        "album_artist",
        "conductor",
        "composer",
        "genre",
        "date",
        "discnumber",
        "comment",
    ];

    let mut items = HashMap::new();
    let stdin = io::stdin();
    for &key in &keys {
        print!("{key}: ");
        io::stdout().flush()?;
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            items.insert(key, trimmed.to_string());
        }
    }

    for obj in &mut objs {
        for (&key, val) in &items {
            obj.set_tag(key, val)?;
        }
    }

    println!("Updated tags for {} tracks", objs.len());
    Ok(())
}

pub fn add_field(
    folder: &Path,
    key: &str,
    value: &str,
    file: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    if let Some(filepath) = file {
        let mut track = AudioTrack::new(filepath.to_path_buf());
        track.set_tag(key, value)?;
        println!("Set `{key}` to `{value}` on {}", filepath.display());
    } else {
        let mut objs = get_audio_objects(folder)?;
        for obj in &mut objs {
            obj.set_tag(key, value)?;
        }
        println!("Set `{key}` to `{value}` on {} tracks", objs.len());
    }
    Ok(())
}

pub fn replace_track(folder: &Path, target: &str, source: &str) -> Result<(), Box<dyn Error>> {
    let mut objs = get_audio_objects(folder)?;
    for obj in &mut objs {
        obj.replace_tag("title", target, source)?;
    }
    println!("Replaced \"{target}\" with \"{source}\" in title for {} files", objs.len());
    Ok(())
}

pub fn remove_invalid_keywords(
    folder: &Path,
    force: bool,
    use_artist: bool,
    use_composer: bool,
) -> Result<(), Box<dyn Error>> {
    let mut objs = get_audio_objects(folder)?;
    let mut modified_count = 0;

    for obj in &mut objs {
        let title = obj.get_tag("title");
        let mut new_title = title.clone();
        for keyword in INVALID_KEYWORDS.iter() {
            if new_title.contains(keyword) {
                new_title = new_title.replace(keyword, "");
            }
        }
        if new_title != title {
            obj.set_tag("title", &new_title)?;
            modified_count += 1;
            println!("Cleaned keywords from: {}", obj.filename);
        }
    }

    if modified_count > 0 {
        println!("Cleaned keywords from {modified_count} files, renaming...");
        rename_files_by_metadata(folder, use_artist, use_composer, force)?;
    } else {
        println!("No invalid keywords found.");
    }

    Ok(())
}

fn tracknumber_alignment(value: &str, total_count: usize) -> (bool, String) {
    let ltotal = total_count.to_string().len();
    let lcurrent = value.len();
    if !value.is_empty() && lcurrent < ltotal {
        let padded = format!("{:0>width$}", value, width = ltotal);
        (true, padded)
    } else {
        (false, value.to_string())
    }
}

pub fn set_tracknumber(folder: &Path, tracklist_file: &Path) -> Result<(), Box<dyn Error>> {
    let content = std::fs::read_to_string(tracklist_file)?;
    let track_regex = Regex::new(r#"^(.+)\.\s+(.+)$"#)?;

    let mut raw_tracks = Vec::new();
    for raw_line in content.lines() {
        let line = name_unify(raw_line);
        if !line.contains(". ") {
            continue;
        }
        if let Some(caps) = track_regex.captures(&line) {
            let num = caps[1].trim().to_string();
            let title = caps[2].trim().to_string();
            raw_tracks.push((num, title));
        }
    }

    let total = raw_tracks.len();
    let mut number_info: HashMap<String, String> = HashMap::new();
    for (num, title) in raw_tracks {
        let (_, padded_num) = tracknumber_alignment(&num, total);
        number_info.insert(padded_num, title);
    }

    let objs = get_audio_objects(folder)?;
    let mut sorted_objs = objs;
    sorted_objs.sort_by(|a, b| {
        let stem_a = a.path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
        let stem_b = b.path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
        name_unify(&stem_b).len().cmp(&name_unify(&stem_a).len())
    });

    let mut rename_plan: Vec<(PathBuf, PathBuf)> = Vec::new();
    for obj in sorted_objs {
        let stem = obj.path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
        let name = name_unify(&stem);

        let mut matched_num = None;
        let mut sorted_items: Vec<(String, String)> = number_info.clone().into_iter().collect();
        sorted_items.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (num, title) in &sorted_items {
            if name.contains(title) {
                matched_num = Some(num.clone());
                break;
            }
        }

        if let Some(num) = matched_num {
            let new_filename = format!("{num}.{}", obj.filename);
            let new_path = folder.join(&new_filename);
            rename_plan.push((obj.path.clone(), new_path));
            number_info.remove(&num);
        } else {
            return Err(format!("{} not matched in tracklist", obj.filename).into());
        }
    }

    let mut new_paths_set = HashSet::new();
    for (old_p, new_p) in &rename_plan {
        if new_p.exists() && new_p != old_p {
            return Err(format!("{}: new name {} already exists", old_p.display(), new_p.display()).into());
        }
        if !new_paths_set.insert(new_p.clone()) {
            return Err(format!("Duplicate target name: {}", new_p.display()).into());
        }
    }

    for (old_p, new_p) in rename_plan {
        println!("set {} to {}", old_p.display(), new_p.display());
        std::fs::rename(old_p, new_p)?;
    }

    Ok(())
}

pub fn autofix(
    folder: &Path,
    force: bool,
    use_artist: bool,
    use_composer: bool,
) -> Result<(), Box<dyn Error>> {
    let _ = remove_invalid_keywords(folder, force, use_artist, use_composer);

    let mut objs = get_audio_objects(folder)?;
    let total_tracks = objs.len();
    let mut modified = false;

    let filename_regex = Regex::new(r#"^([^\.]+)\.(.+)\.([a-zA-Z0-9]+)$"#)?;
    let dir_regex = Regex::new(r#"^(.+)\s+-\s+(.+)$"#)?;

    for obj in &mut objs {
        let tags = obj.get_tags();
        let mut changes: Vec<(&str, String)> = Vec::new();

        // 1. Slash in discnumber / tracknumber
        for &k in &["discnumber", "tracknumber"] {
            if let Some(val) = tags.get(k) {
                if val.contains('/') {
                    let cleaned = val.split('/').next().unwrap_or("").trim().to_string();
                    println!("fix {}, slash: {} -> {}", obj.filename, val, cleaned);
                    changes.push((k, cleaned));
                }
            }
        }

        // 2. Whitespace trimming
        for (&ref k, ref val) in &tags {
            if val.starts_with(' ') || val.ends_with(' ') {
                let trimmed = val.trim().to_string();
                println!("fix {}, space: `{}` -> `{}`", obj.filename, val, trimmed);
                changes.push((k.as_str(), trimmed));
            }
        }

        // 3. Tracknumber alignment
        if let Some(val) = tags.get("tracknumber") {
            let (changed, aligned) = tracknumber_alignment(val, total_tracks);
            if changed {
                println!("fix {}, alignment: {} -> {}", obj.filename, val, aligned);
                changes.push(("tracknumber", aligned));
            }
        }

        // 4. Discnumber default
        if let Some(val) = tags.get("discnumber") {
            if val == "0" || val.is_empty() {
                println!("fix {}, discnumber: {} -> 1", obj.filename, val);
                changes.push(("discnumber", "1".to_string()));
            }
        }

        // 5. Autotag from filename and directory
        let file_stem = obj.path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
        let ext = obj.get_extname();
        let full_name = format!("{file_stem}.{ext}");

        let mut auto_title = String::new();
        let mut auto_disc = String::new();
        let mut auto_track = String::new();

        if let Some(caps) = filename_regex.captures(&full_name) {
            let mut trk = caps[1].to_string();
            auto_title = caps[2].to_string();
            if trk.contains('-') {
                let parts: Vec<&str> = trk.split('-').collect();
                if parts.len() == 2 {
                    auto_disc = parts[0].to_string();
                    trk = parts[1].to_string();
                }
            }
            auto_track = trk;
        }

        let dir_name = obj.path.parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut auto_artist = String::new();
        let mut auto_album = String::new();
        if let Some(caps) = dir_regex.captures(&dir_name) {
            auto_artist = caps[1].trim().to_string();
            auto_album = caps[2].trim().to_string();
        }

        let auto_candidates = [
            ("title", auto_title),
            ("discnumber", auto_disc),
            ("tracknumber", auto_track),
            ("album", auto_album),
            ("album_artist", auto_artist),
        ];

        for (k, v) in auto_candidates {
            if tags.get(k).map(|s| s.as_str()).unwrap_or("").is_empty() && !v.is_empty() {
                println!("fix {}, autotag: `{k}` = `{v}`", obj.filename);
                changes.push((k, v));
            }
        }

        if !changes.is_empty() {
            modified = true;
            for (k, v) in changes {
                obj.set_tag(k, &v)?;
            }
        }
    }

    if modified {
        println!("Autofix applied to files, renaming files by metadata...");
        rename_files_by_metadata(folder, use_artist, use_composer, force)?;
    } else {
        println!("No autofix needed.");
    }

    Ok(())
}
