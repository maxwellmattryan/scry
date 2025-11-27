use crate::curve::{CurveAnalysis, LandCountSource};
use crate::deck::Color;
use chrono::Local;
use std::fs;
use std::io::Write;

/// Markdown exporter for mana curve analysis reports
pub struct CurveReportExporter;

impl CurveReportExporter {
    /// Export curve analysis to a markdown file
    pub fn export(analysis: &CurveAnalysis, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = Self::generate(analysis);
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Generate markdown report as a string
    pub fn generate(analysis: &CurveAnalysis) -> String {
        let mut output = String::new();

        // Header
        output.push_str("# Scry Mana Curve Analysis\n\n");
        output.push_str(&format!(
            "Generated: {}\n\n",
            Local::now().format("%Y-%m-%d %H:%M")
        ));

        // Deck info
        if let Some(name) = &analysis.deck_name {
            output.push_str(&format!("**Deck**: {name}\n"));
        }
        if let Some(format) = &analysis.deck_format {
            output.push_str(&format!("**Format**: {format}\n"));
        }
        output.push_str(&format!("**Total Cards**: {}\n", analysis.total_cards));
        output.push_str(&format!("**Unique Cards**: {}\n\n", analysis.unique_cards));

        // Statistics
        output.push_str("## Statistics\n\n");
        output.push_str(&format!(
            "- **Average CMC**: {:.2}\n",
            analysis.stats.average_cmc
        ));
        output.push_str(&format!(
            "- **Median CMC**: {:.1}\n",
            analysis.stats.median_cmc
        ));
        output.push_str(&format!("- **Mode CMC**: {}\n", analysis.stats.mode_cmc));
        output.push_str(&format!(
            "- **Non-land Cards**: {}\n",
            analysis.stats.total_nonland_cards
        ));
        output.push_str(&format!(
            "- **Creatures**: {} ({:.1}%)\n",
            analysis.stats.total_creatures,
            if analysis.stats.total_nonland_cards > 0 {
                analysis.stats.total_creatures as f64 / analysis.stats.total_nonland_cards as f64
                    * 100.0
            } else {
                0.0
            }
        ));
        output.push_str(&format!(
            "- **Non-creatures**: {} ({:.1}%)\n\n",
            analysis.stats.total_non_creatures,
            if analysis.stats.total_nonland_cards > 0 {
                analysis.stats.total_non_creatures as f64
                    / analysis.stats.total_nonland_cards as f64
                    * 100.0
            } else {
                0.0
            }
        ));

        // Mana Pip Breakdown
        let pip = &analysis.pip_breakdown;
        let total_pips = pip.total();
        if total_pips > 0.0 {
            output.push_str("## Pip Breakdown\n\n");

            // Collect and sort by percentage (descending)
            let mut colors: Vec<(&str, f64)> = vec![
                ("☀️", pip.white),
                ("💧", pip.blue),
                ("💀", pip.black),
                ("🔥", pip.red),
                ("🌳", pip.green),
                ("◇", pip.colorless),
            ]
            .into_iter()
            .filter(|(_, count)| *count > 0.0)
            .collect();

            colors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            for (emoji, count) in colors {
                let pct = count / total_pips * 100.0;
                if count.fract() == 0.0 {
                    output.push_str(&format!(
                        "- {} {} pips ({:.1}%)\n",
                        emoji, count as u32, pct
                    ));
                } else {
                    output.push_str(&format!("- {emoji} {count:.1} pips ({pct:.1}%)\n"));
                }
            }
            output.push('\n');
        }

        // Card lists by CMC
        if !analysis.buckets.is_empty() {
            output.push_str("## Cards by CMC\n\n");
            for bucket in &analysis.buckets {
                output.push_str(&format!("### {} CMC\n\n", bucket.cmc));

                if !bucket.creature_names.is_empty() {
                    output.push_str("**Creatures:**\n");
                    for name in &bucket.creature_names {
                        output.push_str(&format!("- {name}\n"));
                    }
                    output.push('\n');
                }

                if !bucket.non_creature_names.is_empty() {
                    output.push_str("**Non-creatures:**\n");
                    for name in &bucket.non_creature_names {
                        output.push_str(&format!("- {name}\n"));
                    }
                    output.push('\n');
                }
            }
        }

        // Mana Base Recommendation
        if let Some(ref mana_base) = analysis.mana_base {
            output.push_str("## Mana Base Recommendation\n\n");

            // Show detection method
            if let Some(ref source) = analysis.land_source {
                let source_str = match source {
                    LandCountSource::UserProvided => "User specified".to_string(),
                    LandCountSource::DetectedFromDeck(count) => {
                        format!("Detected {count} lands in deck")
                    }
                    LandCountSource::FormatDefault(fmt) => format!("{fmt} format default"),
                };
                output.push_str(&format!(
                    "**Target Lands**: {} ({})\n\n",
                    analysis.target_lands.unwrap_or(0),
                    source_str
                ));
            }

            // Dual lands table (if any detected)
            if !mana_base.duals.is_empty() {
                let total_duals: u32 = mana_base.duals.iter().map(|d| d.count).sum();
                output.push_str(&format!(
                    "### Dual/Multi-color Lands ({total_duals} detected)\n\n"
                ));
                output.push_str("| Type | Colors | Count |\n");
                output.push_str("|------|--------|-------|\n");

                for dual in &mana_base.duals {
                    let colors_str: Vec<_> = dual.colors.iter().map(|c| c.symbol()).collect();
                    output.push_str(&format!(
                        "| {} | {} | {} |\n",
                        dual.name,
                        colors_str.join("/"),
                        dual.count
                    ));
                }
                output.push('\n');
            }

            // Basic lands table
            let total_basics: u32 = mana_base.basics.values().sum();
            output.push_str(&format!("### Basic Lands ({total_basics} recommended)\n\n"));
            output.push_str("| Land | Count | Percentage |\n");
            output.push_str("|------|-------|------------|\n");

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
                let pct = mana_base
                    .color_percentages
                    .get(color)
                    .copied()
                    .unwrap_or(0.0)
                    * 100.0;
                output.push_str(&format!(
                    "| {} | {} | {:.1}% |\n",
                    color.basic_land(),
                    count,
                    pct
                ));
            }
            output.push('\n');

            // Recommendations
            if !mana_base.recommendations.is_empty() {
                output.push_str("### Recommendations\n\n");
                for rec in &mana_base.recommendations {
                    output.push_str(&format!("- {rec}\n"));
                }
                output.push('\n');
            }
        }

        // Footer
        output.push_str("---\n\n");
        output.push_str("*Generated by scry mana curve analyzer*\n");

        output
    }
}
