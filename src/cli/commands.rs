use crate::api::ScryfallClient;
use crate::calculator::{get_calculator, get_intensity_recommendations};
use crate::cli::{AlgorithmArg, FormatArg};
use crate::deck::{guild_name, Algorithm, Color, Deck};
use crate::export::MarkdownExporter;
use colored::Colorize;

use super::interactive::{run_interactive_mana_flow, InteractiveConfig};

pub async fn handle_mana_command(
    format: Option<FormatArg>,
    algorithm: AlgorithmArg,
    colors: Option<String>,
    cards: Option<u32>,
    lands: Option<u32>,
    export: Option<String>,
) {
    let algo = algorithm.to_algorithm();

    // Check if we have all required params for non-interactive mode
    let has_all_params = format.is_some() && colors.is_some();

    if has_all_params {
        // Non-interactive mode
        let fmt = format.unwrap().to_format();
        let color_list = parse_colors(&colors.unwrap());

        if color_list.is_empty() {
            eprintln!("{}", "Error: No valid colors provided".red());
            return;
        }

        let total_cards = cards.unwrap_or(fmt.default_cards());
        let target_lands = lands.unwrap_or(fmt.default_lands());

        // Build a basic deck - in non-interactive mode we don't have symbol counts
        // so we'll assume equal distribution
        let mut deck = Deck::new(fmt);
        deck.total_cards = total_cards;
        deck.target_lands = target_lands;
        deck.colors = color_list.clone();

        // Equal mana symbol distribution for non-interactive
        let symbols_per_color = 20; // Default assumption
        for color in &color_list {
            deck.mana_symbols.insert(*color, symbols_per_color);
        }

        run_calculation(&deck, algo, export);
    } else {
        // Interactive mode
        let config = InteractiveConfig {
            preset_format: format.map(|f| f.to_format()),
            preset_algorithm: Some(algo),
            preset_colors: colors.map(|c| parse_colors(&c)),
            preset_cards: cards,
            preset_lands: lands,
            export_path: export,
        };

        if let Err(e) = run_interactive_mana_flow(config).await {
            eprintln!("{}: {}", "Error".red(), e);
        }
    }
}

pub async fn handle_card_command(name: Option<String>, id: Option<String>) {
    let client = ScryfallClient::new();

    let result = if let Some(card_id) = id {
        client.get_card_by_id(&card_id).await
    } else if let Some(card_name) = name {
        client.search_card(&card_name).await
    } else {
        eprintln!("{}", "Error: Please provide a card name or --id".red());
        return;
    };

    match result {
        Ok(card) => {
            println!();
            println!("{}", card.name.bold().cyan());
            println!("{}", "─".repeat(40));

            if let Some(cost) = &card.mana_cost {
                println!("{}: {}", "Mana Cost".yellow(), cost);
            }

            println!("{}: {}", "Type".yellow(), card.type_line);

            if let Some(text) = &card.oracle_text {
                println!();
                println!("{}", "Oracle Text:".yellow());
                for line in text.lines() {
                    println!("  {line}");
                }
            }

            if let Some(pt) = card.power_toughness() {
                println!();
                println!("{}: {}", "P/T".yellow(), pt);
            }

            println!();
            println!("{}", "Set Information:".yellow());
            println!("  Set: {} ({})", card.set_name, card.set.to_uppercase());
            println!("  Rarity: {}", capitalize(&card.rarity));

            if let Some(prices) = &card.prices {
                println!();
                println!("{}", "Prices:".yellow());
                if let Some(usd) = &prices.usd {
                    println!("  TCGPlayer: ${usd}");
                }
                if let Some(usd_foil) = &prices.usd_foil {
                    println!("  TCGPlayer (Foil): ${usd_foil}");
                }
            }

            println!();
            println!("{}", "Legalities:".yellow());
            let key_formats = ["standard", "modern", "legacy", "commander", "vintage"];
            for format in key_formats {
                if let Some(legality) = card.legalities.get(format) {
                    let status = match legality.as_str() {
                        "legal" => "Legal".green(),
                        "not_legal" => "Not Legal".normal(),
                        "banned" => "Banned".red(),
                        "restricted" => "Restricted".yellow(),
                        _ => legality.normal(),
                    };
                    println!("  {}: {}", capitalize(format), status);
                }
            }

            println!();
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red(), e);
        }
    }
}

