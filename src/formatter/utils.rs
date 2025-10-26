use colored::{ColoredString, Colorize};
use rustemon::model::resource::VerboseEffect;
use unicode_width::UnicodeWidthStr;

pub(crate) fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn split_and_capitalise(s: &str) -> String {
    s.split('-').map(capitalise).collect::<Vec<_>>().join(" ")
}

pub(crate) fn format_columns(items: &[String], num_columns: usize) -> String {
    if items.is_empty() {
        return String::new();
    }

    let max_width = items
        .iter()
        .map(|item| UnicodeWidthStr::width(item.as_str()))
        .max()
        .unwrap_or(0);

    let column_width = max_width + 2;

    let mut output = String::new();
    for (i, item) in items.iter().enumerate() {
        let width = UnicodeWidthStr::width(item.as_str());
        let padding = column_width - width;

        output.push_str("  ");
        output.push_str(item);
        output.push_str(&" ".repeat(padding));

        if (i + 1) % num_columns == 0 {
            output.push('\n');
        }
    }

    if !items.len().is_multiple_of(num_columns) {
        output.push('\n');
    }

    output
}

pub(crate) fn formatln(title: &str, value: &str) -> String {
    format!("  {}{}{}\n", title, ": ", capitalise(value))
}

pub(crate) fn extract_effect(effect_entries: &[VerboseEffect]) -> Option<String> {
    let formatted_effect = effect_entries.iter().find_map(|verbose_effect| {
        if verbose_effect.language.name == "en" {
            let effect = &verbose_effect.short_effect;
            Some(clean_and_wrap_text(effect, 4, 80))
        } else {
            None
        }
    })?;

    Some(formatted_effect)
}

pub(crate) fn parse_maybe_i64(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => String::from("-"),
    }
}

pub(crate) fn clean_and_wrap_text(text: &str, indent: usize, width: usize) -> String {
    let cleaned_text = text
        .replace("\n:", ":")
        .replace(":  ", ": ")
        .replace("  ", " ")
        .replace("\n  ", "\n")
        .trim()
        .to_owned();

    let spaces = &" ".repeat(indent);
    let options = textwrap::Options::new(width).subsequent_indent(spaces);

    textwrap::fill(&cleaned_text, options)
}

// Colours
pub(crate) fn white(str: &str) -> String {
    format_colour(str.white())
}

pub(crate) fn green(str: &str) -> String {
    format_colour(str.green())
}

pub(crate) fn yellow(str: &str) -> String {
    format_colour(str.yellow())
}

pub(crate) fn red(str: &str) -> String {
    format_colour(str.red())
}

pub(crate) fn bright_red(str: &str) -> String {
    format_colour(str.bright_red())
}

pub(crate) fn bright_green(str: &str) -> String {
    format_colour(str.bright_green())
}

fn format_colour(coloured_string: ColoredString) -> String {
    format!("{coloured_string}")
}
