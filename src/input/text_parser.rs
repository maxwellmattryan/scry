use async_trait::async_trait;
use regex::Regex;

use super::decklist::{DeckEntry, DeckList, DeckListParser, DeckSection, DeckSource};

/// Parser for plain text decklist files
pub struct TextDecklistParser;

impl TextDecklistParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a section header from a line
    fn parse_section_header(line: &str) -> Option<DeckSection> {
        let lower = line.to_lowercase();
        let trimmed = lower.trim_start_matches("//").trim();

        if trimmed.starts_with("commander") {
            Some(DeckSection::Commander)
        } else if trimmed.starts_with("mainboard") || trimmed.starts_with("main") {
            Some(DeckSection::Mainboard)
        } else if trimmed.starts_with("sideboard") || trimmed.starts_with("side") {
            Some(DeckSection::Sideboard)
        } else if trimmed.starts_with("maybeboard") || trimmed.starts_with("maybe") {
            Some(DeckSection::Maybeboard)
        } else {
            None
        }
    }

    /// Parse a card line in various formats:
    /// - "4 Lightning Bolt"
    /// - "4x Lightning Bolt"
    /// - "Lightning Bolt x4"
    /// - "Lightning Bolt"
    fn parse_card_line(line: &str) -> Option<(u32, String)> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Pattern 1: "4 Lightning Bolt" or "4x Lightning Bolt"
        let re_prefix = Regex::new(r"^(\d+)x?\s+(.+)$").unwrap();
        if let Some(caps) = re_prefix.captures(trimmed) {
            let qty = caps.get(1)?.as_str().parse().ok()?;
            let name = caps.get(2)?.as_str().trim().to_string();
            return Some((qty, name));
        }

        // Pattern 2: "Lightning Bolt x4"
        let re_suffix = Regex::new(r"^(.+?)\s+x(\d+)$").unwrap();
        if let Some(caps) = re_suffix.captures(trimmed) {
            let name = caps.get(1)?.as_str().trim().to_string();
            let qty = caps.get(2)?.as_str().parse().ok()?;
            return Some((qty, name));
        }

        // Pattern 3: Just card name (assume quantity 1)
        if !trimmed.starts_with("//") && !trimmed.starts_with('#') {
            return Some((1, trimmed.to_string()));
        }

        None
    }
}

impl Default for TextDecklistParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeckListParser for TextDecklistParser {
    async fn parse(&self, path: &str) -> Result<DeckList, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

        let mut deck_list = DeckList::new(DeckSource::TextFile(path.to_string()));
        let mut current_section = DeckSection::Mainboard;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for section headers
            if trimmed.starts_with("//") || trimmed.ends_with(':') {
                if let Some(section) = Self::parse_section_header(trimmed) {
                    current_section = section;
                }
                continue;
            }

            // Skip comments
            if trimmed.starts_with('#') {
                continue;
            }

            // Try to parse as a card line
            if let Some((qty, name)) = Self::parse_card_line(trimmed) {
                deck_list.entries.push(DeckEntry {
                    quantity: qty,
                    card_name: name,
                    card: None,
                    section: current_section,
                });
            }
        }

        Ok(deck_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_card_line_prefix_quantity() {
        let result = TextDecklistParser::parse_card_line("4 Lightning Bolt");
        assert_eq!(result, Some((4, "Lightning Bolt".to_string())));
    }

    #[test]
    fn test_parse_card_line_prefix_with_x() {
        let result = TextDecklistParser::parse_card_line("4x Lightning Bolt");
        assert_eq!(result, Some((4, "Lightning Bolt".to_string())));
    }

    #[test]
    fn test_parse_card_line_suffix_quantity() {
        let result = TextDecklistParser::parse_card_line("Lightning Bolt x4");
        assert_eq!(result, Some((4, "Lightning Bolt".to_string())));
    }

    #[test]
    fn test_parse_card_line_no_quantity() {
        let result = TextDecklistParser::parse_card_line("Lightning Bolt");
        assert_eq!(result, Some((1, "Lightning Bolt".to_string())));
    }

    #[test]
    fn test_parse_section_header() {
        assert_eq!(
            TextDecklistParser::parse_section_header("// Commander"),
            Some(DeckSection::Commander)
        );
        assert_eq!(
            TextDecklistParser::parse_section_header("Sideboard:"),
            Some(DeckSection::Sideboard)
        );
        assert_eq!(
            TextDecklistParser::parse_section_header("// Mainboard"),
            Some(DeckSection::Mainboard)
        );
    }
}
