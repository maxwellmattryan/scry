# 🌑 MTG Deck Builder

[![Lint](https://github.com/maxwellmattryan/mtg/actions/workflows/lint.yml/badge.svg)](https://github.com/maxwellmattryan/mtg/actions/workflows/lint.yml)
[![Format](https://github.com/maxwellmattryan/mtg/actions/workflows/format.yml/badge.svg)](https://github.com/maxwellmattryan/mtg/actions/workflows/format.yml)

**A command-line grimoire for the discerning planeswalker.**

Channel the arcane mathematics of mana base construction. Scry the Multiverse for card knowledge. All from the depths of your terminal.

---

## ✦ Features

| | |
|---|---|
| **Mana Base Calculator** | Divine the optimal land distribution through ancient algorithmic arts |
| **Card Lookup** | Peer into the Scryfall archives — prices, legalities, oracle text revealed |
| **Format Presets** | Commander, Standard, Modern, Limited, or forge your own Custom path |
| **Multiple Algorithms** | Simple proportional, CMC-weighted, or hypergeometric calculations |
| **Interactive Mode** | Let the tool guide you through the ritual with `mtg mana` |
| **Export Results** | Inscribe your findings to Markdown or JSON scrolls |

---

## ⚗️ Installation

```bash
# Summon the repository
git clone https://github.com/maxwellmattryan/mtg.git
cd mtg

# Forge the binary
cargo build --release

# Your artifact awaits at:
./target/release/mtg
```

**Or install directly to your PATH:**
```bash
cargo install --path .
```

This binds the `mtg` command to your shell, accessible from any realm.

---

## ◈ Usage

### Mana Calculator

**Interactive mode** — the tool guides your hand:
```bash
mtg mana
```

**Direct invocation** — for those who know the incantations:
```bash
# Esper Commander deck
mtg mana --format commander --colors WUB

# Gruul Standard with CMC weighting
mtg mana --format standard --colors RG --algorithm cmc

# Custom 80-card vessel
mtg mana --format custom --colors WU --cards 80 --lands 30

# Inscribe results to parchment
mtg mana --format commander --colors WUBRG --export manabase.md
```

### Card Lookup

```bash
# Seek by name (fuzzy matching penetrates minor misspellings)
mtg card "Lightning Bolt"

# Divine by Scryfall ID
mtg card --id <scryfall-uuid>
```

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

The ancient tomes upon which this work is built:

| Crate | Purpose |
|-------|---------|
| `clap` | Argument parsing |
| `dialoguer` | Interactive prompts |
| `reqwest` | Scryfall API communion |
| `tokio` | Async runtime |
| `serde` | Serialization rites |
| `colored` | Terminal enchantments |
| `chrono` | Temporal bindings |

---

## ⚖️ License

Consult the [LICENSE](LICENSE) file.

---
