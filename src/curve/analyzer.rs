use crate::input::DeckList;
use std::collections::HashMap;

use super::types::{CmcBucket, CurveAnalysis, CurveStats};

pub struct CurveAnalyzer;

impl CurveAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a deck's mana curve
    pub fn analyze(&self, deck_list: &DeckList) -> CurveAnalysis {
        let mut analysis = CurveAnalysis::new();
        analysis.deck_name = deck_list.name.clone();
        analysis.deck_format = deck_list.format.clone();
        analysis.total_cards = deck_list.total_cards();
        analysis.unique_cards = deck_list.unique_cards() as u32;

        // Collect CMC data from hydrated cards (mainboard only, excluding lands)
        let mut cmc_map: HashMap<u32, CmcBucket> = HashMap::new();
        let mut all_cmcs: Vec<f64> = Vec::new();

        for entry in deck_list.mainboard() {
            if let Some(card) = &entry.card {
                // Skip lands
                if Self::is_land(&card.type_line) {
                    continue;
                }

                let cmc = card.cmc.round() as u32;
                let is_creature = Self::is_creature(&card.type_line);

                // Track for statistics (accounting for quantity)
                for _ in 0..entry.quantity {
                    all_cmcs.push(card.cmc);
                }

                let bucket = cmc_map.entry(cmc).or_insert_with(|| CmcBucket::new(cmc));

                bucket.total_count += entry.quantity;
                bucket.card_names.push(card.name.clone());

                if is_creature {
                    bucket.creature_count += entry.quantity;
                    bucket.creature_names.push(card.name.clone());
                } else {
                    bucket.non_creature_count += entry.quantity;
                    bucket.non_creature_names.push(card.name.clone());
                }
            }
        }

        // Convert to sorted vec
        let mut buckets: Vec<CmcBucket> = cmc_map.into_values().collect();
        buckets.sort_by_key(|b| b.cmc);

        // Calculate statistics
        let stats = Self::calculate_stats(&buckets, &all_cmcs);

        // Calculate max values for histogram scaling
        let max_cmc = buckets.iter().map(|b| b.cmc).max().unwrap_or(0);
        let max_count = buckets.iter().map(|b| b.total_count).max().unwrap_or(0);

        analysis.buckets = buckets;
        analysis.stats = stats;
        analysis.max_cmc = max_cmc;
        analysis.max_count = max_count;

        analysis
    }

    fn is_land(type_line: &str) -> bool {
        type_line.to_lowercase().contains("land")
    }

    fn is_creature(type_line: &str) -> bool {
        type_line.to_lowercase().contains("creature")
    }

    fn calculate_stats(buckets: &[CmcBucket], all_cmcs: &[f64]) -> CurveStats {
        let total_nonland = all_cmcs.len() as u32;
        if total_nonland == 0 {
            return CurveStats::default();
        }

        // Average
        let average_cmc = all_cmcs.iter().sum::<f64>() / all_cmcs.len() as f64;

        // Median
        let mut sorted_cmcs = all_cmcs.to_vec();
        sorted_cmcs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_cmc = if sorted_cmcs.len().is_multiple_of(2) {
            let mid = sorted_cmcs.len() / 2;
            (sorted_cmcs[mid - 1] + sorted_cmcs[mid]) / 2.0
        } else {
            sorted_cmcs[sorted_cmcs.len() / 2]
        };

        // Mode (most common CMC)
        let mode_cmc = buckets
            .iter()
            .max_by_key(|b| b.total_count)
            .map(|b| b.cmc)
            .unwrap_or(0);

        // Totals
        let total_creatures: u32 = buckets.iter().map(|b| b.creature_count).sum();
        let total_non_creatures: u32 = buckets.iter().map(|b| b.non_creature_count).sum();

        // Distribution percentages
        let mut cmc_distribution = HashMap::new();
        let mut creature_distribution = HashMap::new();
        let mut non_creature_distribution = HashMap::new();

        for bucket in buckets {
            cmc_distribution.insert(bucket.cmc, bucket.total_count as f64 / total_nonland as f64);
            if total_creatures > 0 {
                creature_distribution.insert(
                    bucket.cmc,
                    bucket.creature_count as f64 / total_creatures as f64,
                );
            }
            if total_non_creatures > 0 {
                non_creature_distribution.insert(
                    bucket.cmc,
                    bucket.non_creature_count as f64 / total_non_creatures as f64,
                );
            }
        }

        CurveStats {
            average_cmc,
            median_cmc,
            mode_cmc,
            total_nonland_cards: total_nonland,
            total_creatures,
            total_non_creatures,
            cmc_distribution,
            creature_distribution,
            non_creature_distribution,
        }
    }
}

impl Default for CurveAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
