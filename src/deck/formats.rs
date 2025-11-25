use super::types::Format;

pub struct FormatPreset {
    pub format: Format,
    pub description: &'static str,
}

impl FormatPreset {
    pub fn all() -> Vec<FormatPreset> {
        vec![
            FormatPreset {
                format: Format::Commander,
                description: "100-card singleton with a commander",
            },
            FormatPreset {
                format: Format::Standard,
                description: "60-card constructed with recent sets",
            },
            FormatPreset {
                format: Format::Modern,
                description: "60-card constructed with 8th Edition onwards",
            },
            FormatPreset {
                format: Format::Limited,
                description: "40-card draft or sealed deck",
            },
            FormatPreset {
                format: Format::Custom,
                description: "User-defined deck size and land count",
            },
        ]
    }

    pub fn names() -> Vec<String> {
        Self::all()
            .iter()
            .map(|p| format!("{} - {}", p.format.name(), p.description))
            .collect()
    }
}
