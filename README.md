# MTG - Magic: The Gathering Deck Building Utilities

A command-line tool for Magic: The Gathering deck building, featuring mana base calculations and card lookups via the Scryfall API.

## Features

- **Mana Base Calculator**: Calculate optimal land distributions for your deck based on mana symbol counts
- **Card Lookup**: Search for cards by name or Scryfall ID with detailed information including prices and legalities
- **Multiple Formats**: Built-in presets for Commander, Standard, Modern, Limited, and Custom formats
- **Multiple Algorithms**: Simple proportional, CMC-weighted, and hypergeometric calculation methods
- **Interactive Mode**: Step-by-step wizard for configuring your deck
- **Export Support**: Save analysis results to Markdown or JSON

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd mtg

# Build the project
cargo build --release

# The binary will be at target/release/mtg
```

## Usage

### Mana Calculator

Interactive mode (recommended for first-time users):
```bash
mtg mana
```

With command-line options:
```bash
# Commander deck with specific colors
mtg mana --format commander --colors WUB

# Standard deck with CMC-weighted algorithm
mtg mana --format standard --colors RG --algorithm cmc

# Custom deck size
mtg mana --format custom --colors WU --cards 80 --lands 30

# Export results to file
mtg mana --format commander --colors WUBRG --export manabase.md
```

#### Format Presets

| Format    | Cards | Default Lands | Description                           |
|-----------|-------|---------------|---------------------------------------|
| Commander | 100   | 38            | 100-card singleton with a commander   |
| Standard  | 60    | 24            | 60-card constructed with recent sets  |
| Modern    | 60    | 24            | 60-card constructed (8th Edition+)    |
| Limited   | 40    | 17            | 40-card draft or sealed deck          |
| Custom    | 60    | 24            | User-defined deck size and land count |

#### Calculation Algorithms

- **Simple** (`--algorithm simple`): Proportional distribution based on mana symbol counts
- **CMC** (`--algorithm cmc`): Weighted by converted mana cost
- **Hypergeometric** (`--algorithm hypergeo`): Probability-based calculations (falls back to simple currently)

### Card Lookup

```bash
# Search by name (fuzzy matching)
mtg card "Lightning Bolt"

# Search by Scryfall ID
mtg card --id <scryfall-uuid>
```

Card lookup displays:
- Mana cost and type line
- Oracle text
- Power/toughness (for creatures)
- Set information and rarity
- Current prices (USD, foil)
- Format legalities

## Project Structure

```
src/
├── main.rs           # Entry point
├── api/              # Scryfall API client
│   ├── scryfall.rs   # HTTP client and card types
│   └── cache.rs      # Response caching
├── calculator/       # Mana calculation algorithms
│   ├── algorithms.rs # Calculator trait
│   ├── simple.rs     # Simple proportional calculator
│   ├── cmc_weighted.rs # CMC-weighted calculator
│   └── analyzer.rs   # Pip intensity analysis
├── cli/              # Command-line interface
│   ├── args.rs       # Clap argument definitions
│   ├── commands.rs   # Command handlers
│   └── interactive.rs # Interactive prompts
├── deck/             # Deck data structures
│   ├── types.rs      # Color, Deck, ManaBase types
│   ├── formats.rs    # Format presets
│   └── builder.rs    # Deck builder utilities
└── export/           # Export functionality
    ├── markdown.rs   # Markdown export
    └── json.rs       # JSON export
```

## Color Symbols

The tool uses standard MTG color abbreviations:

| Symbol | Color     | Basic Land |
|--------|-----------|------------|
| W      | White     | Plains     |
| U      | Blue      | Island     |
| B      | Black     | Swamp      |
| R      | Red       | Mountain   |
| G      | Green     | Forest     |
| C      | Colorless | Wastes     |

## Dependencies

- `clap` - Command-line argument parsing
- `dialoguer` - Interactive terminal prompts
- `reqwest` - HTTP client for Scryfall API
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `colored` - Terminal colors
- `chrono` - Date/time handling

## License

See LICENSE file for details.
