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

const URL_AND_FILENAME_PAIRS: &'static [(&'static str, &'static str)] = &[
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
  URL_AND_FILENAME_PAIRS
    .into_iter()
    .for_each(|(url, file_name)| {
      fetch_and_replace(url, file_name)
    })
}

fn fetch_and_replace(url: &str, file_name: &str) {
  let csv = reqwest::blocking::get(url)
    .unwrap()
    .text()
    .unwrap();

  let mut reader = csv::Reader::from_reader(csv.as_bytes());
  let mut names = reader
    .records()
    .map(|record| record.unwrap()[1].to_string())
    .collect::<Vec<_>>();

  names.sort();

  let joined_names = names
    .into_iter()
    .map(|name| {
      format!("        String::from(\"{name}\"),")
    })
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

  let path = determine_file_path(file_name);
  let mut output = File::create(path.clone()).unwrap();
  write!(output, "{}", file_contents).unwrap();

  println!("Successfully saved {} to {}", file_name.replace('_', " "), path.display());
}

fn determine_file_path(file_name: &str) -> PathBuf {
  let root = current_dir().unwrap();
  let formatted_path = format!("src/name_matcher/{file_name}.rs");
  let relative_path = RelativePath::new(&formatted_path);

  relative_path.to_path(&root)
}
