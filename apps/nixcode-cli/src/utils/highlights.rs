use ratatui::prelude::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

lazy_static::lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref THEME: Theme = {
        let themes = ThemeSet::load_defaults();

        themes.themes["base16-ocean.dark"].clone()
    };
}

fn syntect_style_to_ratatui(syntect_style: SyntectStyle) -> Style {
    let mut style = Style::default();
    if syntect_style.foreground != syntect::highlighting::Color::BLACK {
        // Unikaj domyślnego tła jako koloru tekstu
        style = style.fg(Color::Rgb(
            syntect_style.foreground.r,
            syntect_style.foreground.g,
            syntect_style.foreground.b,
        ));
    }

    if syntect_style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        style = style.add_modifier(Modifier::BOLD);
    }
    if syntect_style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if syntect_style
        .font_style
        .contains(syntect::highlighting::FontStyle::UNDERLINE)
    {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

pub fn highlight_code<'a>(
    code: String,
    extension: &str,
) -> Result<Vec<Line<'a>>, Box<dyn std::error::Error>> {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let theme = &THEME;

    let mut h = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line_str in LinesWithEndings::from(code.as_str()) {
        let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line_str, &SYNTAX_SET)?;

        let spans: Vec<Span> = ranges
            .into_iter()
            .map(|(syntect_style, segment)| {
                let ratatui_style = syntect_style_to_ratatui(syntect_style);
                Span::styled(segment.to_string(), ratatui_style)
            })
            .collect();

        lines.push(Line::from(spans));
    }

    Ok(lines)
}
