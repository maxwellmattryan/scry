use crate::deck::{Algorithm, Deck, ManaBase};
use super::{SimpleCalculator, CmcWeightedCalculator};

pub trait ManaCalculator {
    fn calculate(&self, deck: &Deck) -> ManaBase;
    fn name(&self) -> &'static str;
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
