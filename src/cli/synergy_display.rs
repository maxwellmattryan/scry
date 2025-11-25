use colored::Colorize;

use crate::synergy::SynergyMatrix;

/// Display synergy analysis results in the terminal
pub fn display_synergy_matrix(matrix: &SynergyMatrix, verbose: bool) {
    println!();
    println!("{}", "=== SYNERGY ANALYSIS ===".bold().green());
    println!();

    // Deck info
    if let Some(name) = &matrix.deck_name {
        println!("{}: {}", "Deck".yellow(), name);
    }
    if let Some(format) = &matrix.deck_format {
        println!("{}: {}", "Format".yellow(), format);
    }
    println!(
        "{}: {} cards ({} unique)",
        "Total".yellow(),
        matrix.total_cards,
        matrix.unique_cards
    );
    println!();

    // Theme summary
    if matrix.detected_themes.is_empty() {
        println!("{}", "No significant themes detected.".dimmed());
    } else {
        println!("{}", "Detected Themes:".cyan().bold());
        println!("{}", "-".repeat(50));

        for theme in &matrix.detected_themes {
            let percentage = theme.percentage * 100.0;
            let bar = generate_bar(percentage, 20);
            println!(
                "  {}: {} cards ({:.1}%) {}",
                theme.theme.display_name().bold(),
                theme.card_count,
                percentage,
                bar.dimmed()
            );

            // Show enablers/payoffs if any
            if !theme.enablers.is_empty() {
                println!(
                    "    {} {}",
                    "Enablers:".green(),
                    truncate_list(&theme.enablers, 3)
                );
            }
            if !theme.payoffs.is_empty() {
                println!(
                    "    {} {}",
                    "Payoffs:".yellow(),
                    truncate_list(&theme.payoffs, 3)
                );
            }
        }
        println!();
    }

    // Statistics
    println!("{}", "Statistics:".cyan().bold());
    println!("{}", "-".repeat(50));
    println!(
        "  {}: {:.1}%",
        "Theme Coverage".yellow(),
        matrix.stats.theme_coverage * 100.0
    );
    println!(
        "    {}",
        "(% of cards belonging to at least one theme)".dimmed()
    );
    println!(
        "  {}: {:.1}%",
        "Synergy Density".yellow(),
        matrix.stats.synergy_density * 100.0
    );
    println!(
        "    {}",
        "(% of possible card pairings that synergize)".dimmed()
    );
    println!(
        "  {}: {}",
        "Synergy Connections".yellow(),
        matrix.edges.len()
    );
    println!(
        "    {}",
        "(total card-to-card synergy links detected)".dimmed()
    );
    println!();

    // Hub cards
    if !matrix.stats.hub_cards.is_empty() {
        println!("{}", "Hub Cards (Most Synergies):".green());
        for card in &matrix.stats.hub_cards {
            println!("  + {card}");
        }
        println!();
    }

    // Orphan cards
    if !matrix.stats.orphan_cards.is_empty() {
        let orphan_count = matrix.stats.orphan_cards.len();
        if orphan_count <= 10 {
            println!("{}", "Cards with No Synergies:".yellow());
            for orphan in &matrix.stats.orphan_cards {
                println!(
                    "  - {} {}",
                    orphan.name.dimmed(),
                    format!("({})", orphan.reason).dimmed()
                );
            }
        } else {
            println!(
                "{}: {} cards have no detected synergies",
                "Note".yellow(),
                orphan_count
            );
            if verbose {
                for orphan in &matrix.stats.orphan_cards {
                    println!(
                        "  - {} {}",
                        orphan.name.dimmed(),
                        format!("({})", orphan.reason).dimmed()
                    );
                }
            }
        }
        println!();
    }

    // Keyword distribution
    if !matrix.stats.keyword_distribution.is_empty() {
        let mut keywords: Vec<_> = matrix.stats.keyword_distribution.iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(a.1));

        if !keywords.is_empty() {
            println!("{}", "Top Keywords:".cyan().bold());
            for (keyword, count) in keywords.iter().take(8) {
                println!("  {keyword} ({count})");
            }
            println!();
        }
    }

    // Observations
    if !matrix.observations.is_empty() {
        println!("{}", "Observations:".cyan().bold());
        println!("{}", "-".repeat(50));
        for obs in &matrix.observations {
            println!("  * {obs}");
        }
        println!();
    }

    // Verbose card-by-card breakdown
    if verbose {
        println!("{}", "Card-by-Card Analysis:".cyan().bold());
        println!("{}", "-".repeat(50));

        let mut cards: Vec<_> = matrix.card_profiles.values().collect();
        cards.sort_by(|a, b| {
            b.synergy_score
                .partial_cmp(&a.synergy_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for profile in cards {
            println!();
            println!("  {}", profile.card_name.bold());

            if !profile.themes.is_empty() {
                let theme_names: Vec<_> = profile.themes.iter().map(|t| t.display_name()).collect();
                println!("    {}: {}", "Themes".yellow(), theme_names.join(", "));
            }

            if !profile.keywords.is_empty() {
                let keyword_names: Vec<_> =
                    profile.keywords.iter().map(|k| k.display_name()).collect();
                println!("    {}: {}", "Keywords".yellow(), keyword_names.join(", "));
            }

            if let Some(role) = &profile.role {
                println!("    {}: {:?}", "Role".yellow(), role);
            }
        }
        println!();
    }
}

/// Generate a simple bar chart
fn generate_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

/// Truncate a list of items for display
fn truncate_list(items: &[String], max: usize) -> String {
    if items.len() <= max {
        items.join(", ")
    } else {
        let shown: Vec<_> = items.iter().take(max).cloned().collect();
        format!("{}, +{} more", shown.join(", "), items.len() - max)
    }
}

/// Display a simple progress message
pub fn display_progress(message: &str) {
    println!("{}", message.dimmed());
}

/// Display an error message
pub fn display_error(message: &str) {
    eprintln!("{}: {}", "Error".red().bold(), message);
}

/// Display a warning message
pub fn display_warning(message: &str) {
    eprintln!("{}: {}", "Warning".yellow(), message);
}
