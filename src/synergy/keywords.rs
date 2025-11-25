#![allow(dead_code)]

use crate::api::Card;
use lazy_static::lazy_static;
use regex::Regex;

use super::types::Keyword;

lazy_static! {
    /// Patterns for detecting keywords in oracle text
    static ref KEYWORD_PATTERNS: Vec<(Regex, Keyword)> = vec![
        // Evergreen keywords
        (Regex::new(r"(?i)\bflying\b").unwrap(), Keyword::Flying),
        (Regex::new(r"(?i)\btrample\b").unwrap(), Keyword::Trample),
        (Regex::new(r"(?i)\bhaste\b").unwrap(), Keyword::Haste),
        (Regex::new(r"(?i)\bvigilance\b").unwrap(), Keyword::Vigilance),
        (Regex::new(r"(?i)\bdeathtouch\b").unwrap(), Keyword::Deathtouch),
        (Regex::new(r"(?i)\blifelink\b").unwrap(), Keyword::Lifelink),
        (Regex::new(r"(?i)\bfirst strike\b").unwrap(), Keyword::FirstStrike),
        (Regex::new(r"(?i)\bdouble strike\b").unwrap(), Keyword::DoubleStrike),
        (Regex::new(r"(?i)\bmenace\b").unwrap(), Keyword::Menace),
        (Regex::new(r"(?i)\breach\b").unwrap(), Keyword::Reach),
        (Regex::new(r"(?i)\bflash\b").unwrap(), Keyword::Flash),
        (Regex::new(r"(?i)\bhexproof\b").unwrap(), Keyword::Hexproof),
        (Regex::new(r"(?i)\bindestructible\b").unwrap(), Keyword::Indestructible),
        (Regex::new(r"(?i)\bdefender\b").unwrap(), Keyword::Defender),
        (Regex::new(r"(?i)\bward\b").unwrap(), Keyword::Ward),

        // Set mechanics
        (Regex::new(r"(?i)\bflashback\b").unwrap(), Keyword::Flashback),
        (Regex::new(r"(?i)\bunearth\b").unwrap(), Keyword::Unearth),
        (Regex::new(r"(?i)\bescape\b").unwrap(), Keyword::Escape),
        (Regex::new(r"(?i)\bdelve\b").unwrap(), Keyword::Delve),
        (Regex::new(r"(?i)\bconvoke\b").unwrap(), Keyword::Convoke),
        (Regex::new(r"(?i)\bcascade\b").unwrap(), Keyword::Cascade),
        (Regex::new(r"(?i)\bstorm\b").unwrap(), Keyword::Storm),
        (Regex::new(r"(?i)\bproliferate\b").unwrap(), Keyword::Proliferate),
        (Regex::new(r"(?i)\blandfall\b").unwrap(), Keyword::Landfall),
        (Regex::new(r"(?i)\bconstellation\b").unwrap(), Keyword::Constellation),
        (Regex::new(r"(?i)\bdevotion\b").unwrap(), Keyword::Devotion),
        (Regex::new(r"(?i)\bannihilator\b").unwrap(), Keyword::Annihilator),
        (Regex::new(r"(?i)\binfect\b").unwrap(), Keyword::Infect),
        (Regex::new(r"(?i)\bwither\b").unwrap(), Keyword::Wither),
        (Regex::new(r"(?i)\baffinity\b").unwrap(), Keyword::Affinity),
        (Regex::new(r"(?i)\bmadness\b").unwrap(), Keyword::Madness),
        (Regex::new(r"(?i)\boverload\b").unwrap(), Keyword::Overload),
        (Regex::new(r"(?i)\bcrew\b").unwrap(), Keyword::Crew),
        (Regex::new(r"(?i)\bequip\b").unwrap(), Keyword::Equip),
    ];

    /// Common creature types for tribal detection
    static ref COMMON_CREATURE_TYPES: Vec<&'static str> = vec![
        "Human", "Elf", "Goblin", "Zombie", "Vampire", "Wizard", "Soldier",
        "Knight", "Dragon", "Angel", "Demon", "Beast", "Elemental", "Spirit",
        "Warrior", "Cleric", "Rogue", "Shaman", "Merfolk", "Bird", "Cat",
        "Dinosaur", "Sliver", "Ally", "Eldrazi", "Faerie", "Giant", "Horror",
        "Hydra", "Insect", "Ninja", "Pirate", "Rat", "Samurai", "Serpent",
        "Skeleton", "Spider", "Treefolk", "Werewolf", "Wolf", "Artifact",
    ];
}

