use std::collections::HashMap;
use super::types::{Color, Deck, DualLand, Format};

pub struct DeckBuilder {
    deck: Deck,
}

impl DeckBuilder {
    pub fn new(format: Format) -> Self {
        Self {
            deck: Deck::new(format),
        }
    }

    pub fn total_cards(mut self, cards: u32) -> Self {
        self.deck.total_cards = cards;
        self
    }

    pub fn target_lands(mut self, lands: u32) -> Self {
        self.deck.target_lands = lands;
        self
    }

    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.deck.colors = colors;
        self
    }

    pub fn add_color(mut self, color: Color) -> Self {
        if !self.deck.colors.contains(&color) {
            self.deck.colors.push(color);
        }
        self
    }

    pub fn mana_symbols(mut self, symbols: HashMap<Color, u32>) -> Self {
        self.deck.mana_symbols = symbols;
        self
    }

    pub fn set_mana_symbol_count(mut self, color: Color, count: u32) -> Self {
        self.deck.mana_symbols.insert(color, count);
        self
    }

    pub fn dual_lands(mut self, duals: Vec<DualLand>) -> Self {
        self.deck.dual_lands = duals;
        self
    }

    pub fn add_dual_land(mut self, name: String, colors: Vec<Color>, count: u32) -> Self {
        self.deck.dual_lands.push(DualLand::new(name, colors, count));
        self
    }

    pub fn pip_intensity(mut self, intensity: HashMap<Color, u32>) -> Self {
        self.deck.pip_intensity = intensity;
        self
    }

    pub fn set_pip_intensity(mut self, color: Color, count: u32) -> Self {
        self.deck.pip_intensity.insert(color, count);
        self
    }

    pub fn build(self) -> Deck {
        self.deck
    }
}
