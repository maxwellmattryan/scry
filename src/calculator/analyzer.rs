use crate::deck::Deck;

pub struct PipAnalysis {
    pub intensity: u32,
    pub warning: Option<String>,
}

pub fn analyze_pip_intensity(deck: &Deck) -> Vec<PipAnalysis> {
    let mut analyses = Vec::new();

    for color in &deck.colors {
        let intensity = deck.pip_intensity.get(color).copied().unwrap_or(0);

        let warning = if intensity >= 5 {
            Some(format!(
                "{} has very high pip density ({} cards with double+ pips). Strongly consider additional {} sources, fetch lands, or mana rocks.",
                color.name(),
                intensity,
                color.name().to_lowercase()
            ))
        } else if intensity >= 3 {
            Some(format!(
                "{} has high pip density ({} cards with {{{}}}{{{}}} or more). Consider additional {} sources or mana rocks.",
                color.name(),
                intensity,
                color.symbol(),
                color.symbol(),
                color.name().to_lowercase()
            ))
        } else {
            None
        };

        analyses.push(PipAnalysis { intensity, warning });
    }

    // Sort by intensity descending
    analyses.sort_by(|a, b| b.intensity.cmp(&a.intensity));
    analyses
}

pub fn get_intensity_recommendations(deck: &Deck) -> Vec<String> {
    analyze_pip_intensity(deck)
        .into_iter()
        .filter_map(|a| a.warning)
        .collect()
}
