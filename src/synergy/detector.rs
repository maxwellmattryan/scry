#![allow(dead_code)]

use std::collections::HashMap;

use crate::input::DeckList;

use super::keywords::{extract_creature_types, extract_keywords, is_creature};
use super::themes::{classify_card_role, detect_card_themes, detect_tribal_themes};
use super::types::{
    CardSynergyProfile, SynergyEdge, SynergyMatrix, SynergyRelation, SynergyStats, Theme,
    ThemeAnalysis,
};

/// Trait for synergy detection implementations
pub trait SynergyDetector: Send + Sync {
    fn analyze(&self, deck: &DeckList) -> SynergyMatrix;
}

/// Rule-based synergy detector using pattern matching
pub struct RuleBasedDetector {
    /// Minimum number of cards for a theme to be significant
    min_theme_cards: u32,
}

impl RuleBasedDetector {
    pub fn new() -> Self {
        Self { min_theme_cards: 5 }
    }

    pub fn with_min_theme_cards(min_cards: u32) -> Self {
        Self {
            min_theme_cards: min_cards,
        }
    }

    /// Analyze card profiles and extract themes
    fn build_card_profiles(&self, deck: &DeckList) -> HashMap<String, CardSynergyProfile> {
        let mut profiles = HashMap::new();

        for entry in deck.mainboard() {
            if let Some(card) = &entry.card {
                let mut profile = CardSynergyProfile::new(card.name.clone());

                // Extract keywords
                profile.keywords = extract_keywords(card);

                // Detect themes
                let detected = detect_card_themes(card);
                for (theme, _confidence, role_hint) in detected {
                    profile.themes.push(theme.clone());
                    if profile.role.is_none() {
                        profile.role = role_hint;
                    }
                }

                profiles.insert(card.name.clone(), profile);
            }
        }

        profiles
    }

    /// Aggregate themes across all cards
    fn aggregate_themes(
        &self,
        profiles: &HashMap<String, CardSynergyProfile>,
        deck: &DeckList,
    ) -> Vec<ThemeAnalysis> {
        let mut theme_cards: HashMap<Theme, Vec<String>> = HashMap::new();

        // Collect cards per theme
        for (card_name, profile) in profiles {
            for theme in &profile.themes {
                theme_cards
                    .entry(theme.clone())
                    .or_default()
                    .push(card_name.clone());
            }
        }

        // Detect tribal themes
        let mut creature_type_counts: HashMap<String, u32> = HashMap::new();
        let mut total_creatures = 0u32;

        for entry in deck.mainboard() {
            if let Some(card) = &entry.card {
                if is_creature(card) {
                    total_creatures += entry.quantity;
                    for creature_type in extract_creature_types(card) {
                        *creature_type_counts.entry(creature_type).or_insert(0) += entry.quantity;
                    }
                }
            }
        }

        let tribal_themes = detect_tribal_themes(&creature_type_counts, total_creatures);
        for (theme, _count, _percentage) in tribal_themes {
            // Add tribal cards
            let tribe_name = if let Theme::Tribal(ref name) = theme {
                name.clone()
            } else {
                continue;
            };

            let mut tribal_cards = Vec::new();
            for entry in deck.mainboard() {
                if let Some(card) = &entry.card {
                    let types = extract_creature_types(card);
                    if types.contains(&tribe_name) {
                        tribal_cards.push(card.name.clone());
                    }
                }
            }

            if !tribal_cards.is_empty() {
                theme_cards.insert(theme, tribal_cards);
            }
        }

        // Convert to ThemeAnalysis
        let total_cards = deck.total_cards();
        let mut analyses: Vec<ThemeAnalysis> = theme_cards
            .into_iter()
            .filter(|(_, cards)| cards.len() as u32 >= self.min_theme_cards)
            .map(|(theme, cards)| {
                let card_count = cards.len() as u32;
                let mut analysis = ThemeAnalysis::new(theme.clone());
                analysis.card_count = card_count;
                analysis.percentage = card_count as f64 / total_cards as f64;

                // Classify cards into enablers/payoffs/support
                for card_name in cards {
                    if profiles.get(&card_name).is_some() {
                        if let Some(card_entry) = deck
                            .mainboard()
                            .find(|e| e.card.as_ref().is_some_and(|c| c.name == card_name))
                        {
                            if let Some(card) = &card_entry.card {
                                let role = classify_card_role(card, &theme);
                                match role {
                                    super::types::SynergyRole::Enabler => {
                                        analysis.enablers.push(card_name.clone())
                                    }
                                    super::types::SynergyRole::Payoff => {
                                        analysis.payoffs.push(card_name.clone())
                                    }
                                    super::types::SynergyRole::Support => {
                                        analysis.support.push(card_name.clone())
                                    }
                                }
                            }
                        }
                    }
                }

                analysis
            })
            .collect();

        // Sort by card count descending
        analyses.sort_by(|a, b| b.card_count.cmp(&a.card_count));
        analyses
    }

