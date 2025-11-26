use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entry for a single CMC bucket in the mana curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmcBucket {
    pub cmc: u32,
    pub total_count: u32,
    pub creature_count: u32,
    pub non_creature_count: u32,
    pub card_names: Vec<String>,
    pub creature_names: Vec<String>,
    pub non_creature_names: Vec<String>,
}

impl CmcBucket {
    pub fn new(cmc: u32) -> Self {
        Self {
            cmc,
            total_count: 0,
            creature_count: 0,
            non_creature_count: 0,
            card_names: Vec::new(),
            creature_names: Vec::new(),
            non_creature_names: Vec::new(),
        }
    }
}

/// Statistics about the mana curve
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurveStats {
    pub average_cmc: f64,
    pub median_cmc: f64,
    pub mode_cmc: u32,
    pub total_nonland_cards: u32,
    pub total_creatures: u32,
    pub total_non_creatures: u32,
    pub cmc_distribution: HashMap<u32, f64>,
    pub creature_distribution: HashMap<u32, f64>,
    pub non_creature_distribution: HashMap<u32, f64>,
}

/// Complete mana curve analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveAnalysis {
    pub deck_name: Option<String>,
    pub deck_format: Option<String>,
    pub total_cards: u32,
    pub unique_cards: u32,
    pub buckets: Vec<CmcBucket>,
    pub stats: CurveStats,
    pub max_cmc: u32,
    pub max_count: u32,
}

impl CurveAnalysis {
    pub fn new() -> Self {
        Self {
            deck_name: None,
            deck_format: None,
            total_cards: 0,
            unique_cards: 0,
            buckets: Vec::new(),
            stats: CurveStats::default(),
            max_cmc: 0,
            max_count: 0,
        }
    }
}

impl Default for CurveAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
