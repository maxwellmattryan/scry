use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::collections::HashMap;

use crate::calculator::get_calculator;
use crate::deck::{guild_name, Algorithm, Color, Deck, DualLand, Format, FormatPreset};
use crate::export::MarkdownExporter;

use super::commands::run_calculation;

#[derive(Default)]
pub struct InteractiveConfig {
    pub preset_format: Option<Format>,
    pub preset_algorithm: Option<Algorithm>,
    pub preset_colors: Option<Vec<Color>>,
    pub preset_cards: Option<u32>,
    pub preset_lands: Option<u32>,
    pub export_path: Option<String>,
}

pub async fn run_interactive_mana_flow(
    config: InteractiveConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("{}", "Welcome to MTG Mana Calculator!".bold().cyan());
    println!();

    // 1. Select format
    let format = if let Some(fmt) = config.preset_format {
        fmt
    } else {
        select_format()?
    };

    // 2. Get deck size
    let total_cards = if let Some(cards) = config.preset_cards {
        cards
    } else if format == Format::Custom {
        prompt_deck_size()?
    } else {
        format.default_cards()
    };

    // 3. Get target lands
    let target_lands = if let Some(lands) = config.preset_lands {
        lands
    } else {
        prompt_land_count(format)?
    };

    // 4. Select colors
    let colors = if let Some(c) = config.preset_colors {
        c
    } else {
        select_colors()?
    };

    if colors.is_empty() {
        return Err("No colors selected".into());
    }

    // 5. Count mana symbols for each color
    println!();
    println!("{}", "Let's count mana symbols for each color.".cyan());
    println!(
        "{}",
        "(Count each pip individually - a {W}{W} card counts as 2)".dimmed()
    );
    println!();

    let mut mana_symbols: HashMap<Color, u32> = HashMap::new();
    let mut pip_intensity: HashMap<Color, u32> = HashMap::new();

    for color in &colors {
        let count = prompt_mana_symbol_count(color)?;
        mana_symbols.insert(*color, count);

        let intensity = prompt_pip_intensity(color)?;
        pip_intensity.insert(*color, intensity);
    }

    // 6. Dual lands
    let dual_lands = prompt_dual_lands(&colors)?;

    // Build the deck
    let mut deck = Deck::new(format);
    deck.total_cards = total_cards;
    deck.target_lands = target_lands;
    deck.colors = colors;
    deck.mana_symbols = mana_symbols;
    deck.dual_lands = dual_lands;
    deck.pip_intensity = pip_intensity;

    // Get algorithm
    let algorithm = config.preset_algorithm.unwrap_or(Algorithm::Simple);

    // Run calculation and display results
    run_calculation(&deck, algorithm, None);

    // Ask about export
    let export_path = if config.export_path.is_some() {
        config.export_path
    } else {
        prompt_export()?
    };

    if let Some(path) = export_path {
        let calculator = get_calculator(algorithm);
        let mana_base = calculator.calculate(&deck);
        MarkdownExporter::export(&deck, &mana_base, &path)?;
        println!("{}", format!("Results saved to: {path}").green());
    }

    Ok(())
}

fn select_format() -> Result<Format, Box<dyn std::error::Error>> {
    let presets = FormatPreset::all();
    let options: Vec<String> = presets
        .iter()
        .map(|p| format!("{} - {}", p.format.name(), p.description))
        .collect();

    let selection = Select::new()
        .with_prompt("Select your deck format")
        .items(&options)
        .default(0)
        .interact()?;

    Ok(presets[selection].format)
}

fn prompt_deck_size() -> Result<u32, Box<dyn std::error::Error>> {
    let size: u32 = Input::new()
        .with_prompt("How many cards in your deck?")
        .default(60)
        .interact_text()?;

    Ok(size)
}

fn prompt_land_count(format: Format) -> Result<u32, Box<dyn std::error::Error>> {
    let (min, max) = format.recommended_land_range();
    let default = format.default_lands();

    let lands: u32 = Input::new()
        .with_prompt(format!(
            "How many lands do you want to run? [{min}-{max} recommended]"
        ))
        .default(default)
        .interact_text()?;

    Ok(lands)
}

fn select_colors() -> Result<Vec<Color>, Box<dyn std::error::Error>> {
    let color_options = vec!["White (W)", "Blue (U)", "Black (B)", "Red (R)", "Green (G)"];

    let selections = MultiSelect::new()
        .with_prompt("Which colors are in your deck? (Space to select, Enter to confirm)")
        .items(&color_options)
        .interact()?;

    let all_colors = Color::all_colors();
    let colors: Vec<Color> = selections.into_iter().map(|i| all_colors[i]).collect();

    Ok(colors)
}

fn prompt_mana_symbol_count(color: &Color) -> Result<u32, Box<dyn std::error::Error>> {
    let count: u32 = Input::new()
        .with_prompt(format!(
            "How many {{{}}} symbols appear in total for {} cards?",
            color.symbol(),
            color.name().to_uppercase()
        ))
        .default(0)
        .interact_text()?;

    Ok(count)
}

fn prompt_pip_intensity(color: &Color) -> Result<u32, Box<dyn std::error::Error>> {
    let count: u32 = Input::new()
        .with_prompt(format!(
            "How many cards have {{{}}}{{{}}} or more (double+ pips)?",
            color.symbol(),
            color.symbol()
        ))
        .default(0)
        .interact_text()?;

    Ok(count)
}

fn prompt_dual_lands(colors: &[Color]) -> Result<Vec<DualLand>, Box<dyn std::error::Error>> {
    if colors.len() < 2 {
        return Ok(Vec::new());
    }

    println!();
    let has_duals = Confirm::new()
        .with_prompt("Do you have any dual or multi-colored lands?")
        .default(false)
        .interact()?;

    if !has_duals {
        return Ok(Vec::new());
    }

    let mut dual_lands = Vec::new();

    // Generate all possible color pairs from selected colors
    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            let pair = vec![colors[i], colors[j]];
            let name = guild_name(&pair)
                .map(|g| g.to_string())
                .unwrap_or_else(|| format!("{}/{}", colors[i].symbol(), colors[j].symbol()));

            let count: u32 = Input::new()
                .with_prompt(format!(
                    "How many {} ({}/{}) lands?",
                    name,
                    colors[i].symbol(),
                    colors[j].symbol()
                ))
                .default(0)
                .interact_text()?;

            if count > 0 {
                dual_lands.push(DualLand::new(name, pair, count));
            }
        }
    }

    // If 3+ colors, ask about tri-lands
    if colors.len() >= 3 {
        let has_tri = Confirm::new()
            .with_prompt("Do you have any tri-color lands?")
            .default(false)
            .interact()?;

        if has_tri {
            let name: String = Input::new()
                .with_prompt("Name of tri-land (e.g., 'Command Tower')")
                .interact_text()?;

            let count: u32 = Input::new()
                .with_prompt("How many copies?")
                .default(1)
                .interact_text()?;

            if count > 0 {
                dual_lands.push(DualLand::new(name, colors.to_vec(), count));
            }
        }
    }

    Ok(dual_lands)
}

fn prompt_export() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let should_export = Confirm::new()
        .with_prompt("Export results to file?")
        .default(false)
        .interact()?;

    if !should_export {
        return Ok(None);
    }

    let default_name = format!(
        "mtg_manabase_{}.md",
        chrono::Local::now().format("%Y-%m-%d")
    );

    let filename: String = Input::new()
        .with_prompt("Filename")
        .default(default_name)
        .interact_text()?;

    Ok(Some(filename))
}
