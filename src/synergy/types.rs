#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Counter types for counter-based themes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CounterType {
    PlusOne,
    MinusOne,
    Loyalty,
    Generic(String),
}

/// A detected synergy theme in the deck
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Theme {
    // Mechanical themes
    Tokens,
    Counters(CounterType),
    Graveyard,
    Sacrifice,
    Blink,
    Ramp,
    Draw,
    Removal,
    Lifegain,
    Discard,
    Mill,
    Equipment,
    Auras,
    Artifacts,
    Enchantments,
    Lands,

    // Tribal themes
    Tribal(String),

    // Strategy themes
    Aggro,
    Control,
    Combo,
    Midrange,
    Stax,
    Voltron,
    Spellslinger,
    Aristocrats,
    Reanimator,
    Storm,

    // Custom/LLM-detected
    Custom(String),
}

impl Theme {
    /// Get a display name for the theme
    pub fn display_name(&self) -> String {
        match self {
            Theme::Tokens => "Tokens".to_string(),
            Theme::Counters(ct) => match ct {
                CounterType::PlusOne => "+1/+1 Counters".to_string(),
                CounterType::MinusOne => "-1/-1 Counters".to_string(),
                CounterType::Loyalty => "Loyalty Counters".to_string(),
                CounterType::Generic(s) => format!("{s} Counters"),
            },
            Theme::Graveyard => "Graveyard".to_string(),
            Theme::Sacrifice => "Sacrifice".to_string(),
            Theme::Blink => "Blink/Flicker".to_string(),
            Theme::Ramp => "Ramp".to_string(),
            Theme::Draw => "Card Draw".to_string(),
            Theme::Removal => "Removal".to_string(),
            Theme::Lifegain => "Lifegain".to_string(),
            Theme::Discard => "Discard".to_string(),
            Theme::Mill => "Mill".to_string(),
            Theme::Equipment => "Equipment".to_string(),
            Theme::Auras => "Auras".to_string(),
            Theme::Artifacts => "Artifacts Matter".to_string(),
            Theme::Enchantments => "Enchantments Matter".to_string(),
            Theme::Lands => "Lands Matter".to_string(),
            Theme::Tribal(tribe) => format!("{tribe} Tribal"),
            Theme::Aggro => "Aggro".to_string(),
            Theme::Control => "Control".to_string(),
            Theme::Combo => "Combo".to_string(),
            Theme::Midrange => "Midrange".to_string(),
            Theme::Stax => "Stax".to_string(),
            Theme::Voltron => "Voltron".to_string(),
            Theme::Spellslinger => "Spellslinger".to_string(),
            Theme::Aristocrats => "Aristocrats".to_string(),
            Theme::Reanimator => "Reanimator".to_string(),
            Theme::Storm => "Storm".to_string(),
            Theme::Custom(s) => s.clone(),
        }
    }
}

/// MTG keywords that can be detected from oracle text
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    // Evergreen keywords
    Flying,
    Trample,
    Haste,
    Vigilance,
    Deathtouch,
    Lifelink,
    FirstStrike,
    DoubleStrike,
    Menace,
    Reach,
    Flash,
    Hexproof,
    Indestructible,
    Defender,
    Ward,

    // Set/ability keywords
    Flashback,
    Unearth,
    Escape,
    Delve,
    Convoke,
    Cascade,
    Storm,
    Proliferate,
    Landfall,
    Constellation,
    Devotion,
    Annihilator,
    Infect,
    Wither,
    Affinity,
    Madness,
    Overload,
    Crew,
    Equip,

    // Generic catch-all
    Other(String),
}

impl Keyword {
    /// Get a display name for the keyword
    pub fn display_name(&self) -> String {
        match self {
            Keyword::Flying => "Flying".to_string(),
            Keyword::Trample => "Trample".to_string(),
            Keyword::Haste => "Haste".to_string(),
            Keyword::Vigilance => "Vigilance".to_string(),
            Keyword::Deathtouch => "Deathtouch".to_string(),
            Keyword::Lifelink => "Lifelink".to_string(),
            Keyword::FirstStrike => "First Strike".to_string(),
            Keyword::DoubleStrike => "Double Strike".to_string(),
            Keyword::Menace => "Menace".to_string(),
            Keyword::Reach => "Reach".to_string(),
            Keyword::Flash => "Flash".to_string(),
            Keyword::Hexproof => "Hexproof".to_string(),
            Keyword::Indestructible => "Indestructible".to_string(),
            Keyword::Defender => "Defender".to_string(),
            Keyword::Ward => "Ward".to_string(),
            Keyword::Flashback => "Flashback".to_string(),
            Keyword::Unearth => "Unearth".to_string(),
            Keyword::Escape => "Escape".to_string(),
            Keyword::Delve => "Delve".to_string(),
            Keyword::Convoke => "Convoke".to_string(),
            Keyword::Cascade => "Cascade".to_string(),
            Keyword::Storm => "Storm".to_string(),
            Keyword::Proliferate => "Proliferate".to_string(),
            Keyword::Landfall => "Landfall".to_string(),
            Keyword::Constellation => "Constellation".to_string(),
            Keyword::Devotion => "Devotion".to_string(),
            Keyword::Annihilator => "Annihilator".to_string(),
            Keyword::Infect => "Infect".to_string(),
            Keyword::Wither => "Wither".to_string(),
            Keyword::Affinity => "Affinity".to_string(),
            Keyword::Madness => "Madness".to_string(),
            Keyword::Overload => "Overload".to_string(),
            Keyword::Crew => "Crew".to_string(),
            Keyword::Equip => "Equip".to_string(),
            Keyword::Other(s) => s.clone(),
        }
    }
}

