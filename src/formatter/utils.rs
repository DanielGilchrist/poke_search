use colored::{ColoredString, Colorize};
use rustemon::model::resource::VerboseEffect;

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

pub(crate) fn formatln(title: &str, value: &str) -> String {
    format!("  {}{}{}\n", title, ": ", capitalise(value))
}

pub(crate) fn extract_effect(effect_entries: &[VerboseEffect]) -> Option<String> {
    let effect = effect_entries.iter().find_map(|verbose_effect| {
        if verbose_effect.language.name == "en" {
            Some(&verbose_effect.effect)
        } else {
            None
        }
    })?;

    Some(effect.replace('\n', " "))
}

pub(crate) fn parse_maybe_i64(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => String::from("-"),
    }
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
