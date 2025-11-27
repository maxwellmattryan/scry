use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

impl Color {
    pub fn symbol(&self) -> &'static str {
        match self {
            Color::White => "W",
            Color::Blue => "U",
            Color::Black => "B",
            Color::Red => "R",
            Color::Green => "G",
            Color::Colorless => "C",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Color::White => "White",
            Color::Blue => "Blue",
            Color::Black => "Black",
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Colorless => "Colorless",
        }
    }

    pub fn basic_land(&self) -> &'static str {
        match self {
            Color::White => "Plains",
            Color::Blue => "Island",
            Color::Black => "Swamp",
            Color::Red => "Mountain",
            Color::Green => "Forest",
            Color::Colorless => "Wastes",
        }
    }

    pub fn from_symbol(s: &str) -> Option<Color> {
        match s.to_uppercase().as_str() {
            "W" => Some(Color::White),
            "U" => Some(Color::Blue),
            "B" => Some(Color::Black),
            "R" => Some(Color::Red),
            "G" => Some(Color::Green),
            "C" => Some(Color::Colorless),
            _ => None,
        }
    }

    pub fn all_colors() -> Vec<Color> {
        vec![
            Color::White,
            Color::Blue,
            Color::Black,
            Color::Red,
            Color::Green,
        ]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualLand {
    pub name: String,
    pub colors: Vec<Color>,
    pub count: u32,
}

impl DualLand {
    pub fn new(name: String, colors: Vec<Color>, count: u32) -> Self {
        Self {
            name,
            colors,
            count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    Commander,
    Standard,
    Modern,
    Limited,
    Custom,
}

impl Format {
    pub fn default_cards(&self) -> u32 {
        match self {
            Format::Commander => 100,
            Format::Standard => 60,
            Format::Modern => 60,
            Format::Limited => 40,
            Format::Custom => 60,
        }
    }

    pub fn default_lands(&self) -> u32 {
        match self {
            Format::Commander => 38,
            Format::Standard => 24,
            Format::Modern => 24,
            Format::Limited => 17,
            Format::Custom => 24,
        }
    }

    pub fn recommended_land_range(&self) -> (u32, u32) {
        match self {
            Format::Commander => (36, 40),
            Format::Standard => (20, 26),
            Format::Modern => (20, 26),
            Format::Limited => (16, 18),
            Format::Custom => (20, 30),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Format::Commander => "Commander",
            Format::Standard => "Standard",
            Format::Modern => "Modern",
            Format::Limited => "Limited",
            Format::Custom => "Custom",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} cards)", self.name(), self.default_cards())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub format: Format,
    pub total_cards: u32,
    pub target_lands: u32,
    pub colors: Vec<Color>,
    pub mana_symbols: HashMap<Color, u32>,
    pub dual_lands: Vec<DualLand>,
    pub pip_intensity: HashMap<Color, u32>,
}

impl Deck {
    pub fn new(format: Format) -> Self {
        Self {
            format,
            total_cards: format.default_cards(),
            target_lands: format.default_lands(),
            colors: Vec::new(),
            mana_symbols: HashMap::new(),
            dual_lands: Vec::new(),
            pip_intensity: HashMap::new(),
        }
    }

    pub fn total_mana_symbols(&self) -> u32 {
        self.mana_symbols.values().sum()
    }

    pub fn dual_land_count(&self) -> u32 {
        self.dual_lands.iter().map(|d| d.count).sum()
    }

    pub fn basic_land_slots(&self) -> u32 {
        self.target_lands.saturating_sub(self.dual_land_count())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaBase {
    pub basics: HashMap<Color, u32>,
    pub duals: Vec<DualLand>,
    pub recommendations: Vec<String>,
    pub color_percentages: HashMap<Color, f64>,
}

impl ManaBase {
    pub fn new() -> Self {
        Self {
            basics: HashMap::new(),
            duals: Vec::new(),
            recommendations: Vec::new(),
            color_percentages: HashMap::new(),
        }
    }
}

impl Default for ManaBase {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    Simple,
    CmcWeighted,
    Hypergeometric,
}

impl Algorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Simple => "simple",
            Algorithm::CmcWeighted => "cmc",
            Algorithm::Hypergeometric => "hypergeo",
        }
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// Guild names for dual color combinations
pub fn guild_name(colors: &[Color]) -> Option<&'static str> {
    if colors.len() != 2 {
        return None;
    }

    let mut sorted = colors.to_vec();
    sorted.sort_by_key(|c| match c {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
        Color::Colorless => 5,
    });

    match (sorted.first(), sorted.get(1)) {
        (Some(Color::White), Some(Color::Blue)) => Some("Azorius"),
        (Some(Color::White), Some(Color::Black)) => Some("Orzhov"),
        (Some(Color::White), Some(Color::Red)) => Some("Boros"),
        (Some(Color::White), Some(Color::Green)) => Some("Selesnya"),
        (Some(Color::Blue), Some(Color::Black)) => Some("Dimir"),
        (Some(Color::Blue), Some(Color::Red)) => Some("Izzet"),
        (Some(Color::Blue), Some(Color::Green)) => Some("Simic"),
        (Some(Color::Black), Some(Color::Red)) => Some("Rakdos"),
        (Some(Color::Black), Some(Color::Green)) => Some("Golgari"),
        (Some(Color::Red), Some(Color::Green)) => Some("Gruul"),
        _ => None,
    }
}
