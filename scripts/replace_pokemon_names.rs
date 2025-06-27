#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! csv = "1.3"
//! relative-path = "1.9"
//! reqwest = { version = "0.11", features = ["blocking"] }
//! ```

use relative_path::RelativePath;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const SOURCES: &'static [(&'static str, &'static str)] = &[
    (
        "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/pokemon.csv",
        "pokemon_names",
    ),
    (
        "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/moves.csv",
        "move_names",
    ),
    (
        "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/types.csv",
        "type_names",
    ),
    (
        "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/move_damage_classes.csv",
        "move_damage_class_names",
    ),
    (
        "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/abilities.csv",
        "ability_names",
    ),
    (
        "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/items.csv",
        "item_names",
    ),
];

fn main() {
    for (url, file_name) in SOURCES.into_iter() {
        match fetch_and_replace(url, file_name) {
            Ok(_) => (),
            Err(error) => eprintln!("{:?}", error),
        };
    }
}

fn fetch_and_replace(url: &str, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let csv = response.text()?;
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut names = reader
        .records()
        .map(|record| record.unwrap()[1].to_string())
        .collect::<Vec<_>>();

    // This is important as we rely on binary search if we need to find a name in the vector in name_matcher::matcher.
    names.sort();

    let joined_names = names
        .into_iter()
        .map(|name| format!("        String::from(\"{name}\"),"))
        .collect::<Vec<_>>()
        .join("\n");

    let file_name_constant_string = file_name.to_string().to_uppercase();
    let file_contents = format!(
        "use std::sync::LazyLock;

pub static {file_name_constant_string}: LazyLock<Vec<String>> = LazyLock::new(|| {{
    vec![
{joined_names}
    ]
}});
"
    );

    write_contents(file_name, &file_contents)?;

    Ok(())
}

fn write_contents(file_name: &str, file_contents: &str) -> Result<(), std::io::Error> {
    let path = determine_file_path(file_name)?;
    let mut output = File::create(path.clone())?;

    write!(output, "{}", file_contents)?;
    println!(
        "Successfully saved {} to {}",
        file_name.replace('_', " "),
        path.display()
    );

    Ok(())
}

fn determine_file_path(file_name: &str) -> Result<PathBuf, std::io::Error> {
    let root = current_dir()?;
    let formatted_path = format!("src/name_matcher/{file_name}.rs");
    let relative_path = RelativePath::new(&formatted_path);

    Ok(relative_path.to_path(&root))
}
