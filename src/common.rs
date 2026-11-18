use deunicode::deunicode;
use md5::{Digest, Md5};
use regex::Regex;
use std::sync::LazyLock;
use zhconv::{zhconv, Variant};

pub static INVALID_KEYWORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut keywords = vec![
        "hifizg.com",
        "-hifizg.com",
        "hiresyys.com",
        "-hiresyys.com",
        "hiresdsd.com",
        "-hiresdsd.com",
        "sonyhires.com",
        "-sonyhires.com",
        "superdsd.com",
        "-superdsd.com",
        "HiRes音乐社",
    ];
    keywords.sort_by(|a, b| b.len().cmp(&a.len()));
    keywords
});

static SANITIZE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[/:"\';?]"#).expect("invalid regex")
});

pub fn get_md5(bytes: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn name_unify(s: &str) -> String {
    zhconv(s.trim(), Variant::ZhCN).to_lowercase()
}

pub fn pad_string(s: &str, length: usize) -> String {
    let char_count = s.chars().count();
    if char_count > length {
        if length > 3 {
            let prefix: String = s.chars().take(length - 3).collect();
            format!("{prefix}...")
        } else {
            s.chars().take(length).collect()
        }
    } else {
        format!("{:<width$}", s, width = length)
    }
}

pub fn sanitize_filename(s: &str) -> String {
    let transliterated = deunicode(s);
    let replaced = SANITIZE_REGEX.replace_all(&transliterated, "_");
    replaced
        .replace('[', "(")
        .replace(']', ")")
        .replace('·', "-")
        .replace('<', "(")
        .replace('>', ")")
        .replace('《', "(")
        .replace('》', ")")
        .replace('`', "")
        .replace('*', "")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_unify() {
        assert_eq!(name_unify("  憂鬱的臺灣烏龜  "), "忧郁的台湾乌龟");
        assert_eq!(name_unify("HELLO WORLD"), "hello world");
    }

    #[test]
    fn test_pad_string() {
        assert_eq!(pad_string("short", 10), "short     ");
        assert_eq!(pad_string("very long filename text", 10), "very lo...");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Track/Name: Test?"), "Track_Name_ Test_");
        assert_eq!(sanitize_filename("《Song》 [Remix]"), "((Song)) (Remix)");
    }
}

