use std::fs;
use std::io::Write;

use crate::calculator::get_intensity_recommendations;
use crate::deck::{guild_name, Color, Deck, ManaBase};

pub struct MarkdownExporter;

impl MarkdownExporter {
    pub fn export(
        deck: &Deck,
        mana_base: &ManaBase,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = Self::generate(deck, mana_base);
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn generate(deck: &Deck, mana_base: &ManaBase) -> String {
        let mut output = String::new();

        // Header
        output.push_str("# MTG Deck Mana Base Analysis\n\n");
        output.push_str(&format!(
            "Date: {}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M")
        ));

        // Deck Configuration
        output.push_str("## Deck Configuration\n\n");
        output.push_str(&format!("- **Format**: {}\n", deck.format.name()));
        output.push_str(&format!("- **Total Cards**: {}\n", deck.total_cards));
        output.push_str(&format!("- **Target Lands**: {}\n", deck.target_lands));

        let color_names: Vec<&str> = deck.colors.iter().map(|c| c.name()).collect();
        output.push_str(&format!("- **Colors**: {}\n", color_names.join(", ")));
        output.push('\n');

        // Mana Symbol Distribution
        output.push_str("## Mana Symbol Distribution\n\n");
        let total_symbols = deck.total_mana_symbols();

        for color in &deck.colors {
            let count = deck.mana_symbols.get(color).copied().unwrap_or(0);
            let percentage = if total_symbols > 0 {
                (count as f64 / total_symbols as f64) * 100.0
            } else {
                0.0
            };

            output.push_str(&format!(
                "- **{} ({})**: {} symbols ({:.1}%)\n",
                color.name(),
                color.symbol(),
                count,
                percentage
            ));
        }
        output.push('\n');

        // Recommended Mana Base
        output.push_str("## Recommended Mana Base\n\n");

        // Basic Lands
        let total_basics: u32 = mana_base.basics.values().sum();
        output.push_str(&format!("### Basic Lands ({total_basics} total)\n\n"));

        let mut basics: Vec<_> = mana_base.basics.iter().collect();
        basics.sort_by_key(|(c, _)| match c {
            Color::White => 0,
            Color::Blue => 1,
            Color::Black => 2,
            Color::Red => 3,
            Color::Green => 4,
            Color::Colorless => 5,
        });

        for (color, count) in &basics {
            output.push_str(&format!("- {}: {}\n", color.basic_land(), count));
        }
        output.push('\n');

        // Dual Lands
        if !mana_base.duals.is_empty() {
            let total_duals: u32 = mana_base.duals.iter().map(|d| d.count).sum();
            output.push_str(&format!("### Dual Lands ({total_duals} total)\n\n"));

            for dual in &mana_base.duals {
                let name = if dual.colors.len() == 2 {
                    guild_name(&dual.colors)
                        .map(|g| g.to_string())
                        .unwrap_or_else(|| dual.name.clone())
                } else {
                    dual.name.clone()
                };

                let colors_str: Vec<_> = dual.colors.iter().map(|c| c.symbol()).collect();
                output.push_str(&format!(
                    "- {} ({}): {}\n",
                    name,
                    colors_str.join("/"),
                    dual.count
                ));
            }
            output.push('\n');
        }

        // Pip Intensity Analysis
        let recommendations = get_intensity_recommendations(deck);
        if !recommendations.is_empty() {
            output.push_str("## Pip Intensity Analysis\n\n");
            for rec in recommendations {
                output.push_str(&format!("- ⚠️ {rec}\n"));
            }
            output.push('\n');
        }

        // Color Source Summary
        output.push_str("## Color Source Summary\n\n");
        output.push_str("| Color | Basic | Dual | Total | % of Lands |\n");
        output.push_str("|-------|-------|------|-------|------------|\n");

        for color in &deck.colors {
            let basic_count = mana_base.basics.get(color).copied().unwrap_or(0);

            let dual_count: u32 = mana_base
                .duals
                .iter()
                .filter(|d| d.colors.contains(color))
                .map(|d| d.count)
                .sum();

            let total = basic_count + dual_count;
            let percentage = (total as f64 / deck.target_lands as f64) * 100.0;

            output.push_str(&format!(
                "| {} | {} | {} | {} | {:.1}% |\n",
                color.name(),
                basic_count,
                dual_count,
                total,
                percentage
            ));
        }
        output.push('\n');

        output
    }
}
