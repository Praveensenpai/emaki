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
    fn test_no_match() {
        let text = "No links here https://google.com/search?q=test";
        let reels = ReelExtractor::extract_all(text);
        assert!(reels.is_empty());
    }
}
