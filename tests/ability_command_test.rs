mod utils;

use poke_search::{client::MockClientImplementation, run};
use rustemon::static_resources;
use utils::parse_args;

#[tokio::test]
async fn ability_default_description() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_ability()
        .with(mockall::predicate::eq("static"))
        .once()
        .returning(move |_args| Ok(static_resources::get_ability()));

    let cli = parse_args(vec!["ability", "static"]);

    let expected = "Ability
  Name: Static
  Description: Has a 30% chance of paralyzing attacking Pokémon on contact.";

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn ability_verbose_description() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_ability()
        .with(mockall::predicate::eq("static"))
        .once()
        .returning(move |_args| Ok(static_resources::get_ability()));

    let cli = parse_args(vec!["ability", "static", "-v"]);

    let expected = "Ability
  Name: Static
  Description: Whenever a move makes contact with this Pokémon, the move's user has a 30%
    chance of being paralyzed.

    Pokémon that are immune to electric-type moves can still be paralyzed by this
    ability.

    Overworld: If the lead Pokémon has this ability, there is a 50% chance that
    encounters will be with an electric Pokémon, if applicable.";

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}
