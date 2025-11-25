use super::{CmcWeightedCalculator, SimpleCalculator};
use crate::deck::{Algorithm, Deck, ManaBase};

pub trait ManaCalculator {
    fn calculate(&self, deck: &Deck) -> ManaBase;
}

pub fn get_calculator(algorithm: Algorithm) -> Box<dyn ManaCalculator> {
    match algorithm {
        Algorithm::Simple => Box::new(SimpleCalculator),
        Algorithm::CmcWeighted => Box::new(CmcWeightedCalculator),
        Algorithm::Hypergeometric => {
            // Hypergeometric is complex, fall back to simple for now
            Box::new(SimpleCalculator)
        }
    }
}
