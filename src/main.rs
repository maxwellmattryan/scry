mod api;
mod calculator;
mod cli;
mod deck;
mod export;
mod input;
mod synergy;

use clap::Parser;
use cli::{
    handle_card_command, handle_mana_command, handle_synergy_command, print_help, Cli, Commands,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Mana {
            format,
            algorithm,
            colors,
            cards,
            lands,
            export,
        }) => {
            handle_mana_command(format, algorithm, colors, cards, lands, export).await;
        }
        Some(Commands::Card { name, id }) => {
            handle_card_command(name, id).await;
        }
        Some(Commands::Synergy {
            input,
            llm,
            provider,
            export,
            json,
            verbose,
        }) => {
            handle_synergy_command(input, llm, provider, export, json, verbose).await;
        }
        None => {
            print_help();
        }
    }
}
