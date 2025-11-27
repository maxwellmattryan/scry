use crate::deck::{Color, ManaBase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Breakdown of colored mana pips in a deck
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorPipBreakdown {
    pub white: f64,
    pub blue: f64,
    pub black: f64,
    pub red: f64,
    pub green: f64,
    pub colorless: f64,
}

impl ColorPipBreakdown {
    pub fn total(&self) -> f64 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    pub fn add(&mut self, other: &ColorPipBreakdown) {
        self.white += other.white;
        self.blue += other.blue;
        self.black += other.black;
        self.red += other.red;
        self.green += other.green;
        self.colorless += other.colorless;
    }

    /// Convert pip breakdown to mana symbol counts for calculator input
    pub fn to_mana_symbols(&self) -> HashMap<Color, u32> {
        let mut symbols = HashMap::new();

        if self.white > 0.0 {
            symbols.insert(Color::White, self.white.round() as u32);
        }
        if self.blue > 0.0 {
            symbols.insert(Color::Blue, self.blue.round() as u32);
        }
        if self.black > 0.0 {
            symbols.insert(Color::Black, self.black.round() as u32);
        }
        if self.red > 0.0 {
            symbols.insert(Color::Red, self.red.round() as u32);
        }
        if self.green > 0.0 {
            symbols.insert(Color::Green, self.green.round() as u32);
        }
        if self.colorless > 0.0 {
            symbols.insert(Color::Colorless, self.colorless.round() as u32);
        }

        symbols
    }

    /// Get colors present in the pip breakdown (non-zero, excluding colorless)
    pub fn colors(&self) -> Vec<Color> {
        let mut colors = Vec::new();
        if self.white > 0.0 {
            colors.push(Color::White);
        }
        if self.blue > 0.0 {
            colors.push(Color::Blue);
        }
        if self.black > 0.0 {
            colors.push(Color::Black);
        }
        if self.red > 0.0 {
            colors.push(Color::Red);
        }
        if self.green > 0.0 {
            colors.push(Color::Green);
        }
        colors
    }
}

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

/// How the target land count was determined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LandCountSource {
    /// User explicitly provided via --lands flag
    UserProvided,
    /// Detected by counting lands in the deck
    DetectedFromDeck(u32),
    /// Inferred from deck format defaults
    FormatDefault(String),
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
    pub pip_breakdown: ColorPipBreakdown,
    /// Optional mana base recommendation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mana_base: Option<ManaBase>,
    /// Target land count used for calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_lands: Option<u32>,
    /// How the land count was determined
    #[serde(skip_serializing_if = "Option::is_none")]
    pub land_source: Option<LandCountSource>,
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
            pip_breakdown: ColorPipBreakdown::default(),
            mana_base: None,
            target_lands: None,
            land_source: None,
        }
    }
}

impl Default for CurveAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
