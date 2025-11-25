mod api;
mod calculator;
mod cli;
mod deck;
mod export;

use clap::Parser;
use cli::{handle_card_command, handle_mana_command, print_help, Cli, Commands};

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
        None => {
            print_help();
        }
    }
}
