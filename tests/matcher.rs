// use poke_search_cli::client::{Client, ClientImplementation};

use assert_cmd::{cargo::CargoError, prelude::*};
use predicates::prelude::*;
use poke_search_cli::client::MockClientImplementation;

use std::process::Command;

use rustemon::error::Error;

#[test]
fn pokemon_move_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
    let incorrect_name = "flamhrower";

    let mut mock_implementation = MockClientImplementation::new();
    mock_implementation.expect_fetch_move()
        .with(mockall::predicate::eq(incorrect_name))
        .once()
        .returning(|_x| {
          println!("It works!!!!");
          Err(Error::FollowEmptyURL)
        });

    mock_implementation.checkpoint();

    let mut cmd = build_command()?;
    let incorrect_name = "flamhrower";

    let expected = build_suggestion("move", incorrect_name, "flamethrower");
    assert_failure(cmd.arg("move").arg(incorrect_name), expected);

    Ok(())
}

// #[test]
// fn pokemon_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
//   let mut cmd = build_command()?;
//   let incorrect_name = "pikchuy";

//   let expected = build_suggestion("pokemon", incorrect_name, "pikachu");
//   assert_failure(cmd.arg("pokemon").arg(incorrect_name), expected);

//   Ok(())
// }

// #[test]
// fn pokemon_single_type_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
//   let mut cmd = build_command()?;
//   let incorrect_name = "drraggon";

//   let expected = build_suggestion("type", incorrect_name, "dragon");
//   assert_failure(cmd.arg("type").arg(incorrect_name), expected);

//   Ok(())
// }

// #[test]
// fn pokemon_dual_type_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
//   let mut cmd = build_command()?;
//   let incorrect_name = "pschi";

//   let expected = build_suggestion("type", incorrect_name, "psychic");
//   assert_failure(cmd.arg("type").arg("water").arg("-s").arg(incorrect_name), expected);

//   Ok(())
// }

fn build_suggestion(keyword: &str, name: &str, correct_name: &str) -> String {
    format!(
        "Unknown {} \"{}\"\nDid you mean \"{}\"?",
        keyword, name, correct_name
    )
}

fn build_command() -> Result<Command, CargoError> {
    Command::cargo_bin("poke_search_cli")
}

fn assert_failure(cmd: &mut Command, expected: String) {
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains(expected));
}
