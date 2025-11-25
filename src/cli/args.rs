use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "mtg")]
#[command(author = "MTG Mana Calculator")]
#[command(version = "0.1.0")]
#[command(about = "Magic: The Gathering deck building utilities", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Calculate optimal mana ratios for your deck
    Mana {
        /// Deck format preset
        #[arg(short, long, value_enum)]
        format: Option<FormatArg>,

        /// Calculation algorithm to use
        #[arg(short, long, value_enum, default_value = "simple")]
        algorithm: AlgorithmArg,

        /// Deck colors (e.g., WUB for White/Blue/Black)
        #[arg(short, long)]
        colors: Option<String>,

        /// Total cards in deck
        #[arg(long)]
        cards: Option<u32>,

        /// Target number of lands
        #[arg(short, long)]
        lands: Option<u32>,

        /// Export results to file
        #[arg(short, long)]
        export: Option<String>,
    },

    /// Look up card information from Scryfall
    Card {
        /// Card name to search for
        name: Option<String>,

        /// Search by Scryfall ID
        #[arg(long)]
        id: Option<String>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Commander,
    Standard,
    Modern,
    Limited,
    Custom,
}

impl FormatArg {
    pub fn to_format(&self) -> crate::deck::Format {
        match self {
            FormatArg::Commander => crate::deck::Format::Commander,
            FormatArg::Standard => crate::deck::Format::Standard,
            FormatArg::Modern => crate::deck::Format::Modern,
            FormatArg::Limited => crate::deck::Format::Limited,
            FormatArg::Custom => crate::deck::Format::Custom,
        }
    }
}

#[derive(Clone, Copy, ValueEnum, Default)]
pub enum AlgorithmArg {
    #[default]
    Simple,
    Cmc,
    Hypergeo,
}

impl AlgorithmArg {
    pub fn to_algorithm(&self) -> crate::deck::Algorithm {
        match self {
            AlgorithmArg::Simple => crate::deck::Algorithm::Simple,
            AlgorithmArg::Cmc => crate::deck::Algorithm::CmcWeighted,
            AlgorithmArg::Hypergeo => crate::deck::Algorithm::Hypergeometric,
        }
    }
}
