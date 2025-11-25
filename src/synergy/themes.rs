use crate::api::Card;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use super::keywords::{is_artifact, is_aura, is_enchantment, is_equipment, is_land};
use super::types::{CounterType, SynergyRole, Theme};

/// A rule for detecting a theme from oracle text
pub struct ThemeRule {
    pub theme: Theme,
    /// Patterns that indicate this theme (any match counts)
    pub oracle_patterns: Vec<Regex>,
    /// Type line patterns
    pub type_patterns: Vec<Regex>,
    /// Minimum confidence to consider a match (0.0 - 1.0)
    pub min_confidence: f64,
    /// Does this pattern indicate an enabler or payoff?
    pub role_hint: Option<SynergyRole>,
}

lazy_static! {
    pub static ref THEME_RULES: Vec<ThemeRule> = vec![
        // Tokens theme
        ThemeRule {
            theme: Theme::Tokens,
            oracle_patterns: vec![
                Regex::new(r"(?i)create.*token").unwrap(),
                Regex::new(r"(?i)token.*enters").unwrap(),
                Regex::new(r"(?i)for each.*token").unwrap(),
                Regex::new(r"(?i)tokens you control").unwrap(),
                Regex::new(r"(?i)creature tokens").unwrap(),
                Regex::new(r"(?i)put.*token").unwrap(),
                Regex::new(r"(?i)tokens get").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // +1/+1 Counters theme
        ThemeRule {
            theme: Theme::Counters(CounterType::PlusOne),
            oracle_patterns: vec![
                Regex::new(r"(?i)\+1/\+1 counter").unwrap(),
                Regex::new(r"(?i)proliferate").unwrap(),
                Regex::new(r"(?i)with.*counters on").unwrap(),
                Regex::new(r"(?i)counter on it").unwrap(),
                Regex::new(r"(?i)distribute.*counters").unwrap(),
                Regex::new(r"(?i)move.*counter").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Graveyard theme
        ThemeRule {
            theme: Theme::Graveyard,
            oracle_patterns: vec![
                Regex::new(r"(?i)from your graveyard").unwrap(),
                Regex::new(r"(?i)in your graveyard").unwrap(),
                Regex::new(r"(?i)return.*from.*graveyard").unwrap(),
                Regex::new(r"(?i)cards in your graveyard").unwrap(),
                Regex::new(r"(?i)\bflashback\b").unwrap(),
                Regex::new(r"(?i)\bunearth\b").unwrap(),
                Regex::new(r"(?i)\bescape\b").unwrap(),
                Regex::new(r"(?i)\bdelve\b").unwrap(),
                Regex::new(r"(?i)exile.*from your graveyard").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Sacrifice theme
        ThemeRule {
            theme: Theme::Sacrifice,
            oracle_patterns: vec![
                Regex::new(r"(?i)sacrifice a").unwrap(),
                Regex::new(r"(?i)sacrifice another").unwrap(),
                Regex::new(r"(?i)when.*dies").unwrap(),
                Regex::new(r"(?i)whenever.*dies").unwrap(),
                Regex::new(r"(?i)sacrifice.*:").unwrap(),
                Regex::new(r"(?i)you may sacrifice").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Blink/Flicker theme
        ThemeRule {
            theme: Theme::Blink,
            oracle_patterns: vec![
                Regex::new(r"(?i)exile.*return.*to the battlefield").unwrap(),
                Regex::new(r"(?i)flicker").unwrap(),
                Regex::new(r"(?i)exile.*then return").unwrap(),
                Regex::new(r"(?i)when.*enters the battlefield").unwrap(),
                Regex::new(r"(?i)whenever.*enters the battlefield").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.15,
            role_hint: None,
        },

        // Ramp theme
        ThemeRule {
            theme: Theme::Ramp,
            oracle_patterns: vec![
                Regex::new(r"(?i)search your library for.*land").unwrap(),
                Regex::new(r"(?i)add.*mana").unwrap(),
                Regex::new(r"(?i)put.*land.*onto the battlefield").unwrap(),
                Regex::new(r"(?i)additional land").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Card Draw theme
        ThemeRule {
            theme: Theme::Draw,
            oracle_patterns: vec![
                Regex::new(r"(?i)draw.*card").unwrap(),
                Regex::new(r"(?i)draws.*card").unwrap(),
                Regex::new(r"(?i)whenever you draw").unwrap(),
                Regex::new(r"(?i)for each card you").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Removal theme
        ThemeRule {
            theme: Theme::Removal,
            oracle_patterns: vec![
                Regex::new(r"(?i)destroy target").unwrap(),
                Regex::new(r"(?i)exile target").unwrap(),
                Regex::new(r"(?i)deals.*damage to").unwrap(),
                Regex::new(r"(?i)target creature gets -").unwrap(),
                Regex::new(r"(?i)return target.*to.*owner").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Lifegain theme
        ThemeRule {
            theme: Theme::Lifegain,
            oracle_patterns: vec![
                Regex::new(r"(?i)you gain.*life").unwrap(),
                Regex::new(r"(?i)gain.*life").unwrap(),
                Regex::new(r"(?i)whenever you gain life").unwrap(),
                Regex::new(r"(?i)\blifelink\b").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Discard theme
        ThemeRule {
            theme: Theme::Discard,
            oracle_patterns: vec![
                Regex::new(r"(?i)target.*discard").unwrap(),
                Regex::new(r"(?i)opponent.*discard").unwrap(),
                Regex::new(r"(?i)whenever.*discard").unwrap(),
                Regex::new(r"(?i)discard a card").unwrap(),
                Regex::new(r"(?i)\bmadness\b").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Mill theme
        ThemeRule {
            theme: Theme::Mill,
            oracle_patterns: vec![
                Regex::new(r"(?i)mill").unwrap(),
                Regex::new(r"(?i)put.*cards from.*library into.*graveyard").unwrap(),
                Regex::new(r"(?i)cards in your library").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Reanimator theme
        ThemeRule {
            theme: Theme::Reanimator,
            oracle_patterns: vec![
                Regex::new(r"(?i)return.*creature.*from.*graveyard.*to the battlefield").unwrap(),
                Regex::new(r"(?i)put.*creature.*from.*graveyard onto the battlefield").unwrap(),
                Regex::new(r"(?i)reanimate").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Spellslinger theme (instants/sorceries matter)
        ThemeRule {
            theme: Theme::Spellslinger,
            oracle_patterns: vec![
                Regex::new(r"(?i)whenever you cast an instant or sorcery").unwrap(),
                Regex::new(r"(?i)instant and sorcery").unwrap(),
                Regex::new(r"(?i)noncreature spell").unwrap(),
                Regex::new(r"(?i)\bstorm\b").unwrap(),
                Regex::new(r"(?i)copy.*instant").unwrap(),
                Regex::new(r"(?i)copy.*sorcery").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.1,
            role_hint: None,
        },

        // Aristocrats theme (death triggers + sacrifice)
        ThemeRule {
            theme: Theme::Aristocrats,
            oracle_patterns: vec![
                Regex::new(r"(?i)whenever.*creature.*dies").unwrap(),
                Regex::new(r"(?i)whenever another.*dies").unwrap(),
                Regex::new(r"(?i)sacrifice.*creature").unwrap(),
            ],
            type_patterns: vec![],
            min_confidence: 0.15,
            role_hint: None,
        },

        // Voltron theme
        ThemeRule {
            theme: Theme::Voltron,
            oracle_patterns: vec![
                Regex::new(r"(?i)equipped creature").unwrap(),
                Regex::new(r"(?i)enchanted creature").unwrap(),
                Regex::new(r"(?i)commander deals combat damage").unwrap(),
            ],
            type_patterns: vec![
                Regex::new(r"(?i)Equipment").unwrap(),
                Regex::new(r"(?i)Aura").unwrap(),
            ],
            min_confidence: 0.1,
            role_hint: None,
        },
    ];
}

/// Detect themes for a single card
pub fn detect_card_themes(card: &Card) -> Vec<(Theme, f64, Option<SynergyRole>)> {
    let mut themes = Vec::new();

    // Combine all oracle text
    let all_text: String = card
        .all_oracle_text()
        .into_iter()
        .collect::<Vec<_>>()
        .join(" ");

    let all_types: String = card.all_type_lines().join(" ");

    for rule in THEME_RULES.iter() {
        let oracle_matches: usize = rule
            .oracle_patterns
            .iter()
            .filter(|p| p.is_match(&all_text))
            .count();

        let type_matches: usize = rule
            .type_patterns
            .iter()
            .filter(|p| p.is_match(&all_types))
            .count();

        let total_patterns = rule.oracle_patterns.len() + rule.type_patterns.len();
        let total_matches = oracle_matches + type_matches;

        if total_matches > 0 {
            let confidence = total_matches as f64 / total_patterns as f64;
            if confidence >= rule.min_confidence {
                themes.push((rule.theme.clone(), confidence, rule.role_hint));
            }
        }
    }

    // Add type-based themes
    if is_equipment(card) {
        themes.push((Theme::Equipment, 1.0, Some(SynergyRole::Support)));
    }
    if is_aura(card) {
        themes.push((Theme::Auras, 1.0, Some(SynergyRole::Support)));
    }
    if is_artifact(card) && !is_equipment(card) {
        // Check for "artifacts matter" text
        if all_text.to_lowercase().contains("artifact") {
            themes.push((Theme::Artifacts, 0.5, None));
        }
    }
    if is_enchantment(card) && !is_aura(card) && all_text.to_lowercase().contains("enchantment") {
        themes.push((Theme::Enchantments, 0.5, None));
    }
    if is_land(card) && all_text.to_lowercase().contains("land") {
        themes.push((Theme::Lands, 0.5, None));
    }

    themes
}

/// Detect tribal themes from creature type distribution
pub fn detect_tribal_themes(
    creature_type_counts: &HashMap<String, u32>,
    total_creatures: u32,
) -> Vec<(Theme, u32, f64)> {
    let mut tribal_themes = Vec::new();

    // Minimum threshold for tribal (8+ creatures or 30%+ of creature base)
    let min_count = 8u32;
    let min_percentage = 0.30;

    for (creature_type, count) in creature_type_counts {
        let percentage = if total_creatures > 0 {
            *count as f64 / total_creatures as f64
        } else {
            0.0
        };

        if *count >= min_count || percentage >= min_percentage {
            tribal_themes.push((Theme::Tribal(creature_type.clone()), *count, percentage));
        }
    }

    // Sort by count descending
    tribal_themes.sort_by(|a, b| b.1.cmp(&a.1));
    tribal_themes
}

/// Determine if a card is an enabler, payoff, or support for a theme
pub fn classify_card_role(card: &Card, theme: &Theme) -> SynergyRole {
    let all_text: String = card
        .all_oracle_text()
        .into_iter()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    match theme {
        Theme::Tokens => {
            if all_text.contains("create") && all_text.contains("token") {
                SynergyRole::Enabler
            } else if all_text.contains("tokens you control")
                || all_text.contains("for each")
                || all_text.contains("tokens get")
            {
                SynergyRole::Payoff
            } else {
                SynergyRole::Support
            }
        }
        Theme::Counters(_) => {
            if all_text.contains("put") && all_text.contains("counter") {
                SynergyRole::Enabler
            } else if all_text.contains("with") && all_text.contains("counter") {
                SynergyRole::Payoff
            } else {
                SynergyRole::Support
            }
        }
        Theme::Graveyard => {
            if all_text.contains("mill") || all_text.contains("discard") {
                SynergyRole::Enabler
            } else if all_text.contains("from your graveyard")
                || all_text.contains("flashback")
                || all_text.contains("escape")
            {
                SynergyRole::Payoff
            } else {
                SynergyRole::Support
            }
        }
        Theme::Sacrifice => {
            if all_text.contains("create") && all_text.contains("token") {
                SynergyRole::Enabler
            } else if all_text.contains("when") && all_text.contains("dies") {
                SynergyRole::Payoff
            } else {
                SynergyRole::Support
            }
        }
        Theme::Lifegain => {
            if all_text.contains("gain") && all_text.contains("life") {
                SynergyRole::Enabler
            } else if all_text.contains("whenever you gain life") {
                SynergyRole::Payoff
            } else {
                SynergyRole::Support
            }
        }
        _ => SynergyRole::Support,
    }
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
    fn test_detect_token_theme() {
        let card = mock_card("Create two 1/1 white Soldier creature tokens.", "Sorcery");
        let themes = detect_card_themes(&card);
        assert!(themes.iter().any(|(t, _, _)| *t == Theme::Tokens));
    }

    #[test]
    fn test_detect_graveyard_theme() {
        let card = mock_card(
            "Return target creature card from your graveyard to your hand.",
            "Sorcery",
        );
        let themes = detect_card_themes(&card);
        assert!(themes.iter().any(|(t, _, _)| *t == Theme::Graveyard));
    }

    #[test]
    fn test_detect_sacrifice_theme() {
        let card = mock_card(
            "Whenever a creature you control dies, draw a card.",
            "Enchantment",
        );
        let themes = detect_card_themes(&card);
        assert!(themes.iter().any(|(t, _, _)| *t == Theme::Sacrifice));
    }
}
