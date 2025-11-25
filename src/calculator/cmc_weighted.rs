use std::collections::HashMap;
use crate::deck::{Color, Deck, ManaBase};
use super::algorithms::ManaCalculator;

pub struct CmcWeightedCalculator;

impl ManaCalculator for CmcWeightedCalculator {
    fn calculate(&self, deck: &Deck) -> ManaBase {
        // For CMC-weighted calculation, we give more weight to colors with
        // lower average CMC cards. Since we don't have full card data,
        // we'll use pip intensity as a proxy - colors with higher pip intensity
        // (double/triple pips) likely have more demanding early game requirements.

        let mut mana_base = ManaBase::new();
        let total_symbols = deck.total_mana_symbols();

        if total_symbols == 0 || deck.colors.is_empty() {
            return mana_base;
        }

        // Calculate weighted percentages based on pip intensity
        let mut weighted_symbols: HashMap<Color, f64> = HashMap::new();
        for color in &deck.colors {
            let count = deck.mana_symbols.get(color).copied().unwrap_or(0) as f64;
            let intensity = deck.pip_intensity.get(color).copied().unwrap_or(0) as f64;

            // Weight formula: base count + (intensity * 0.5)
            // This gives extra weight to colors with intensive requirements
            let weighted = count + (intensity * 0.5);
            weighted_symbols.insert(*color, weighted);
        }

        let total_weighted: f64 = weighted_symbols.values().sum();

        // Calculate percentage for each color
        let mut color_percentages: HashMap<Color, f64> = HashMap::new();
        for color in &deck.colors {
            let weighted = weighted_symbols.get(color).copied().unwrap_or(0.0);
            let percentage = weighted / total_weighted;
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

        // Add recommendations for pip-intensive colors
        for color in &deck.colors {
            let intensity = deck.pip_intensity.get(color).copied().unwrap_or(0);
            if intensity >= 3 {
                mana_base.recommendations.push(format!(
                    "{} has high pip density ({} cards with {{{}}}{{{}}} or more). Consider additional {} sources or mana rocks.",
                    color.name(),
                    intensity,
                    color.symbol(),
                    color.symbol(),
                    color.name().to_lowercase()
                ));
            }
        }

        mana_base
    }

    fn name(&self) -> &'static str {
        "CMC-Weighted"
    }
}
