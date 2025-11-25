use crate::api::create_client;
use crate::calculator::{get_calculator, get_intensity_recommendations};
use crate::cli::{AlgorithmArg, ApiProviderArg, FormatArg, LlmProviderArg};
use crate::deck::{guild_name, Algorithm, Color, Deck};
use crate::export::{JsonExporter, MarkdownExporter, SynergyReportExporter};
use crate::input::{DeckListParser, MoxfieldClient, TextDecklistParser};
use crate::synergy::get_detector;
use colored::Colorize;

use super::interactive::{run_interactive_mana_flow, InteractiveConfig};
use super::synergy_display::{
    display_error, display_llm_insights, display_progress, display_synergy_matrix, display_warning,
};

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

pub async fn handle_card_command(
    name: Option<String>,
    id: Option<String>,
    api: ApiProviderArg,
    no_fallback: bool,
) {
    let client = create_client(api.to_provider(), !no_fallback);

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
            eprintln!("{}: {}", "Error".red(), e.message);
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

#[allow(clippy::too_many_arguments)]
pub async fn handle_synergy_command(
    input: String,
    llm: bool,
    llm_provider_arg: Option<LlmProviderArg>,
    export: Option<String>,
    json: Option<String>,
    verbose: bool,
    api: ApiProviderArg,
    no_fallback: bool,
    excludes_lands: bool,
) {
    println!();
    display_progress("Analyzing deck synergies...");
    println!();

    // 1. Parse the decklist
    let mut deck_list = if input.contains("moxfield.com")
        || MoxfieldClient::extract_deck_id(&input).is_some()
            && !std::path::Path::new(&input).exists()
    {
        display_progress("Fetching deck from Moxfield...");
        let client = MoxfieldClient::new();
        match client.parse(&input).await {
            Ok(deck) => deck,
            Err(e) => {
                display_error(&format!("Failed to fetch from Moxfield: {e}"));
                return;
            }
        }
    } else {
        display_progress("Parsing decklist file...");
        let parser = TextDecklistParser::new();
        match parser.parse(&input).await {
            Ok(deck) => deck,
            Err(e) => {
                display_error(&format!("Failed to parse decklist: {e}"));
                return;
            }
        }
    };

    // Set the excludes_lands flag from CLI
    deck_list.excludes_lands = excludes_lands;

    // 2. Hydrate with card data from selected provider
    let provider = api.to_provider();
    display_progress(&format!(
        "Fetching card data for {} cards from {}...",
        deck_list.unique_cards(),
        provider.name()
    ));

    let client = create_client(provider, !no_fallback);
    let card_names = deck_list.card_names();

    match client.batch_fetch_cards(card_names).await {
        Ok(cards) => {
            // Match fetched cards to deck entries
            for entry in &mut deck_list.entries {
                if let Some(card) = cards.get(&entry.card_name) {
                    entry.card = Some(card.clone());
                } else {
                    display_warning(&format!("Card not found: {}", entry.card_name));
                }
            }
        }
        Err(e) => {
            display_error(&format!("Failed to fetch card data: {}", e.message));
            return;
        }
    }

    let hydrated_count = deck_list
        .entries
        .iter()
        .filter(|e| e.card.is_some())
        .count();
    display_progress(&format!(
        "Hydrated {} of {} cards",
        hydrated_count,
        deck_list.entries.len()
    ));

    // 3. Run synergy analysis
    display_progress("Running synergy analysis...");
    let detector = get_detector();
    let matrix = detector.analyze(&deck_list);

    // 4. Display results
    display_synergy_matrix(&matrix, verbose);

    // 5. Run LLM-enhanced analysis if requested
    if llm {
        // Default to Anthropic if --provider not specified
        let llm_provider = llm_provider_arg
            .map(|p| p.to_provider())
            .unwrap_or(crate::llm::LlmProvider::Anthropic);

        display_progress(&format!(
            "Running LLM-enhanced analysis with {}...",
            llm_provider.name()
        ));

        let report = SynergyReportExporter::generate(&matrix);

        match crate::llm::create_llm_client(llm_provider) {
            Ok(client) => match client.analyze_synergies(&deck_list, &matrix, &report).await {
                Ok(result) => {
                    display_llm_insights(&result);
                }
                Err(e) => {
                    display_error(&format!("LLM analysis failed: {e}"));
                }
            },
            Err(e) => {
                display_error(&format!("Failed to initialize LLM: {e}"));
                display_warning(&format!(
                    "Set {} in your environment.",
                    llm_provider.env_var()
                ));
            }
        }
    }

    // 6. Export if requested
    if let Some(path) = export {
        match SynergyReportExporter::export(&matrix, &path) {
            Ok(_) => println!("{}", format!("Report saved to: {path}").green()),
            Err(e) => display_error(&format!("Failed to export: {e}")),
        }
    }

    if let Some(path) = json {
        match JsonExporter::export(&matrix, &path) {
            Ok(_) => println!("{}", format!("JSON saved to: {path}").green()),
            Err(e) => display_error(&format!("Failed to export JSON: {e}")),
        }
    }
}

pub fn print_help() {
    println!(
        "{}",
        "Scry - Magic: The Gathering Deck Building Utilities"
            .bold()
            .cyan()
    );
    println!();
    println!("{}", "USAGE:".yellow());
    println!("    scry <COMMAND>");
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
    println!(
        "    {} Analyze deck synergies from a decklist",
        "synergy".green()
    );
    println!("    {}    Print this help message", "help".green());
    println!();
    println!("{}", "EXAMPLES:".yellow());
    println!("    scry mana                           # Interactive mana calculator");
    println!("    scry mana --format commander        # Start with Commander preset");
    println!("    scry mana --algorithm cmc           # Use CMC-weighted algorithm");
    println!("    scry card \"Lightning Bolt\"          # Look up a card by name");
    println!("    scry card --id <scryfall-id>        # Look up a card by ID");
    println!("    scry synergy -i deck.txt            # Analyze synergies from file");
    println!("    scry synergy -i https://moxfield.com/decks/xyz  # From Moxfield");
    println!();
    println!("{}", "For more information on a command, run:".dimmed());
    println!("    scry <COMMAND> --help");
}
