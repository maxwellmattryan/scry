//! Bridge between curve analysis and mana base calculation.
//!
//! This module provides functions to convert curve analysis data into
//! inputs for the mana calculator, and to determine appropriate land counts.

use crate::calculator::get_calculator;
use crate::curve::{CurveAnalysis, LandCountSource};
use crate::deck::{guild_name, Algorithm, Color, Deck, DualLand, Format, ManaBase};
use crate::input::DeckList;
use std::collections::HashMap;

/// Determines target land count using priority:
/// 1. User-provided --lands flag
/// 2. Detected from deck (count existing lands)
/// 3. Format-based default
pub fn determine_land_count(
    deck_list: &DeckList,
    user_lands: Option<u32>,
    excludes_lands: bool,
) -> (u32, LandCountSource) {
    // Priority 1: User provided
    if let Some(lands) = user_lands {
        return (lands, LandCountSource::UserProvided);
    }

    // Priority 2: Detect from deck (if not excludes_lands)
    if !excludes_lands {
        let detected = deck_list.count_lands();
        if detected > 0 {
            return (detected, LandCountSource::DetectedFromDeck(detected));
        }
    }

    // Priority 3: Format default
    let format = detect_format_from_deck(deck_list);
    (
        format.default_lands(),
        LandCountSource::FormatDefault(format.name().to_string()),
    )
}

/// Detect format from deck metadata or size heuristics
pub fn detect_format_from_deck(deck_list: &DeckList) -> Format {
    // Check if format string is provided
    if let Some(format_str) = &deck_list.format {
        let lower = format_str.to_lowercase();
        if lower.contains("commander") || lower.contains("edh") {
            return Format::Commander;
        }
        if lower.contains("standard") {
            return Format::Standard;
        }
        if lower.contains("modern") {
            return Format::Modern;
        }
        if lower.contains("limited") || lower.contains("draft") || lower.contains("sealed") {
            return Format::Limited;
        }
    }

    // Fallback to card count heuristics
    let total = deck_list.total_cards();
    let has_commander = deck_list.commanders().count() > 0;

    if has_commander || total >= 99 {
        Format::Commander
    } else if total <= 45 {
        Format::Limited
    } else {
        Format::Standard // Default to 60-card format
    }
}

/// Detect multi-colored lands from the deck and group them by color combination
pub fn detect_dual_lands(deck_list: &DeckList) -> Vec<DualLand> {
    // Group lands by their color identity (sorted for consistent keys)
    let mut land_groups: HashMap<Vec<Color>, u32> = HashMap::new();

    for entry in deck_list.mainboard() {
        let Some(card) = &entry.card else {
            continue;
        };

        // Check if it's a land
        if !card.type_line.to_lowercase().contains("land") {
            continue;
        }

        // Check if it has multiple colors in its identity
        if card.color_identity.len() < 2 {
            continue;
        }

        // Convert color identity strings to Color enum
        let mut colors: Vec<Color> = card
            .color_identity
            .iter()
            .filter_map(|s| Color::from_symbol(s))
            .collect();

        // Sort colors for consistent grouping
        colors.sort_by_key(|c| match c {
            Color::White => 0,
            Color::Blue => 1,
            Color::Black => 2,
            Color::Red => 3,
            Color::Green => 4,
            Color::Colorless => 5,
        });

        // Add to the group
        *land_groups.entry(colors).or_insert(0) += entry.quantity;
    }

    // Convert groups to DualLand structs
    land_groups
        .into_iter()
        .map(|(colors, count)| {
            // Generate a name for the land group
            let name = if colors.len() == 2 {
                guild_name(&colors)
                    .map(|g| format!("{g} lands"))
                    .unwrap_or_else(|| {
                        let symbols: Vec<_> = colors.iter().map(|c| c.symbol()).collect();
                        format!("{} lands", symbols.join("/"))
                    })
            } else {
                let symbols: Vec<_> = colors.iter().map(|c| c.symbol()).collect();
                format!("{}-color lands", symbols.join("/"))
            };

            DualLand::new(name, colors, count)
        })
        .collect()
}

/// Build a Deck struct from CurveAnalysis for calculator input
pub fn build_deck_from_analysis(
    analysis: &CurveAnalysis,
    deck_list: &DeckList,
    target_lands: u32,
    format: Format,
) -> Deck {
    let mut deck = Deck::new(format);

    deck.total_cards = analysis.total_cards;
    deck.target_lands = target_lands;
    deck.colors = analysis.pip_breakdown.colors();
    deck.mana_symbols = analysis.pip_breakdown.to_mana_symbols();

    // Detect dual lands from the actual deck
    deck.dual_lands = detect_dual_lands(deck_list);

    deck
}

/// Calculate mana base recommendation from curve analysis
pub fn calculate_mana_base(
    analysis: &CurveAnalysis,
    deck_list: &DeckList,
    target_lands: u32,
    format: Format,
    algorithm: Algorithm,
) -> ManaBase {
    let deck = build_deck_from_analysis(analysis, deck_list, target_lands, format);
    let calculator = get_calculator(algorithm);
    calculator.calculate(&deck)
}
