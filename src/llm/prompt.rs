use crate::input::{DeckList, DeckSection};
use crate::synergy::SynergyMatrix;

/// System prompt for the MTG deck analyst
pub const SYSTEM_PROMPT: &str = r#"You are an expert Magic: The Gathering deck analyst and strategist with deep knowledge of card interactions, meta strategies, and deck building principles across all formats.

Analyze decks and provide actionable insights. Focus on:
1. Identifying synergies that pattern-matching might miss
2. Strategic insights about the deck's game plan
3. Potential weaknesses and suggested improvements

Be concise and specific. Reference actual card names from the deck list.

Format your response for terminal display using ANSI escape codes:
- Section headers: \x1b[1;33m (bold yellow), then \x1b[0m to reset
- Subheaders: \x1b[1;36m (bold cyan), then \x1b[0m to reset
- Card names: \x1b[1m (bold), then \x1b[0m to reset
- Positive notes/strengths: \x1b[32m (green), then \x1b[0m to reset
- Warnings/weaknesses: \x1b[31m (red), then \x1b[0m to reset
- Less important text: \x1b[2m (dimmed), then \x1b[0m to reset
- Use plain dashes (-) for bullet points
- Do NOT use markdown syntax (**, `, #, etc.)"#;

/// Build the synergy analysis prompt for Claude
pub fn build_synergy_prompt(deck: &DeckList, _matrix: &SynergyMatrix, report: &str) -> String {
    let deck_list = format_deck_list(deck);

    format!(
        r#"## Deck Analysis Request

### Deck List
{deck_list}

### Rule-Based Analysis Results
{report}

---

## Your Task

Analyze this Magic: The Gathering deck and provide insights beyond the rule-based analysis.

### 1. Missed Synergies
Identify card interactions the automated analysis may have missed:
- Non-obvious card combinations
- Cross-theme synergies
- Subtle enabler/payoff relationships

### 2. Strategic Assessment
Evaluate the deck's strategy:
- Primary game plan and win conditions
- Strengths and weaknesses
- How well card choices support the strategy

### 3. Improvement Suggestions
Provide specific, actionable suggestions:
- Cards that seem out of place
- Missing effects the deck needs
- Ratio adjustments (lands, threats, answers)
- Cards to consider adding

Remember to use ANSI escape codes for formatting as specified in your instructions."#
    )
}

/// Format the deck list for the prompt
fn format_deck_list(deck: &DeckList) -> String {
    let mut output = String::new();

    // Commander(s) first
    let commanders: Vec<_> = deck.commanders().collect();
    if !commanders.is_empty() {
        output.push_str("**Commander:**\n");
        for entry in commanders {
            output.push_str(&format!("- {}\n", entry.card_name));
        }
        output.push('\n');
    }

    // Mainboard
    output.push_str("**Mainboard:**\n");
    for entry in deck.mainboard() {
        if entry.section != DeckSection::Commander {
            output.push_str(&format!("{}x {}\n", entry.quantity, entry.card_name));
        }
    }

    // Sideboard
    let sideboard: Vec<_> = deck.sideboard().collect();
    if !sideboard.is_empty() {
        output.push_str("\n**Sideboard:**\n");
        for entry in sideboard {
            output.push_str(&format!("{}x {}\n", entry.quantity, entry.card_name));
        }
    }

    output
}