/// Card role within a synergy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynergyRole {
    Enabler,
    Payoff,
    Support,
}

/// Represents a card's role in the deck's synergy web
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSynergyProfile {
    pub card_name: String,
    pub themes: Vec<Theme>,
    pub keywords: Vec<Keyword>,
    pub role: Option<SynergyRole>,
    pub synergizes_with: Vec<String>,
    pub synergy_score: f64,
}

impl CardSynergyProfile {
    pub fn new(card_name: String) -> Self {
        Self {
            card_name,
            themes: Vec::new(),
            keywords: Vec::new(),
            role: None,
            synergizes_with: Vec::new(),
            synergy_score: 0.0,
        }
    }
}

/// Card relationship type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynergyRelation {
    Enables,
    PayoffFor,
    Supports,
    Combos,
}

/// A synergy connection between two cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyEdge {
    pub card_a: String,
    pub card_b: String,
    pub relation: SynergyRelation,
    pub themes: Vec<Theme>,
    pub strength: f64,
    pub reason: String,
}

/// Analysis results for a single theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeAnalysis {
    pub theme: Theme,
    pub card_count: u32,
    pub percentage: f64,
    pub enablers: Vec<String>,
    pub payoffs: Vec<String>,
    pub support: Vec<String>,
}

impl ThemeAnalysis {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            card_count: 0,
            percentage: 0.0,
            enablers: Vec::new(),
            payoffs: Vec::new(),
            support: Vec::new(),
        }
    }

    /// Get all cards in this theme
    pub fn all_cards(&self) -> Vec<&String> {
        let mut cards: Vec<&String> = self
            .enablers
            .iter()
            .chain(self.payoffs.iter())
            .chain(self.support.iter())
            .collect();
        cards.dedup();
        cards
    }
}

/// Info about a card with no synergies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanCard {
    pub name: String,
    pub reason: String,
}

/// Statistics about the deck's synergies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyStats {
    /// Ratio of synergy edges to possible edges
    pub synergy_density: f64,
    /// Percentage of cards in at least one theme
    pub theme_coverage: f64,
    /// Cards with no synergy connections (with reasons)
    pub orphan_cards: Vec<OrphanCard>,
    /// Cards with most connections
    pub hub_cards: Vec<String>,
    /// How focused the themes are (higher = more focused)
    pub theme_coherence: f64,
    /// Distribution of keywords
    pub keyword_distribution: HashMap<String, u32>,
}

impl Default for SynergyStats {
    fn default() -> Self {
        Self {
            synergy_density: 0.0,
            theme_coverage: 0.0,
            orphan_cards: Vec::new(),
            hub_cards: Vec::new(),
            theme_coherence: 0.0,
            keyword_distribution: HashMap::new(),
        }
    }
}

/// The complete synergy analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyMatrix {
    pub deck_name: Option<String>,
    pub deck_format: Option<String>,
    pub total_cards: u32,
    pub unique_cards: u32,

    /// Theme analysis results
    pub detected_themes: Vec<ThemeAnalysis>,
    /// The dominant theme
    pub primary_theme: Option<Theme>,

    /// Per-card synergy profiles
    pub card_profiles: HashMap<String, CardSynergyProfile>,

    /// Synergy edges (card-to-card relationships)
    pub edges: Vec<SynergyEdge>,

    /// Overall statistics
    pub stats: SynergyStats,

    /// Observations and insights
    pub observations: Vec<String>,
}

impl SynergyMatrix {
    pub fn new() -> Self {
        Self {
            deck_name: None,
            deck_format: None,
            total_cards: 0,
            unique_cards: 0,
            detected_themes: Vec::new(),
            primary_theme: None,
            card_profiles: HashMap::new(),
            edges: Vec::new(),
            stats: SynergyStats::default(),
            observations: Vec::new(),
        }
    }

    /// Sort themes by card count (descending)
    pub fn sort_themes(&mut self) {
        self.detected_themes
            .sort_by(|a, b| b.card_count.cmp(&a.card_count));
    }

    /// Get top N themes
    pub fn top_themes(&self, n: usize) -> Vec<&ThemeAnalysis> {
        self.detected_themes.iter().take(n).collect()
    }
}

impl Default for SynergyMatrix {
    fn default() -> Self {
        Self::new()
    }
}
