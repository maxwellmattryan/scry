use crate::synergy::SynergyMatrix;
use chrono::Local;
use std::fs;
use std::io::Write;

/// Markdown exporter for synergy analysis reports
pub struct SynergyReportExporter;

impl SynergyReportExporter {
    /// Export synergy matrix to a markdown file
    pub fn export(matrix: &SynergyMatrix, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = Self::generate(matrix);
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Generate markdown report as a string
    pub fn generate(matrix: &SynergyMatrix) -> String {
        let mut output = String::new();

        // Header
        output.push_str("# MTG Deck Synergy Analysis\n\n");
        output.push_str(&format!(
            "Generated: {}\n\n",
            Local::now().format("%Y-%m-%d %H:%M")
        ));

        // Deck info
        if let Some(name) = &matrix.deck_name {
            output.push_str(&format!("**Deck**: {name}\n"));
        }
        if let Some(format) = &matrix.deck_format {
            output.push_str(&format!("**Format**: {format}\n"));
        }
        output.push_str(&format!("**Total Cards**: {}\n", matrix.total_cards));
        output.push_str(&format!("**Unique Cards**: {}\n\n", matrix.unique_cards));

        // Theme Analysis
        output.push_str("## Detected Themes\n\n");

        if matrix.detected_themes.is_empty() {
            output.push_str("No significant themes detected.\n\n");
        } else {
            output.push_str("| Theme | Cards | % of Deck |\n");
            output.push_str("|-------|-------|----------|\n");

            for theme in &matrix.detected_themes {
                output.push_str(&format!(
                    "| {} | {} | {:.1}% |\n",
                    theme.theme.display_name(),
                    theme.card_count,
                    theme.percentage * 100.0
                ));
            }
            output.push('\n');

            // Theme Details
            for theme in &matrix.detected_themes {
                output.push_str(&format!("### {}\n\n", theme.theme.display_name()));

                if !theme.enablers.is_empty() {
                    output.push_str("**Enablers:**\n");
                    for card in &theme.enablers {
                        output.push_str(&format!("- {card}\n"));
                    }
                    output.push('\n');
                }

                if !theme.payoffs.is_empty() {
                    output.push_str("**Payoffs:**\n");
                    for card in &theme.payoffs {
                        output.push_str(&format!("- {card}\n"));
                    }
                    output.push('\n');
                }

                if !theme.support.is_empty() {
                    output.push_str("**Support:**\n");
                    for card in &theme.support {
                        output.push_str(&format!("- {card}\n"));
                    }
                    output.push('\n');
                }
            }
        }

        // Statistics
        output.push_str("## Statistics\n\n");
        output.push_str(&format!(
            "- **Theme Coverage**: {:.1}%\n",
            matrix.stats.theme_coverage * 100.0
        ));
        output.push_str(&format!(
            "- **Synergy Density**: {:.1}%\n",
            matrix.stats.synergy_density * 100.0
        ));
        output.push_str(&format!(
            "- **Synergy Connections**: {}\n",
            matrix.edges.len()
        ));
        output.push('\n');

        // Hub Cards
        if !matrix.stats.hub_cards.is_empty() {
            output.push_str("### Hub Cards (Most Synergies)\n\n");
            for card in &matrix.stats.hub_cards {
                output.push_str(&format!("- {card}\n"));
            }
            output.push('\n');
        }

        // Orphan Cards
        if !matrix.stats.orphan_cards.is_empty() {
            output.push_str("### Cards with No Synergies\n\n");
            for card in &matrix.stats.orphan_cards {
                output.push_str(&format!("- {card}\n"));
            }
            output.push('\n');
        }

        // Keyword Distribution
        if !matrix.stats.keyword_distribution.is_empty() {
            output.push_str("### Keyword Distribution\n\n");
            output.push_str("| Keyword | Count |\n");
            output.push_str("|---------|-------|\n");

            let mut keywords: Vec<_> = matrix.stats.keyword_distribution.iter().collect();
            keywords.sort_by(|a, b| b.1.cmp(a.1));

            for (keyword, count) in keywords.iter().take(10) {
                output.push_str(&format!("| {keyword} | {count} |\n"));
            }
            output.push('\n');
        }

        // Observations
        if !matrix.observations.is_empty() {
            output.push_str("## Observations\n\n");
            for obs in &matrix.observations {
                output.push_str(&format!("- {obs}\n"));
            }
            output.push('\n');
        }

        // Footer
        output.push_str("---\n\n");
        output.push_str("*Generated by mtg-cli synergy analyzer*\n");

        output
    }
}