pub fn run_calculation(deck: &Deck, algorithm: Algorithm, export: Option<String>) {
    let calculator = get_calculator(algorithm);
    let mana_base = calculator.calculate(deck);

    println!();
    println!("{}", "=== MANA BASE RECOMMENDATION ===".bold().green());
    println!();

    // Basic lands
    let total_basics: u32 = mana_base.basics.values().sum();
    println!(
        "{}",
        format!("Basic Lands ({total_basics} total):").yellow()
    );

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
        let percentage = mana_base
            .color_percentages
            .get(color)
            .copied()
            .unwrap_or(0.0);
        println!(
            "  • {}: {} ({:.1}%)",
            color.basic_land(),
            count,
            percentage * 100.0
        );
    }

    // Dual lands
    if !mana_base.duals.is_empty() {
        let total_duals: u32 = mana_base.duals.iter().map(|d| d.count).sum();
        println!();
        println!("{}", format!("Dual Lands ({total_duals} total):").yellow());

        for dual in &mana_base.duals {
            let name = if dual.colors.len() == 2 {
                guild_name(&dual.colors)
                    .map(|g| g.to_string())
                    .unwrap_or_else(|| dual.name.clone())
            } else {
                dual.name.clone()
            };

            let colors_str: Vec<_> = dual.colors.iter().map(|c| c.symbol()).collect();
            println!("  • {} ({}): {}", name, colors_str.join("/"), dual.count);
        }
    }

    // Pip intensity analysis
    let recommendations = get_intensity_recommendations(deck);
    if !recommendations.is_empty() {
        println!();
        println!("{}", "Pip Intensity Analysis:".yellow());
        for rec in &recommendations {
            println!("  {} {}", "⚠".yellow(), rec);
        }
    }

    // Additional recommendations from mana base
    if !mana_base.recommendations.is_empty() {
        for rec in &mana_base.recommendations {
            println!("  {} {}", "⚠".yellow(), rec);
        }
    }

    println!();

    // Export if requested
    if let Some(path) = export {
        match MarkdownExporter::export(deck, &mana_base, &path) {
            Ok(_) => println!("{}", format!("Results saved to: {path}").green()),
            Err(e) => eprintln!("{}: {}", "Export error".red(), e),
        }
    }
}

fn parse_colors(input: &str) -> Vec<Color> {
    input
        .chars()
        .filter_map(|c| Color::from_symbol(&c.to_string()))
        .collect()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

pub fn print_help() {
    println!(
        "{}",
        "MTG - Magic: The Gathering Deck Building Utilities"
            .bold()
            .cyan()
    );
    println!();
    println!("{}", "USAGE:".yellow());
    println!("    mtg <COMMAND>");
    println!();
    println!("{}", "COMMANDS:".yellow());
    println!(
        "    {}    Calculate optimal mana ratios for your deck",
        "mana".green()
    );
    println!(
        "    {}    Look up card information from Scryfall",
        "card".green()
    );
    println!("    {}    Print this help message", "help".green());
    println!();
    println!("{}", "EXAMPLES:".yellow());
    println!("    mtg mana                           # Interactive mana calculator");
    println!("    mtg mana --format commander        # Start with Commander preset");
    println!("    mtg mana --algorithm cmc           # Use CMC-weighted algorithm");
    println!("    mtg card \"Lightning Bolt\"          # Look up a card by name");
    println!("    mtg card --id <scryfall-id>        # Look up a card by ID");
    println!();
    println!("{}", "For more information on a command, run:".dimmed());
    println!("    mtg <COMMAND> --help");
}
