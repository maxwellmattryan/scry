use super::algorithms::ManaCalculator;
use crate::deck::{Color, Deck, ManaBase};
use std::collections::HashMap;

pub struct SimpleCalculator;

impl ManaCalculator for SimpleCalculator {
    fn calculate(&self, deck: &Deck) -> ManaBase {
        let mut mana_base = ManaBase::new();
        let total_symbols = deck.total_mana_symbols();

        if total_symbols == 0 || deck.colors.is_empty() {
            return mana_base;
        }

        // Calculate percentage for each color
        let mut color_percentages: HashMap<Color, f64> = HashMap::new();
        for color in &deck.colors {
            let count = deck.mana_symbols.get(color).copied().unwrap_or(0);
            let percentage = (count as f64) / (total_symbols as f64);
            color_percentages.insert(*color, percentage);
        }
        mana_base.color_percentages = color_percentages.clone();

        // Copy dual lands from deck
        mana_base.duals = deck.dual_lands.clone();

        // Calculate effective color sources from dual lands
        let mut dual_sources: HashMap<Color, f64> = HashMap::new();
        for dual in &deck.dual_lands {
            for color in &dual.colors {
                *dual_sources.entry(*color).or_insert(0.0) += dual.count as f64;
            }
        }

        // Calculate basic lands needed
        let basic_slots = deck.basic_land_slots() as f64;
        let mut basic_counts: HashMap<Color, f64> = HashMap::new();

        for color in &deck.colors {
            let percentage = color_percentages.get(color).copied().unwrap_or(0.0);
            let target_sources = percentage * (deck.target_lands as f64);
            let dual_contribution = dual_sources.get(color).copied().unwrap_or(0.0);
            let basics_needed = (target_sources - dual_contribution).max(0.0);
            basic_counts.insert(*color, basics_needed);
        }

        // Normalize to fit within basic slots
        let total_basics_needed: f64 = basic_counts.values().sum();
        if total_basics_needed > 0.0 {
            let scale = basic_slots / total_basics_needed;
            for color in &deck.colors {
                if let Some(count) = basic_counts.get(color) {
                    let adjusted = (count * scale).round() as u32;
                    if adjusted > 0 {
                        mana_base.basics.insert(*color, adjusted);
                    }
                }
            }
        }

        // Adjust rounding errors
        let current_total: u32 = mana_base.basics.values().sum();
        let target_total = deck.basic_land_slots();
        if current_total != target_total && !mana_base.basics.is_empty() {
            let diff = target_total as i32 - current_total as i32;
            // Find the color with highest percentage and adjust
            if let Some((&color, _)) = color_percentages
                .iter()
                .filter(|(c, _)| deck.colors.contains(c))
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            {
                if let Some(count) = mana_base.basics.get_mut(&color) {
                    *count = (*count as i32 + diff).max(0) as u32;
                }
            }
        }

        mana_base
    }
}
