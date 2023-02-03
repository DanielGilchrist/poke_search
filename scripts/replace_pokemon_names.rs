#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! csv = "1.1"
//! relative-path = "1.7.3"
//! reqwest = { version = "0.11", features = ["blocking"] }
//! ```

use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use relative_path::RelativePath;

const SOURCES: &'static [(&'static str, &'static str)] = &[
  (
    "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/pokemon.csv",
    "pokemon_names"
  ),
  (
    "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/moves.csv",
    "move_names"
  )
];

fn main() {
  SOURCES
    .into_iter()
    .for_each(|(url, file_name)| {
      match fetch_and_replace(url, file_name) {
        Ok(_) => (),
        Err(error) => eprintln!("{:?}", error)
      };
    })
}

fn fetch_and_replace(url: &str, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let response = reqwest::blocking::get(url)?;
  let csv = response.text()?;
  let mut reader = csv::Reader::from_reader(csv.as_bytes());
  let mut names = reader.records().map(|record| record.unwrap()[1].to_string()).collect::<Vec<_>>();

  names.sort();

  let joined_names = names
    .into_iter()
    .map(|name| format!("        String::from(\"{name}\"),"))
    .collect::<Vec<_>>()
    .join("\n");

  let file_name_constant_string = file_name.to_string().to_uppercase();
  let file_contents = format!("use once_cell::sync::Lazy;

pub static {file_name_constant_string}: Lazy<Vec<String>> = Lazy::new(|| {{
    vec![
{joined_names}
    ]
}});
");

  write_contents(file_name, &file_contents)?;

  Ok(())
}

fn write_contents(file_name: &str, file_contents: &str) -> Result<(), std::io::Error> {
  let path = determine_file_path(file_name)?;
  let mut output = File::create(path.clone())?;

  write!(output, "{}", file_contents)?;
  println!("Successfully saved {} to {}", file_name.replace('_', " "), path.display());

  Ok(())
}

fn determine_file_path(file_name: &str) -> Result<PathBuf, std::io::Error> {
  let root = current_dir()?;
  let formatted_path = format!("src/name_matcher/{file_name}.rs");
  let relative_path = RelativePath::new(&formatted_path);

  Ok(relative_path.to_path(&root))
}
