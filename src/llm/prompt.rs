use crate::input::{DeckEntry, DeckList, DeckSection};
use crate::synergy::SynergyMatrix;

/// System prompt for the MTG deck analyst
pub const SYSTEM_PROMPT: &str = r#"You are an expert Magic: The Gathering deck analyst running inside a terminal/CLI application.

You are the second stage of a two-phase analysis system:
1. RULE-BASED ANALYSIS (already completed) - Pattern matching detected themes, keywords, and obvious synergies
2. YOUR ANALYSIS (now) - Find what the rules missed: subtle interactions, strategic insights, and improvements

CRITICAL: Your output will be displayed directly in a terminal. You MUST:
- NEVER use Markdown syntax (no **, `, #, [], (), etc.)
- ALWAYS use ANSI escape codes for styling
- Use ASCII/Unicode box-drawing for structure

═══════════════════════════════════════════════════════════════════════════════
FORMATTING GUIDE
═══════════════════════════════════════════════════════════════════════════════

ANSI Escape Codes:
- Bold: \x1b[1m ... \x1b[0m
- Dim: \x1b[2m ... \x1b[0m
- Italic: \x1b[3m ... \x1b[0m
- Underline: \x1b[4m ... \x1b[0m
- Bold Yellow (headers): \x1b[1;33m ... \x1b[0m
- Bold Cyan (subheaders): \x1b[1;36m ... \x1b[0m
- Green (positive/strengths): \x1b[32m ... \x1b[0m
- Red (warnings/weaknesses): \x1b[31m ... \x1b[0m
- Magenta (highlights): \x1b[35m ... \x1b[0m

Box-Drawing for Section Headers:
┌──────────────────────────────────┐
│ SECTION TITLE                    │
└──────────────────────────────────┘

Horizontal Dividers:
- Heavy: ═══════════════════════════
- Light: ───────────────────────────

Bullet Points:
- Use plain dashes: -
- Or arrows: →
- Or bullets: •

═══════════════════════════════════════════════════════════════════════════════

When analyzing decks:
- Be SPECIFIC: Always name actual cards, never say "cards like X" or "similar effects"
- Be ACTIONABLE: Format suggestions as "Cut [CARD] → Add [CARD]: [reason]"
- Be CONCISE: Quality over quantity; 3 great insights beat 10 generic ones
- Reference the rule-based analysis: Build on what it found, don't repeat it

Emojis are allowed sparingly: ⚔️ 🛡️ 💀 🔥 ✨ ⚡ 🌿 💧

REMINDER: NO MARKDOWN. Use ANSI codes and box-drawing characters only."#;

/// Detected deck format for analysis guidance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckFormat {
    Commander,
    Constructed60,
    Limited,
    Unknown,
}

impl DeckFormat {
    fn display_name(&self) -> &'static str {
        match self {
            DeckFormat::Commander => "Commander",
            DeckFormat::Constructed60 => "Constructed (60-card)",
            DeckFormat::Limited => "Limited",
            DeckFormat::Unknown => "Unknown Format",
        }
    }
}

/// Detect the deck format from the decklist
fn detect_format(deck: &DeckList) -> DeckFormat {
    // Check explicit format field first
    if let Some(fmt) = &deck.format {
        let fmt_lower = fmt.to_lowercase();
        if fmt_lower.contains("commander") || fmt_lower.contains("edh") {
            return DeckFormat::Commander;
        }
        if fmt_lower.contains("limited")
            || fmt_lower.contains("draft")
            || fmt_lower.contains("sealed")
        {
            return DeckFormat::Limited;
        }
        if fmt_lower.contains("standard")
            || fmt_lower.contains("modern")
            || fmt_lower.contains("pioneer")
            || fmt_lower.contains("legacy")
            || fmt_lower.contains("vintage")
            || fmt_lower.contains("pauper")
        {
            return DeckFormat::Constructed60;
        }
    }

    // Infer from structure
    let has_commander = deck.commanders().count() > 0;
    let total_cards = deck.total_cards();

    if has_commander {
        return DeckFormat::Commander;
    }

    // ~100 singleton cards suggests Commander even without explicit commander
    let unique = deck.unique_cards();
    if total_cards >= 95 && unique >= 90 {
        return DeckFormat::Commander;
    }

    if total_cards <= 45 {
        return DeckFormat::Limited;
    }

    if (55..=80).contains(&total_cards) {
        return DeckFormat::Constructed60;
    }

    DeckFormat::Unknown
}

/// Get format-specific analysis guidance
fn format_guidance(format: DeckFormat, deck: &DeckList) -> String {
    match format {
        DeckFormat::Commander => {
            let commander_names: Vec<_> = deck.commanders().map(|e| e.card_name.as_str()).collect();
            let commander_str = if commander_names.is_empty() {
                "the commander(s)".to_string()
            } else {
                commander_names.join(" and ")
            };

            format!(
                r#"This is a COMMANDER deck. Focus your analysis on:
- How well the 99 synergizes with {commander_str}
- Color pie coverage: Does the deck have enough removal, card draw, and ramp?
- Interaction density: Are there enough answers for a multiplayer game?
- Mana curve considerations for a longer, multiplayer format
- Provide a BRACKET RATING (1-5) with justification:
  1 = Ultra casual / precon-level
  2 = Casual / upgraded precon
  3 = Focused / optimized casual
  4 = High power / tuned
  5 = Competitive / cEDH"#
            )
        }
        DeckFormat::Constructed60 => {
            let has_sideboard = deck.sideboard().count() > 0;
            let sideboard_note = if has_sideboard {
                "\n- Sideboard plan: Are the 15 well-positioned against expected matchups?"
            } else {
                ""
            };

            format!(
                r#"This is a 60-CARD CONSTRUCTED deck. Focus your analysis on:
- Consistency: Are there enough playsets (4x) of key cards?
- Mana curve: Is it appropriate for the deck's speed?
- Threat density vs answers ratio{sideboard_note}
- Provide a POWER LEVEL estimate (casual / FNM / competitive)"#
            )
        }
        DeckFormat::Limited => r#"This is a LIMITED deck (draft/sealed). Focus your analysis on:
- Creature count and curve (typically want 15-17 creatures)
- Removal density (premium in limited)
- Mana base and splash viability
- Synergy vs raw card quality tradeoffs"#
            .to_string(),
        DeckFormat::Unknown => r#"Format unclear - provide general analysis:
- Overall strategy and game plan
- Card quality and synergy assessment
- Potential improvements"#
            .to_string(),
    }
}

/// Format the top synergy edges from the matrix
fn format_synergy_edges(matrix: &SynergyMatrix, limit: usize) -> String {
    if matrix.edges.is_empty() {
        return "No synergy connections detected by rule-based analysis.".to_string();
    }

    let mut output = String::new();

    // Take top edges (they should already be meaningful connections)
    for edge in matrix.edges.iter().take(limit) {
        output.push_str(&format!(
            "• {} + {}: {}\n",
            edge.card_a, edge.card_b, edge.reason
        ));
    }

    // Add count if truncated
    if matrix.edges.len() > limit {
        output.push_str(&format!(
            "\n({} more connections detected...)\n",
            matrix.edges.len() - limit
        ));
    }

    output
}

/// Build the synergy analysis prompt
pub fn build_synergy_prompt(deck: &DeckList, matrix: &SynergyMatrix, report: &str) -> String {
    let deck_list = format_deck_list(deck);
    let format = detect_format(deck);
    let format_name = format.display_name();
    let total_cards = deck.total_cards();
    let guidance = format_guidance(format, deck);
    let synergy_edges = format_synergy_edges(matrix, 15);

    // Get primary theme if detected
    let primary_theme = matrix
        .primary_theme
        .as_ref()
        .map(|t| t.display_name())
        .unwrap_or_else(|| "None detected".to_string());

    format!(
        r#"═══════════════════════════════════════════════════════════════════════════════
DECK ANALYSIS REQUEST
═══════════════════════════════════════════════════════════════════════════════

Format: {format_name}
Card Count: {total_cards} cards
Primary Theme: {primary_theme}

───────────────────────────────────────────────────────────────────────────────
DECK LIST
───────────────────────────────────────────────────────────────────────────────

{deck_list}

───────────────────────────────────────────────────────────────────────────────
RULE-BASED ANALYSIS (already completed)
───────────────────────────────────────────────────────────────────────────────

{report}

───────────────────────────────────────────────────────────────────────────────
DETECTED SYNERGIES (top connections found by rules)
───────────────────────────────────────────────────────────────────────────────

{synergy_edges}

═══════════════════════════════════════════════════════════════════════════════
YOUR TASK
═══════════════════════════════════════════════════════════════════════════════

{guidance}

Analyze this deck with the following priorities:

1. MISSED SYNERGIES
   The rule-based system found the connections listed above.
   Identify interactions it missed:
   → Non-obvious card combinations
   → Cross-theme synergies (cards that bridge multiple strategies)
   → Cards that secretly enable multiple strategies

2. STRATEGIC ASSESSMENT
   → Primary game plan and win conditions
   → Key weaknesses: what beats this deck?
   → Cards that don't fit the strategy (potential cuts)

3. SPECIFIC IMPROVEMENTS
   Format as: "Cut [CARD] → Add [CARD]: [reason]"
   → Name actual cards from the list and actual replacements
   → Consider budget when suggesting expensive cards
   → Prioritize impactful changes over minor optimizations

Remember: Use ANSI escape codes for formatting. NO MARKDOWN."#
    )
}

/// Format a single card entry in condensed format with full card details
fn format_card_condensed(entry: &DeckEntry) -> String {
    let prefix = if entry.section == DeckSection::Commander {
        format!("- {}", entry.card_name)
    } else {
        format!("{}x {}", entry.quantity, entry.card_name)
    };

    let Some(card) = &entry.card else {
        return prefix; // Fallback if card not hydrated
    };

    let cost = card.mana_cost.as_deref().unwrap_or("Land");
    let type_line = &card.type_line;

    // P/T for creatures only
    let pt = card
        .power_toughness()
        .map(|pt| format!(" | {pt}"))
        .unwrap_or_default();

    // Oracle text - combine all faces, replace newlines
    let oracle = card.all_oracle_text().join(" // ").replace('\n', " ");

    if oracle.is_empty() {
        format!("{prefix} {{{cost}}} | {type_line}{pt}")
    } else {
        format!("{prefix} {{{cost}}} | {type_line}{pt} | {oracle}")
    }
}

/// Format the deck list for the prompt
fn format_deck_list(deck: &DeckList) -> String {
    let mut output = String::new();

    // Add note if basic lands are excluded
    if deck.excludes_lands {
        output.push_str("NOTE: This decklist intentionally excludes basic lands (common Moxfield export practice). Do NOT suggest adding basic lands or flag the deck as incomplete.\n\n");
    }

    // Commander(s) first
    let commanders: Vec<_> = deck.commanders().collect();
    if !commanders.is_empty() {
        output.push_str("COMMANDER:\n");
        for entry in commanders {
            output.push_str(&format_card_condensed(entry));
            output.push('\n');
        }
        output.push('\n');
    }

    // Mainboard
    output.push_str("MAINBOARD:\n");
    for entry in deck.mainboard() {
        if entry.section != DeckSection::Commander {
            output.push_str(&format_card_condensed(entry));
            output.push('\n');
        }
    }

    // Sideboard
    let sideboard: Vec<_> = deck.sideboard().collect();
    if !sideboard.is_empty() {
        output.push_str("\nSIDEBOARD:\n");
        for entry in sideboard {
            output.push_str(&format_card_condensed(entry));
            output.push('\n');
        }
    }

    output
}
