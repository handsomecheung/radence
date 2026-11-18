use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::picture::{Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, ItemValue, Tag, TagExt, TagItem};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

pub const AUDIO_EXTENSIONS: &[&str] = &["flac", "mp3", "dsf", "m4a", "wav", "ape", "dff"];

pub fn is_audio_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

pub fn map_key_to_item_key(key: &str) -> Option<ItemKey> {
    match key.to_lowercase().as_str() {
        "title" => Some(ItemKey::TrackTitle),
        "artist" => Some(ItemKey::TrackArtist),
        "album" => Some(ItemKey::AlbumTitle),
        "album_artist" | "albumartist" => Some(ItemKey::AlbumArtist),
        "genre" => Some(ItemKey::Genre),
        "composer" => Some(ItemKey::Composer),
        "conductor" => Some(ItemKey::Conductor),
        "tracknumber" | "track" => Some(ItemKey::TrackNumber),
        "discnumber" | "disc" => Some(ItemKey::DiscNumber),
        "date" | "year" => Some(ItemKey::RecordingDate),
        "comment" => Some(ItemKey::Comment),
        _ => {
            for variant in ItemKey::VARIANTS {
                if format!("{variant:?}").eq_ignore_ascii_case(key) {
                    return Some(*variant);
                }
            }
            None
        }
    }
}

pub struct AudioTrack {
    pub path: PathBuf,
    pub filename: String,
}

