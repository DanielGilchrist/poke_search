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

struct Source<'a> {
    url: String,
    file_name: &'a str,
    index: usize,
}

impl<'a> Source<'a> {
    pub fn new(url_file_name: &str, file_name: &'a str) -> Self {
        Self {
            url: Self::build_url(url_file_name),
            file_name,
            index: 1,
        }
    }

    fn build_url(url_file_name: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/{url_file_name}"
        )
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }
}

fn main() {
    let sources = vec![
        Source::new("pokemon.csv", "pokemon_names"),
        Source::new("moves.csv", "move_names"),
        Source::new("types.csv", "type_names"),
        Source::new("move_damage_classes.csv", "move_damage_class_names"),
        Source::new("abilities.csv", "ability_names"),
        Source::new("items.csv", "item_names"),
        Source::new("generations.csv", "generation_names").index(2),
    ];

    for source in sources.into_iter() {
        match fetch_and_replace(source) {
            Ok(_) => (),
            Err(error) => eprintln!("{:?}", error),
        };
    }
}

fn fetch_and_replace(source: Source) -> Result<(), Box<dyn std::error::Error>> {
    let Source {
        url,
        file_name,
        index,
    } = source;

    let response = reqwest::blocking::get(url)?;
    let csv = response.text()?;
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut names = reader
        .records()
        .map(|record| record.unwrap()[index].to_string())
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
