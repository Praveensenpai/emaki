use regex::Regex;
use std::sync::LazyLock;

static REEL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://(?:www\.)?instagram\.com/(?:reel|reels|p|share/reel)/([a-zA-Z0-9_-]+)")
        .expect("Failed to compile Instagram Reel regex pattern")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedReel {
    pub id: String,
    pub canonical_url: String,
}

pub struct ReelExtractor;

impl ReelExtractor {
    pub fn extract_all(text: &str) -> Vec<ExtractedReel> {
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();

        for cap in REEL_REGEX.captures_iter(text) {
            if let Some(id_match) = cap.get(1) {
                let id = id_match.as_str().to_string();
                if seen.insert(id.clone()) {
                    let canonical_url = format!("https://www.instagram.com/reel/{id}/");
                    results.push(ExtractedReel { id, canonical_url });
                }
            }
        }

        results
    }
}

pub fn format_caption(
    uploader: Option<&str>,
    description: Option<&str>,
    canonical_url: &str,
) -> String {
    let mut parts = Vec::new();

    if let Some(user) = uploader.filter(|u| !u.trim().is_empty()) {
        parts.push(format!("🎬 *Reel by @{user}*"));
    } else {
        parts.push("🎬 *Instagram Reel*".to_string());
    }

    if let Some(desc) = description.map(clean_description).filter(|d| !d.is_empty()) {
        let quote = desc
            .lines()
            .map(|l| format!("> {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(quote);
    }

    parts.push(format!("🔗 {canonical_url}"));
    parts.join("\n\n")
}

fn clean_description(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let without_hashtag_tail = strip_trailing_hashtags(trimmed);

    const MAX_CHARS: usize = 280;
    if without_hashtag_tail.chars().count() <= MAX_CHARS {
        without_hashtag_tail
    } else {
        let truncated: String = without_hashtag_tail.chars().take(MAX_CHARS).collect();
        format!("{truncated}...")
    }
}

fn strip_trailing_hashtags(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut end = lines.len();

    while end > 0 {
        let line = lines[end - 1].trim();
        if line.is_empty() {
            end -= 1;
            continue;
        }

        let words: Vec<&str> = line.split_whitespace().collect();
        let all_hashtags = !words.is_empty() && words.iter().all(|w| w.starts_with('#'));
        if all_hashtags {
            end -= 1;
        } else {
            break;
        }
    }

    lines[..end].join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_standard_reel() {
        let text =
            "Check this out https://www.instagram.com/reel/C8XYZ123_a/?igsh=abc1234 hilarious";
        let reels = ReelExtractor::extract_all(text);
        assert_eq!(reels.len(), 1);
        assert_eq!(reels[0].id, "C8XYZ123_a");
        assert_eq!(
            reels[0].canonical_url,
            "https://www.instagram.com/reel/C8XYZ123_a/"
        );
    }

    #[test]
    fn test_extract_post_format() {
        let text = "Look at this https://instagram.com/p/DFxyz999/";
        let reels = ReelExtractor::extract_all(text);
        assert_eq!(reels.len(), 1);
        assert_eq!(reels[0].id, "DFxyz999");
    }

    #[test]
    fn test_format_caption_with_metadata() {
        let caption = format_caption(
            Some("paisen"),
            Some("Hilarious cat jumping over couch\n#cat #funny #viral"),
            "https://www.instagram.com/reel/C8XYZ123_a/",
        );
        assert!(caption.contains("🎬 *Reel by @paisen*"));
        assert!(caption.contains("> Hilarious cat jumping over couch"));
        assert!(!caption.contains("#cat"));
        assert!(caption.contains("🔗 https://www.instagram.com/reel/C8XYZ123_a/"));
    }

    #[test]
    fn test_format_caption_fallback() {
        let caption = format_caption(None, None, "https://www.instagram.com/reel/123/");
        assert_eq!(
            caption,
            "🎬 *Instagram Reel*\n\n🔗 https://www.instagram.com/reel/123/"
        );
    }
}