    /// Build synergy edges between cards
    fn build_edges(
        &self,
        _profiles: &HashMap<String, CardSynergyProfile>,
        themes: &[ThemeAnalysis],
    ) -> Vec<SynergyEdge> {
        let mut edges = Vec::new();
        let mut seen_pairs: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();

        // Cards sharing themes have synergy
        for theme_analysis in themes {
            let all_cards = theme_analysis.all_cards();

            for i in 0..all_cards.len() {
                for j in (i + 1)..all_cards.len() {
                    let card_a = all_cards[i].clone();
                    let card_b = all_cards[j].clone();

                    // Normalize pair order
                    let pair = if card_a < card_b {
                        (card_a.clone(), card_b.clone())
                    } else {
                        (card_b.clone(), card_a.clone())
                    };

                    // Skip if we've already added this pair
                    if seen_pairs.contains(&pair) {
                        continue;
                    }
                    seen_pairs.insert(pair);

                    // Determine relationship type
                    let relation = if theme_analysis.enablers.contains(&card_a)
                        && theme_analysis.payoffs.contains(&card_b)
                    {
                        SynergyRelation::Enables
                    } else if theme_analysis.payoffs.contains(&card_a)
                        && theme_analysis.enablers.contains(&card_b)
                    {
                        SynergyRelation::PayoffFor
                    } else {
                        SynergyRelation::Supports
                    };

                    edges.push(SynergyEdge {
                        card_a,
                        card_b,
                        relation,
                        themes: vec![theme_analysis.theme.clone()],
                        strength: 0.5,
                        reason: format!(
                            "Both support {} theme",
                            theme_analysis.theme.display_name()
                        ),
                    });
                }
            }
        }

        edges
    }

    /// Calculate synergy statistics
    fn calculate_stats(
        &self,
        profiles: &HashMap<String, CardSynergyProfile>,
        edges: &[SynergyEdge],
        deck: &DeckList,
    ) -> SynergyStats {
        let total_cards = deck.unique_cards();
        let possible_edges = if total_cards > 1 {
            (total_cards * (total_cards - 1)) / 2
        } else {
            0
        };

        // Cards with themes
        let themed_count = profiles.values().filter(|p| !p.themes.is_empty()).count();

        // Find orphan cards (no synergies)
        let cards_in_edges: std::collections::HashSet<_> = edges
            .iter()
            .flat_map(|e| vec![e.card_a.clone(), e.card_b.clone()])
            .collect();

        let orphan_cards: Vec<String> = profiles
            .keys()
            .filter(|name| !cards_in_edges.contains(*name))
            .cloned()
            .collect();

        // Find hub cards (most edges)
        let mut edge_counts: HashMap<String, u32> = HashMap::new();
        for edge in edges {
            *edge_counts.entry(edge.card_a.clone()).or_insert(0) += 1;
            *edge_counts.entry(edge.card_b.clone()).or_insert(0) += 1;
        }

        let mut sorted_by_edges: Vec<_> = edge_counts.into_iter().collect();
        sorted_by_edges.sort_by(|a, b| b.1.cmp(&a.1));

        let hub_cards: Vec<String> = sorted_by_edges
            .into_iter()
            .take(5)
            .map(|(name, _)| name)
            .collect();

        // Keyword distribution
        let mut keyword_dist: HashMap<String, u32> = HashMap::new();
        for profile in profiles.values() {
            for keyword in &profile.keywords {
                *keyword_dist.entry(keyword.display_name()).or_insert(0) += 1;
            }
        }

        SynergyStats {
            synergy_density: if possible_edges > 0 {
                edges.len() as f64 / possible_edges as f64
            } else {
                0.0
            },
            theme_coverage: if total_cards > 0 {
                themed_count as f64 / total_cards as f64
            } else {
                0.0
            },
            orphan_cards,
            hub_cards,
            theme_coherence: 0.0, // TODO: Calculate properly
            keyword_distribution: keyword_dist,
        }
    }

