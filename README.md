# 🔮 Scry

[![Build](https://github.com/maxwellmattryan/scry/actions/workflows/ci.build.yml/badge.svg)](https://github.com/maxwellmattryan/scry/actions/workflows/ci.build.yml)
[![Lint](https://github.com/maxwellmattryan/scry/actions/workflows/ci.lint.yml/badge.svg)](https://github.com/maxwellmattryan/scry/actions/workflows/ci.lint.yml)
[![Format](https://github.com/maxwellmattryan/scry/actions/workflows/ci.fmt.yml/badge.svg)](https://github.com/maxwellmattryan/scry/actions/workflows/ci.fmt.yml)

**A command-line grimoire for the discerning planeswalker.**

Peer into the top cards of your library—powered by Rust, enlightened by the Multiverse. Scry combines the arcane mathematics of mana base construction with real-time card data from the aether. All from the depths of your terminal.

---

## ✦ Features

| | |
|---|---|
| **Mana Base Calculator** | Divine optimal land distribution through algorithmic sorcery |
| **Card Lookup** | Query the Scryfall and MTG.io APIs — prices, legalities, oracle text revealed |
| **Synergy Analysis** | Parse your decklist to uncover hidden interactions and combo lines |
| **Mana Curve Analysis** | Visualize your deck's CMC distribution with ASCII histograms and statistics |
| **Format Presets** | Commander, Standard, Modern, Limited, or compile your own Custom config |
| **Multiple Algorithms** | Simple proportional, CMC-weighted, or hypergeometric probability engines |
| **LLM Enhancement** | Neural networks trained on the Multiverse detect synergies beyond regex |
| **Interactive Mode** | Let Scry guide you through the ritual with intelligent prompts |
| **Export Results** | Serialize your findings to Markdown or JSON artifacts |

---

## ⚗️ Installation

```bash
# Summon the repository
git clone https://github.com/maxwellmattryan/scry.git
cd scry

# Forge the binary
cargo build --release

# Your artifact awaits at:
./target/release/scry
```

**Or install directly to your PATH:**
```bash
cargo install --path .
```

This binds the `scry` binary to your shell environment, accessible from any realm.

---

## ◈ Usage

### Mana Calculator

**Interactive mode** — Scry guides your hand:
```bash
scry mana
```

**Direct invocation** — for those who know the incantations:
```bash
# Esper Commander deck
scry mana --format commander --colors WUB

# Gruul Standard with CMC weighting
scry mana --format standard --colors RG --algorithm cmc

# Custom 80-card vessel
scry mana --format custom --colors WU --cards 80 --lands 30

# Inscribe results to parchment
scry mana --format commander --colors WUBRG --export manabase.md
```

**Available options:**
- `-f, --format <FORMAT>` — Format preset: `commander`, `standard`, `modern`, `limited`, `custom`
- `-a, --algorithm <ALGORITHM>` — Calculation algorithm: `simple` (default), `cmc`, `hypergeo`
- `-c, --colors <COLORS>` — Deck colors (e.g., WUB for Esper)
- `--cards <CARDS>` — Total cards in deck (required for custom format)
- `-l, --lands <LANDS>` — Target number of lands (required for custom format)
- `-e, --export <FILE>` — Export results to markdown file

### Card Lookup

```bash
# Seek by name (fuzzy search handles minor typos)
scry card "Lightning Bolt"

# Query by Scryfall ID
scry card --id <scryfall-uuid>

# Hit the mtg.io API instead
scry card "Sol Ring" --api mtgio

# Disable fallback to secondary API
scry card "Mox Ruby" --no-fallback
```

**Available options:**
- `<NAME>` — Card name to search for
- `--id <ID>` — Search by provider-specific ID
- `--api <API>` — API provider: `scryfall` (default), `mtgio`
- `--no-fallback` — Disable fallback to secondary API on failure

### Synergy Analysis

Parse your decklist and uncover hidden combo potential:
```bash
# Analyze a decklist file
scry synergy --input deck.txt

# Analyze a Moxfield deck URL
scry synergy --input "https://www.moxfield.com/decks/..."

# Enhanced analysis with LLM backend
scry synergy --input deck.txt --llm --provider anthropic

# Export detailed analysis to markdown
scry synergy --input deck.txt --export synergies.md --verbose

# Serialize to JSON for programmatic consumption
scry synergy --input deck.txt --json synergies.json

# For decklists exported from Moxfield (without basic lands)
scry synergy --input moxfield-deck.txt --excludes-lands
```

**Available options:**
- `-i, --input <INPUT>` — Path to decklist file or Moxfield URL (required)
- `--llm` — Enable LLM-enhanced synergy detection
- `--provider <PROVIDER>` — LLM provider: `anthropic`, `openai`, `ollama`
- `-e, --export <FILE>` — Export results to markdown file
- `--json <FILE>` — Export results to JSON file
- `-v, --verbose` — Show detailed card-by-card analysis
- `--api <API>` — API provider for card data: `scryfall` (default), `mtgio`
- `--no-fallback` — Disable fallback to secondary API on failure
- `--excludes-lands` — Indicates decklist excludes basic lands

### Mana Curve Analysis

Visualize your deck's mana curve distribution:
```bash
# Analyze a decklist file
scry curve --input deck.txt

# Analyze a Moxfield deck URL
scry curve --input "https://www.moxfield.com/decks/..."

# Show creatures vs non-creatures separately
scry curve --input deck.txt --by-type

# Export to markdown or JSON
scry curve --input deck.txt --export curve.md
scry curve --input deck.txt --json curve.json

# For decklists exported from Moxfield (without basic lands)
scry curve --input moxfield-deck.txt --excludes-lands
```

**Available options:**
- `-i, --input <INPUT>` — Path to decklist file or Moxfield URL (required)
- `--by-type` — Show creatures vs non-creatures separately in histogram
- `-e, --export <FILE>` — Export results to markdown file
- `--json <FILE>` — Export results to JSON file
- `--api <API>` — API provider for card data: `scryfall` (default), `mtgio`
- `--no-fallback` — Disable fallback to secondary API on failure
- `--excludes-lands` — Indicates decklist excludes basic lands

---

## ⬡ Color Symbols

The five pillars of mana, plus the void:

| Symbol | Color | Basic Land | |
|:------:|-------|------------|:---:|
| **W** | White | Plains | ☀️ |
| **U** | Blue | Island | 💧 |
| **B** | Black | Swamp | 💀 |
| **R** | Red | Mountain | 🔥 |
| **G** | Green | Forest | 🌲 |
| **C** | Colorless | Wastes | ◇ |

> *Why U for Blue? Black claimed B first. Such are the old ways.*

---

## 📜 Dependencies

The crates and ancient tomes upon which Scry is compiled:

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `dialoguer` | Interactive prompt rendering |
| `reqwest` | HTTP client for API communion |
| `tokio` | Async runtime engine |
| `serde` | Serialization and deserialization |
| `colored` | ANSI terminal enchantments |
| `chrono` | Temporal type bindings |

---

## ⚖️ License

Consult the [LICENSE](LICENSE) file.

---