/// Extract keywords from a card's oracle text
pub fn extract_keywords(card: &Card) -> Vec<Keyword> {
    let mut keywords = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Combine all oracle text from main card and faces
    let all_text: String = card
        .all_oracle_text()
        .into_iter()
        .collect::<Vec<_>>()
        .join(" ");

    for (pattern, keyword) in KEYWORD_PATTERNS.iter() {
        if pattern.is_match(&all_text) {
            // Use display_name as key to avoid duplicates
            let key = keyword.display_name();
            if !seen.contains(&key) {
                seen.insert(key);
                keywords.push(keyword.clone());
            }
        }
    }

    keywords
}

/// Extract creature types from a card's type line
pub fn extract_creature_types(card: &Card) -> Vec<String> {
    let mut types = Vec::new();

    for type_line in card.all_type_lines() {
        // Check if it's a creature
        if !type_line.contains("Creature") {
            continue;
        }

        // Extract subtypes after the dash
        if let Some(subtypes_str) = type_line.split(" — ").nth(1) {
            for word in subtypes_str.split_whitespace() {
                // Check if it's a recognized creature type
                for &creature_type in COMMON_CREATURE_TYPES.iter() {
                    if word.eq_ignore_ascii_case(creature_type) {
                        types.push(creature_type.to_string());
                    }
                }
            }
        }
    }

    types.sort();
    types.dedup();
    types
}

/// Check if a card is a creature
pub fn is_creature(card: &Card) -> bool {
    card.all_type_lines().iter().any(|t| t.contains("Creature"))
}

/// Check if a card is an instant or sorcery
pub fn is_instant_or_sorcery(card: &Card) -> bool {
    card.all_type_lines()
        .iter()
        .any(|t| t.contains("Instant") || t.contains("Sorcery"))
}

/// Check if a card is a land
pub fn is_land(card: &Card) -> bool {
    card.all_type_lines().iter().any(|t| t.contains("Land"))
}

/// Check if a card is an artifact
pub fn is_artifact(card: &Card) -> bool {
    card.all_type_lines().iter().any(|t| t.contains("Artifact"))
}

/// Check if a card is an enchantment
pub fn is_enchantment(card: &Card) -> bool {
    card.all_type_lines()
        .iter()
        .any(|t| t.contains("Enchantment"))
}

/// Check if a card is an equipment
pub fn is_equipment(card: &Card) -> bool {
    card.all_type_lines()
        .iter()
        .any(|t| t.contains("Equipment"))
}

/// Check if a card is an aura
pub fn is_aura(card: &Card) -> bool {
    card.all_type_lines().iter().any(|t| t.contains("Aura"))
}

/// Check if a card is a planeswalker
pub fn is_planeswalker(card: &Card) -> bool {
    card.all_type_lines()
        .iter()
        .any(|t| t.contains("Planeswalker"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_card(oracle_text: &str, type_line: &str) -> Card {
        Card {
            id: "test".to_string(),
            name: "Test Card".to_string(),
            mana_cost: None,
            cmc: 0.0,
            type_line: type_line.to_string(),
            oracle_text: Some(oracle_text.to_string()),
            power: None,
            toughness: None,
            colors: None,
            color_identity: vec![],
            set: "TST".to_string(),
            set_name: "Test Set".to_string(),
            rarity: "common".to_string(),
            prices: None,
            legalities: std::collections::HashMap::new(),
            image_uris: None,
            scryfall_uri: "https://scryfall.com".to_string(),
            card_faces: None,
            layout: None,
        }
    }

    #[test]
    fn test_extract_keywords_flying() {
        let card = mock_card("Flying", "Creature — Bird");
        let keywords = extract_keywords(&card);
        assert!(keywords.contains(&Keyword::Flying));
    }

    #[test]
    fn test_extract_keywords_multiple() {
        let card = mock_card("Flying, vigilance, lifelink", "Creature — Angel");
        let keywords = extract_keywords(&card);
        assert!(keywords.contains(&Keyword::Flying));
        assert!(keywords.contains(&Keyword::Vigilance));
        assert!(keywords.contains(&Keyword::Lifelink));
    }

    #[test]
    fn test_extract_creature_types() {
        let card = mock_card("", "Creature — Human Wizard");
        let types = extract_creature_types(&card);
        assert!(types.contains(&"Human".to_string()));
        assert!(types.contains(&"Wizard".to_string()));
    }

    #[test]
    fn test_is_creature() {
        let card = mock_card("", "Creature — Elf");
        assert!(is_creature(&card));

        let spell = mock_card("", "Instant");
        assert!(!is_creature(&spell));
    }
}