    /// Generate observations about the deck
    fn generate_observations(&self, themes: &[ThemeAnalysis], stats: &SynergyStats) -> Vec<String> {
        let mut observations = Vec::new();

        // Theme observations
        if let Some(primary) = themes.first() {
            observations.push(format!(
                "Primary theme: {} ({} cards, {:.0}% of deck)",
                primary.theme.display_name(),
                primary.card_count,
                primary.percentage * 100.0
            ));
        }

        if themes.len() > 1 {
            let secondary_themes: Vec<_> = themes
                .iter()
                .skip(1)
                .take(2)
                .map(|t| t.theme.display_name())
                .collect();
            observations.push(format!("Secondary themes: {}", secondary_themes.join(", ")));
        }

        // Synergy density observations
        if stats.synergy_density < 0.1 {
            observations.push(
                "Low synergy density. Consider adding more cards that work together.".to_string(),
            );
        } else if stats.synergy_density > 0.3 {
            observations.push("High synergy density! Cards work well together.".to_string());
        }

        // Theme coverage observations
        if stats.theme_coverage < 0.5 {
            observations.push(format!(
                "{:.0}% of cards don't contribute to any detected theme.",
                (1.0 - stats.theme_coverage) * 100.0
            ));
        }

        // Orphan card observations
        if !stats.orphan_cards.is_empty() && stats.orphan_cards.len() <= 5 {
            observations.push(format!(
                "Cards with no detected synergies: {}",
                stats.orphan_cards.join(", ")
            ));
        } else if stats.orphan_cards.len() > 5 {
            observations.push(format!(
                "{} cards have no detected synergies.",
                stats.orphan_cards.len()
            ));
        }

        observations
    }
}

impl Default for RuleBasedDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SynergyDetector for RuleBasedDetector {
    fn analyze(&self, deck: &DeckList) -> SynergyMatrix {
        let mut matrix = SynergyMatrix::new();

        // Set deck metadata
        matrix.deck_name = deck.name.clone();
        matrix.deck_format = deck.format.clone();
        matrix.total_cards = deck.total_cards();
        matrix.unique_cards = deck.unique_cards() as u32;

        // Build card profiles
        matrix.card_profiles = self.build_card_profiles(deck);

        // Aggregate themes
        matrix.detected_themes = self.aggregate_themes(&matrix.card_profiles, deck);

        // Set primary theme
        matrix.primary_theme = matrix.detected_themes.first().map(|t| t.theme.clone());

        // Build synergy edges
        matrix.edges = self.build_edges(&matrix.card_profiles, &matrix.detected_themes);

        // Calculate statistics
        matrix.stats = self.calculate_stats(&matrix.card_profiles, &matrix.edges, deck);

        // Generate observations
        matrix.observations = self.generate_observations(&matrix.detected_themes, &matrix.stats);

        matrix
    }
}

/// Factory function to get a synergy detector
pub fn get_detector() -> Box<dyn SynergyDetector> {
    Box::new(RuleBasedDetector::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Card;
    use crate::input::{DeckEntry, DeckSection, DeckSource};

    fn mock_card(name: &str, oracle_text: &str, type_line: &str) -> Card {
        Card {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            mana_cost: Some("{2}{W}".to_string()),
            cmc: 3.0,
            type_line: type_line.to_string(),
            oracle_text: Some(oracle_text.to_string()),
            power: None,
            toughness: None,
            colors: Some(vec!["W".to_string()]),
            color_identity: vec!["W".to_string()],
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

    fn create_token_deck() -> DeckList {
        let mut deck = DeckList::new(DeckSource::Manual);
        deck.name = Some("Token Test Deck".to_string());

        // Token generators
        let token_cards = vec![
            ("Raise the Alarm", "Create two 1/1 white Soldier creature tokens.", "Instant"),
            ("Spectral Procession", "Create three 1/1 white Spirit creature tokens with flying.", "Sorcery"),
            ("Lingering Souls", "Create two 1/1 white Spirit creature tokens with flying.\nFlashback {1}{B}", "Sorcery"),
            ("Intangible Virtue", "Creature tokens you control get +1/+1 and have vigilance.", "Enchantment"),
            ("Anointed Procession", "If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.", "Enchantment"),
        ];

        for (name, text, type_line) in token_cards {
            deck.entries.push(DeckEntry {
                quantity: 4,
                card_name: name.to_string(),
                card: Some(mock_card(name, text, type_line)),
                section: DeckSection::Mainboard,
            });
        }

        deck
    }

    #[test]
    fn test_detect_token_theme() {
        let deck = create_token_deck();
        let detector = RuleBasedDetector::new();
        let matrix = detector.analyze(&deck);

        // Should detect token theme
        assert!(matrix
            .detected_themes
            .iter()
            .any(|t| t.theme == Theme::Tokens));
    }

    #[test]
    fn test_primary_theme() {
        let deck = create_token_deck();
        let detector = RuleBasedDetector::new();
        let matrix = detector.analyze(&deck);

        assert!(matrix.primary_theme.is_some());
    }
}
