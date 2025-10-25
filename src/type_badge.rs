use colored::Colorize;

use std::{collections::HashMap, sync::LazyLock};

#[allow(clippy::upper_case_acronyms)]
type RGB = (u8, u8, u8);

static TYPE_NAME_TO_RGB: LazyLock<HashMap<&'static str, RGB>> = LazyLock::new(|| {
    HashMap::from([
        ("bug", (166, 185, 26)),
        ("dark", (112, 87, 70)),
        ("dragon", (111, 53, 252)),
        ("electric", (247, 208, 44)),
        ("fairy", (214, 133, 173)),
        ("fighting", (194, 46, 40)),
        ("fire", (238, 129, 48)),
        ("flying", (169, 143, 243)),
        ("ghost", (115, 87, 151)),
        ("grass", (122, 199, 76)),
        ("ground", (226, 191, 101)),
        ("ice", (150, 217, 214)),
        ("normal", (168, 167, 122)),
        ("poison", (163, 62, 161)),
        ("psychic", (249, 85, 135)),
        ("rock", (182, 161, 54)),
        ("steel", (183, 183, 206)),
        ("water", (99, 144, 240)),
    ])
});

static MAX_TYPE_LENGTH: LazyLock<usize> = LazyLock::new(|| {
    TYPE_NAME_TO_RGB
        .keys()
        .map(|name| shorten_type_name(name).len())
        .max()
        .unwrap_or(0)
});

static THIN_SPACE: &str = "\u{2009}";

pub fn fetch(type_name: &str) -> String {
    match TYPE_NAME_TO_RGB.get(type_name) {
        Some(&(r, g, b)) => {
            let formatted_type_name = format_type_name(type_name);
            let display_type_name = formatted_type_name
                .truecolor(255, 255, 255)
                .bold()
                .on_truecolor(r, g, b);

            format!("{display_type_name}")
        }
        None => type_name.to_owned(),
    }
}

fn shorten_type_name(type_name: &str) -> String {
    match type_name {
        "electric" => "ELECTR".to_string(),
        "fighting" => "FIGHT".to_string(),
        "psychic" => "PSYCHC".to_string(),
        _ => type_name.to_uppercase(),
    }
}

fn format_type_name(type_name: &str) -> String {
    let shortened = shorten_type_name(type_name);
    let max_length = *MAX_TYPE_LENGTH;

    if shortened.len() < max_length {
        pad(shortened, THIN_SPACE, max_length)
    } else {
        shortened
    }
}

fn pad(word: String, pad_with: &str, max_length: usize) -> String {
    let total_padding = max_length - word.len();
    let padding = pad_with.repeat(total_padding / 2);

    format!("{padding}{word}{padding}")
}