impl AudioTrack {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        Self { path, filename }
    }

    pub fn get_extname(&self) -> String {
        self.path
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    }

    fn read_tag(&self) -> Result<Option<Tag>, Box<dyn Error>> {
        let tagged_file = Probe::open(&self.path)?.guess_file_type()?.read()?;
        if let Some(tag) = tagged_file.primary_tag() {
            Ok(Some(tag.clone()))
        } else if let Some(first) = tagged_file.first_tag() {
            Ok(Some(first.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn get_tags(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let fields = [
            "title",
            "artist",
            "album",
            "album_artist",
            "genre",
            "composer",
            "conductor",
            "tracknumber",
            "discnumber",
            "date",
            "comment",
        ];

        let tag = self.read_tag().ok().flatten();

        for &field in &fields {
            let val = self.extract_field_value(tag.as_ref(), field);
            map.insert(field.to_string(), val);
        }

        if map.get("discnumber").map(|s| s.as_str()).unwrap_or("") == "" {
            map.insert("discnumber".to_string(), "1".to_string());
        }

        map
    }

    fn extract_field_value(&self, tag: Option<&Tag>, field: &str) -> String {
        let Some(tag) = tag else {
            return String::new();
        };

        match field {
            "title" => tag
                .title()
                .map(|s| s.trim().to_string())
                .or_else(|| tag.get_string(ItemKey::TrackTitle).map(|s| s.trim().to_string()))
                .unwrap_or_default(),
            "artist" => tag
                .artist()
                .map(|s| s.trim().to_string())
                .or_else(|| tag.get_string(ItemKey::TrackArtist).map(|s| s.trim().to_string()))
                .unwrap_or_default(),
            "album" => tag
                .album()
                .map(|s| s.trim().to_string())
                .or_else(|| tag.get_string(ItemKey::AlbumTitle).map(|s| s.trim().to_string()))
                .unwrap_or_default(),
            "album_artist" => tag
                .get_string(ItemKey::AlbumArtist)
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            "genre" => tag
                .genre()
                .map(|s| s.trim().to_string())
                .or_else(|| tag.get_string(ItemKey::Genre).map(|s| s.trim().to_string()))
                .unwrap_or_default(),
            "composer" => tag
                .get_string(ItemKey::Composer)
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            "conductor" => tag
                .get_string(ItemKey::Conductor)
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            "tracknumber" => {
                if let Some(s) = tag.get_string(ItemKey::TrackNumber) {
                    s.trim().to_string()
                } else if let Some(n) = tag.track() {
                    n.to_string()
                } else {
                    String::new()
                }
            }
            "discnumber" => {
                if let Some(s) = tag.get_string(ItemKey::DiscNumber) {
                    s.trim().to_string()
                } else if let Some(n) = tag.disk() {
                    n.to_string()
                } else {
                    "1".to_string()
                }
            }
            "date" => {
                if let Some(s) = tag.get_string(ItemKey::RecordingDate).or_else(|| tag.get_string(ItemKey::Year)) {
                    s.trim().to_string()
                } else if let Some(d) = tag.date() {
                    d.to_string()
                } else {
                    String::new()
                }
            }
            "comment" => tag
                .comment()
                .map(|s| s.trim().to_string())
                .or_else(|| tag.get_string(ItemKey::Comment).map(|s| s.trim().to_string()))
                .unwrap_or_default(),
            other => {
                if let Some(item_key) = map_key_to_item_key(other) {
                    tag.get_string(item_key)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            }
        }
    }

    pub fn get_tag(&self, key: &str) -> String {
        let tag = self.read_tag().ok().flatten();
        let val = self.extract_field_value(tag.as_ref(), key);
        if key == "discnumber" && val.is_empty() {
            "1".to_string()
        } else {
            val
        }
    }

    pub fn set_tag(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut tagged_file = Probe::open(&self.path)?.guess_file_type()?.read()?;
        let tag_type = tagged_file.primary_tag_type();
        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                if let Some(first) = tagged_file.first_tag_mut() {
                    first
                } else {
                    tagged_file.insert_tag(Tag::new(tag_type));
                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        if let Some(item_key) = map_key_to_item_key(key) {
            tag.insert(TagItem::new(
                item_key,
                ItemValue::Text(value.trim().to_string()),
            ));
            tag.save_to_path(&self.path, WriteOptions::default())?;
        } else {
            eprintln!("Warning: Unsupported tag key `{key}` for {:?}", self.path);
        }

        Ok(())
    }

    pub fn replace_tag(&mut self, key: &str, target: &str, source: &str) -> Result<(), Box<dyn Error>> {
        let current = self.get_tag(key);
        let new_value = current.replace(target, source);
        self.set_tag(key, &new_value)
    }

    pub fn get_artworks(&self) -> Vec<Vec<u8>> {
        if let Ok(Some(tag)) = self.read_tag() {
            tag.pictures().iter().map(|p| p.data().to_vec()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn remove_artworks(&mut self) -> Result<(), Box<dyn Error>> {
        let mut tagged_file = Probe::open(&self.path)?.guess_file_type()?.read()?;
        let tag_type = tagged_file.primary_tag_type();
        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                if let Some(first) = tagged_file.first_tag_mut() {
                    first
                } else {
                    tagged_file.insert_tag(Tag::new(tag_type));
                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        while !tag.pictures().is_empty() {
            tag.remove_picture(0);
        }
        tag.save_to_path(&self.path, WriteOptions::default())?;
        Ok(())
    }

    pub fn set_artworks(&mut self, cover_path: &Path) -> Result<(), Box<dyn Error>> {
        let mut cover_file = std::fs::File::open(cover_path)?;
        let picture = Picture::from_reader(&mut cover_file)?;

        let mut tagged_file = Probe::open(&self.path)?.guess_file_type()?.read()?;
        let tag_type = tagged_file.primary_tag_type();
        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                if let Some(first) = tagged_file.first_tag_mut() {
                    first
                } else {
                    tagged_file.insert_tag(Tag::new(tag_type));
                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        tag.remove_picture_type(PictureType::CoverFront);
        while !tag.pictures().is_empty() {
            tag.remove_picture(0);
        }
        tag.push_picture(picture);
        tag.save_to_path(&self.path, WriteOptions::default())?;
        Ok(())
    }

    pub fn write_artwork_files(&self, tmp_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut files = Vec::new();
        for data in self.get_artworks() {
            let hash = crate::common::get_md5(&data);
            let filename = tmp_dir.join(format!("{hash}.jpg"));
            std::fs::write(&filename, data)?;
            files.push(filename);
        }
        Ok(files)
    }
}

pub fn get_audio_objects(folder: &Path) -> Result<Vec<AudioTrack>, Box<dyn Error>> {
    let mut tracks = Vec::new();
    let entries = std::fs::read_dir(folder)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if is_audio_file(&path) {
            tracks.push(AudioTrack::new(path));
        }
    }

    tracks.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(tracks)
}
