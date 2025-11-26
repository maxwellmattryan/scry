use colored::Colorize;

use crate::curve::CurveAnalysis;

const HISTOGRAM_WIDTH: usize = 40;
const BAR_CHAR: &str = "=";
const CREATURE_CHAR: &str = "#";
const NON_CREATURE_CHAR: &str = "-";

/// Display curve analysis results in the terminal
pub fn display_curve_analysis(analysis: &CurveAnalysis, by_type: bool) {
    println!();
    println!("{}", "=== MANA CURVE ANALYSIS ===".bold().green());
    println!();

    // Deck info
    if let Some(name) = &analysis.deck_name {
        println!("{}: {}", "Deck".yellow(), name);
    }
    if let Some(format) = &analysis.deck_format {
        println!("{}: {}", "Format".yellow(), format);
    }
    println!(
        "{}: {} cards ({} unique)",
        "Total".yellow(),
        analysis.total_cards,
        analysis.unique_cards
    );
    println!();

    // ASCII Histogram
    println!("{}", "Mana Curve Distribution:".cyan().bold());
    println!("{}", "-".repeat(60));
    println!();

    if analysis.buckets.is_empty() {
        println!("{}", "No non-land cards found in deck.".dimmed());
    } else if by_type {
        display_split_histogram(analysis);
    } else {
        display_combined_histogram(analysis);
    }

    println!();

    // Statistics Summary
    println!("{}", "Statistics:".cyan().bold());
    println!("{}", "-".repeat(60));
    println!(
        "  {}: {:.2}",
        "Average CMC".yellow(),
        analysis.stats.average_cmc
    );
    println!(
        "  {}: {:.1}",
        "Median CMC".yellow(),
        analysis.stats.median_cmc
    );
    println!("  {}: {}", "Mode CMC".yellow(), analysis.stats.mode_cmc);
    println!();
    println!(
        "  {}: {}",
        "Non-land Cards".yellow(),
        analysis.stats.total_nonland_cards
    );
    println!(
        "  {}: {} ({:.1}%)",
        "Creatures".yellow(),
        analysis.stats.total_creatures,
        if analysis.stats.total_nonland_cards > 0 {
            analysis.stats.total_creatures as f64 / analysis.stats.total_nonland_cards as f64
                * 100.0
        } else {
            0.0
        }
    );
    println!(
        "  {}: {} ({:.1}%)",
        "Non-creatures".yellow(),
        analysis.stats.total_non_creatures,
        if analysis.stats.total_nonland_cards > 0 {
            analysis.stats.total_non_creatures as f64 / analysis.stats.total_nonland_cards as f64
                * 100.0
        } else {
            0.0
        }
    );
    println!();

    // Distribution table
    if !analysis.buckets.is_empty() {
        println!("{}", "Distribution by CMC:".cyan().bold());
        println!("{}", "-".repeat(60));
        println!("  {:>3} | {:>5} | {:>6}", "CMC", "Count", "Percent");
        println!("  {}", "-".repeat(22));

        for bucket in &analysis.buckets {
            let pct = analysis
                .stats
                .cmc_distribution
                .get(&bucket.cmc)
                .copied()
                .unwrap_or(0.0)
                * 100.0;
            println!(
                "  {:>3} | {:>5} | {:>5.1}%",
                bucket.cmc, bucket.total_count, pct
            );
        }
        println!();
    }
}

fn display_combined_histogram(analysis: &CurveAnalysis) {
    let max_count = analysis.max_count.max(1);

    for bucket in &analysis.buckets {
        let bar_len =
            (bucket.total_count as f64 / max_count as f64 * HISTOGRAM_WIDTH as f64) as usize;
        let bar = BAR_CHAR.repeat(bar_len);
        println!(
            "  {:>2} | {} {}",
            bucket.cmc,
            bar.green(),
            format!("({})", bucket.total_count).dimmed()
        );
    }
}

fn display_split_histogram(analysis: &CurveAnalysis) {
    let max_creature = analysis
        .buckets
        .iter()
        .map(|b| b.creature_count)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_non_creature = analysis
        .buckets
        .iter()
        .map(|b| b.non_creature_count)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_count = max_creature.max(max_non_creature);

    println!(
        "  {} = Creatures  {} = Non-creatures",
        CREATURE_CHAR.green(),
        NON_CREATURE_CHAR.cyan()
    );
    println!();

    for bucket in &analysis.buckets {
        let creature_len =
            (bucket.creature_count as f64 / max_count as f64 * HISTOGRAM_WIDTH as f64) as usize;
        let non_creature_len =
            (bucket.non_creature_count as f64 / max_count as f64 * HISTOGRAM_WIDTH as f64) as usize;

        let creature_bar = CREATURE_CHAR.repeat(creature_len);
        let non_creature_bar = NON_CREATURE_CHAR.repeat(non_creature_len);

        println!(
            "  {:>2} | {}{}  {}",
            bucket.cmc,
            creature_bar.green(),
            non_creature_bar.cyan(),
            format!("({}/{})", bucket.creature_count, bucket.non_creature_count).dimmed()
        );
    }
}
