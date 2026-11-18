use crate::metadata::get_audio_objects;
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

fn parse_ffprobe_output(
    output: &str,
    start: &str,
    end: &str,
) -> Vec<HashMap<String, String>> {
    let mut sections = Vec::new();
    let mut section_current = HashMap::new();
    let mut section_enable = false;

    for raw_line in output.lines() {
        let line = raw_line.trim();

        if line == end {
            if section_enable {
                section_enable = false;
                sections.push(section_current);
                section_current = HashMap::new();
            }
        }

        if section_enable {
            if let Some((k, v)) = line.split_once('=') {
                section_current.insert(k.to_string(), v.to_string());
            }
        }

        if line == start {
            section_enable = true;
        }
    }

    sections
}

pub fn get_audio_quality(filename: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-show_format", "-show_streams"])
        .arg(filename)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed on {}: {}",
            filename.display(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    let streams = parse_ffprobe_output(&out_str, "[STREAM]", "[/STREAM]");
    let formats = parse_ffprobe_output(&out_str, "[FORMAT]", "[/FORMAT]");

    let stream = streams
        .iter()
        .find(|s| s.get("codec_name").map(|c| c.as_str()) != Some("mjpeg"))
        .ok_or_else(|| format!("No non-mjpeg stream found in {}", filename.display()))?;

    let format = formats
        .first()
        .ok_or_else(|| format!("No format info found in {}", filename.display()))?;

    let format_name = format.get("format_name").map(|s| s.as_str()).unwrap_or("");
    let mut quality = if format_name == "dsf" {
        let sample_rate: i64 = stream
            .get("sample_rate")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let bits_per_sample: i64 = stream
            .get("bits_per_sample")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let total_rate = sample_rate * bits_per_sample;
        let ratio = total_rate / 44100;
        if ![64, 128, 256, 512, 1024].contains(&ratio) {
            eprintln!("Warning: unusual DSD ratio {ratio} for {}", filename.display());
        }
        format!("DSD{ratio}")
    } else if format_name == "flac" {
        let sample_rate: f64 = stream
            .get("sample_rate")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let bits = stream
            .get("bits_per_raw_sample")
            .cloned()
            .unwrap_or_else(|| "16".to_string());
        let rate = sample_rate / 1000.0;
        if rate <= 192.0 {
            format!("FLAC({rate:.1}-{bits})")
        } else {
            "DXD".to_string()
        }
    } else if format_name == "mp3" {
        let bit_rate: i64 = stream
            .get("bit_rate")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let rate = bit_rate / 1000;
        format!("MP3({rate})")
    } else if format_name.contains("mp4")
        || format_name.contains("m4a")
        || format_name.contains("mov")
    {
        let bit_rate: i64 = stream
            .get("bit_rate")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let rate = bit_rate / 1000;
        format!("AAC({rate})")
    } else {
        return Err(format!("Unsupported format_name `{format_name}` in {}", filename.display()).into());
    };

    if let Some(channel_layout) = stream.get("channel_layout") {
        if channel_layout != "stereo" {
            if channel_layout.contains("5.1") {
                quality.push_str(" 5.1CH");
            } else {
                eprintln!("Warning: unusual channel_layout `{channel_layout}`");
            }
        }
    }

    Ok(quality)
}

pub fn get_tracks_qualities(folder: &Path) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let objs = get_audio_objects(folder)?;
    let mut map = HashMap::new();
    for obj in objs {
        match get_audio_quality(&obj.path) {
            Ok(q) => {
                map.insert(obj.filename, q);
            }
            Err(e) => {
                eprintln!("Error getting quality for {}: {e}", obj.filename);
            }
        }
    }
    Ok(map)
}

pub fn get_folder_quality(folder: &Path, show_all: bool) -> Result<String, Box<dyn Error>> {
    let qualities = get_tracks_qualities(folder)?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for q in qualities.values() {
        *counts.entry(q.clone()).or_insert(0) += 1;
    }

    let mut sorted_qualities: Vec<(String, usize)> = counts.into_iter().collect();
    sorted_qualities.sort_by(|a, b| b.1.cmp(&a.1));

    let list: Vec<String> = if !show_all && !sorted_qualities.is_empty() {
        vec![sorted_qualities[0].0.clone()]
    } else {
        sorted_qualities.into_iter().map(|(q, _)| q).collect()
    };

    Ok(format!("[{}]", list.join(" ")))
}

pub fn show_quality(folders: &[PathBuf], detail: bool) -> Result<(), Box<dyn Error>> {
    let target_folders = if folders.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        folders.to_vec()
    };

    for folder in target_folders {
        if !folder.is_dir() {
            continue;
        }

        if detail {
            println!("{}: \n", folder.display());
            let qualities = get_tracks_qualities(&folder)?;
            for (name, quality) in qualities {
                println!("{quality} : {name}");
            }
        } else {
            let q = get_folder_quality(&folder, true)?;
            println!("{}: {q}", folder.display());
        }
    }

    Ok(())
}

pub fn rename_by_quality(folders: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let target_folders = if folders.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        folders.to_vec()
    };

    for folder in target_folders {
        if !folder.is_dir() {
            continue;
        }

        let quality = get_folder_quality(&folder, false)?;
        let folder_name = folder
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let new_name = format!("{folder_name} {quality}");
        let parent = folder.parent().unwrap_or_else(|| Path::new("."));
        let new_path = parent.join(&new_name);

        println!("rename folder `{}` to `{}`", folder.display(), new_path.display());
        std::fs::rename(&folder, &new_path)?;
    }

    Ok(())
}
