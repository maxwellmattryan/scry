use super::algorithms::ManaCalculator;
use crate::deck::{Color, Deck, ManaBase};
use std::collections::HashMap;

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

        // Calculate baseline basics (as if no duals existed)
        let basic_slots = deck.basic_land_slots() as f64;

        // Calculate remaining basics needed after accounting for duals
        let mut remaining: HashMap<Color, f64> = HashMap::new();
        for color in &deck.colors {
            let percentage = color_percentages.get(color).copied().unwrap_or(0.0);
            let baseline = percentage * (deck.target_lands as f64);
            let dual_contribution = dual_sources.get(color).copied().unwrap_or(0.0);
            remaining.insert(*color, (baseline - dual_contribution).max(0.0));
        }

        let total_remaining: f64 = remaining.values().sum();

        // Calculate final basic counts based on constraint
        let mut basic_counts: HashMap<Color, f64> = HashMap::new();
        if total_remaining >= basic_slots {
            // Not enough slots: scale down proportionally
            let scale = if total_remaining > 0.0 {
                basic_slots / total_remaining
            } else {
                0.0
            };
            for color in &deck.colors {
                let count = remaining.get(color).copied().unwrap_or(0.0);
                basic_counts.insert(*color, count * scale);
            }
        } else {
            // Extra slots: fill remaining needs, distribute extras by original percentage
            let extras = basic_slots - total_remaining;
            for color in &deck.colors {
                let need = remaining.get(color).copied().unwrap_or(0.0);
                let percentage = color_percentages.get(color).copied().unwrap_or(0.0);
                basic_counts.insert(*color, need + extras * percentage);
            }
        }

        // Round using largest remainder method for accuracy
        let target_total = deck.basic_land_slots();
        let mut fractional_parts: Vec<(Color, f64)> = Vec::new();
        let mut rounded_total: u32 = 0;

        for color in &deck.colors {
            let count = basic_counts.get(color).copied().unwrap_or(0.0);
            let floored = count.floor() as u32;
            let fractional = count - count.floor();
            mana_base.basics.insert(*color, floored);
            rounded_total += floored;
            fractional_parts.push((*color, fractional));
        }

        // Distribute remaining slots to colors with highest fractional parts
        fractional_parts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut slots_to_add = target_total.saturating_sub(rounded_total);
        for (color, _) in fractional_parts {
            if slots_to_add == 0 {
                break;
            }
            if let Some(count) = mana_base.basics.get_mut(&color) {
                *count += 1;
                slots_to_add -= 1;
            }
        }

        // Remove zero-count entries
        mana_base.basics.retain(|_, &mut v| v > 0);

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
}
