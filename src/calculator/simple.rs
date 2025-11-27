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

        mana_base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{DualLand, Format};

    fn make_deck(
        colors: Vec<Color>,
        mana_symbols: HashMap<Color, u32>,
        dual_lands: Vec<DualLand>,
        target_lands: u32,
    ) -> Deck {
        let mut deck = Deck::new(Format::Standard);
        deck.colors = colors;
        deck.mana_symbols = mana_symbols;
        deck.dual_lands = dual_lands;
        deck.target_lands = target_lands;
        deck
    }

    #[test]
    fn test_mono_color_no_duals() {
        // Case 1: 24 lands, 100% Blue, no duals -> 24 Islands
        let mut symbols = HashMap::new();
        symbols.insert(Color::Blue, 20);

        let deck = make_deck(vec![Color::Blue], symbols, vec![], 24);

        let calc = SimpleCalculator;
        let result = calc.calculate(&deck);

        assert_eq!(result.basics.get(&Color::Blue), Some(&24));
        let total: u32 = result.basics.values().sum();
        assert_eq!(total, 24);
    }

    #[test]
    fn test_two_color_even_split_with_duals() {
        // Case 2: 24 lands, 50% U / 50% B, 4 U/B duals
        // Expected: 8 Islands, 8 Swamps
        let mut symbols = HashMap::new();
        symbols.insert(Color::Blue, 10);
        symbols.insert(Color::Black, 10);

        let dual = DualLand::new(
            "Dimir lands".to_string(),
            vec![Color::Blue, Color::Black],
            4,
        );

        let deck = make_deck(vec![Color::Blue, Color::Black], symbols, vec![dual], 24);

        let calc = SimpleCalculator;
        let result = calc.calculate(&deck);

        // With 4 duals, we have 20 basic slots
        // Each color needs 12 sources, duals provide 4 each
        // Remaining need: 8 each, total = 16, but we have 20 slots
        // Extras (4) distributed by percentage (50/50) = +2 each
        // Final: 10 Islands, 10 Swamps
        let total: u32 = result.basics.values().sum();
        assert_eq!(total, 20); // 24 - 4 duals = 20 basics

        // Both colors should be roughly equal
        let blue = result.basics.get(&Color::Blue).copied().unwrap_or(0);
        let black = result.basics.get(&Color::Black).copied().unwrap_or(0);
        assert_eq!(blue, black);
    }

    #[test]
    fn test_grixis_heavy_duals_maintains_balance() {
        // Case 3: Grixis with heavy U/B duals (the bug case)
        // 24 lands, 40% U / 35% B / 25% R, 8 U/B duals
        // Old algorithm would give Red 12 Mountains (over-allocated)
        // New algorithm should maintain color balance
        let mut symbols = HashMap::new();
        symbols.insert(Color::Blue, 8); // 40%
        symbols.insert(Color::Black, 7); // 35%
        symbols.insert(Color::Red, 5); // 25%

        let dual = DualLand::new(
            "Dimir lands".to_string(),
            vec![Color::Blue, Color::Black],
            8,
        );

        let deck = make_deck(
            vec![Color::Blue, Color::Black, Color::Red],
            symbols,
            vec![dual],
            24,
        );

        let calc = SimpleCalculator;
        let result = calc.calculate(&deck);

        let total: u32 = result.basics.values().sum();
        assert_eq!(total, 16); // 24 - 8 duals = 16 basics

        let blue = result.basics.get(&Color::Blue).copied().unwrap_or(0);
        let black = result.basics.get(&Color::Black).copied().unwrap_or(0);
        let red = result.basics.get(&Color::Red).copied().unwrap_or(0);

        // Red should NOT dominate just because duals don't help it
        // With the bug, Red would get 12. With fix, it should be ~8
        assert!(red <= 10, "Red should not be over-allocated: got {}", red);

        // Blue should still get some basics despite heavy dual coverage
        assert!(blue >= 3, "Blue needs some basics: got {}", blue);

        // Verify rough proportions are maintained
        // Blue should be > Black (40% > 35%)
        assert!(
            blue >= black,
            "Blue ({}) should be >= Black ({})",
            blue,
            black
        );
    }

    #[test]
    fn test_duals_exceed_target_for_one_color() {
        // Case 5: 24 lands, 60% U / 40% B, 10 U/B duals
        // Duals give: U=10, B=10 (B is over-satisfied since it only needs 9.6)
        let mut symbols = HashMap::new();
        symbols.insert(Color::Blue, 12); // 60%
        symbols.insert(Color::Black, 8); // 40%

        let dual = DualLand::new(
            "Dimir lands".to_string(),
            vec![Color::Blue, Color::Black],
            10,
        );

        let deck = make_deck(vec![Color::Blue, Color::Black], symbols, vec![dual], 24);

        let calc = SimpleCalculator;
        let result = calc.calculate(&deck);

        let total: u32 = result.basics.values().sum();
        assert_eq!(total, 14); // 24 - 10 duals = 14 basics

        let blue = result.basics.get(&Color::Blue).copied().unwrap_or(0);
        let black = result.basics.get(&Color::Black).copied().unwrap_or(0);

        // Blue should get more since it has higher percentage
        assert!(
            blue > black,
            "Blue ({}) should be > Black ({})",
            blue,
            black
        );

        // Both should have some basics (extras distributed by percentage)
        assert!(blue >= 8, "Blue should have adequate basics: got {}", blue);
        assert!(black >= 4, "Black should have some basics: got {}", black);
    }

    #[test]
    fn test_overconstrained_five_color() {
        // Case 4: 24 lands, 5-color even (20% each), 2 duals covering W/U
        // Tests scaling DOWN when not enough slots
        let mut symbols = HashMap::new();
        symbols.insert(Color::White, 4);
        symbols.insert(Color::Blue, 4);
        symbols.insert(Color::Black, 4);
        symbols.insert(Color::Red, 4);
        symbols.insert(Color::Green, 4);

        let dual = DualLand::new(
            "Azorius lands".to_string(),
            vec![Color::White, Color::Blue],
            2,
        );

        let deck = make_deck(
            vec![
                Color::White,
                Color::Blue,
                Color::Black,
                Color::Red,
                Color::Green,
            ],
            symbols,
            vec![dual],
            24,
        );

        let calc = SimpleCalculator;
        let result = calc.calculate(&deck);

        let total: u32 = result.basics.values().sum();
        assert_eq!(total, 22); // 24 - 2 duals = 22 basics

        // Each color should have roughly similar counts
        // W and U get 2 from duals, so need less basics
        let white = result.basics.get(&Color::White).copied().unwrap_or(0);
        let blue = result.basics.get(&Color::Blue).copied().unwrap_or(0);
        let black = result.basics.get(&Color::Black).copied().unwrap_or(0);

        // Colors not covered by duals should have more basics
        assert!(
            black > white || black > blue,
            "Non-dual colors should have more basics"
        );
    }
}
