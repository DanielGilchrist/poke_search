mod utils;

use poke_search::{client::MockClientImplementation, name_matcher::matcher, run};
use rustemon::static_resources;
use utils::parse_args;

#[tokio::test]
async fn pokemon_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
    let incorrect_name = "lkfdjslsdkjfkls";

    let mock_client = MockClientImplementation::new();
    let cli = parse_args(vec!["pokemon", incorrect_name]);
    let expected = matcher::build_unknown_name("pokemon", incorrect_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_autocorrect_if_similar_enough() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let similar_name = "Charzard";
    let correct_name = "charizard";

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_pokemon()
        .with(mockall::predicate::eq(correct_name))
        .once()
        .returning(move |_args| Ok(static_resources::get_pokemon()));

    mock_client
        .expect_fetch_pokemon_species()
        .with(mockall::predicate::eq(correct_name))
        .once()
        .returning(move |_args| Ok(static_resources::get_pokemon_species()));

    mock_client
        .expect_fetch_ability()
        .with(mockall::predicate::eq("blaze"))
        .once()
        .returning(move |_args| Ok(static_resources::get_ability()));

    mock_client
        .expect_fetch_ability()
        .with(mockall::predicate::eq("solar-power"))
        .once()
        .returning(move |_args| Ok(static_resources::get_ability()));

    let cli = parse_args(vec!["pokemon", similar_name]);

    let fire = poke_search::type_badge::fetch("fire");
    let flying = poke_search::type_badge::fetch("flying");

    let expected = format!(
        "Summary
  Name: Charizard
  Type: {fire} | {flying}
  Abilities: Blaze | Solar Power
  Generation: I

Stats
  HP: 78
  Attack: 84
  Defense: 78
  Special Attack: 109
  Special Defense: 85
  Speed: 100
  Total: 534

Abilities
  Name: Static
  Description: Has a 30% chance of paralyzing attacking Pokémon on contact.

  Name: Static
  Description: Has a 30% chance of paralyzing attacking Pokémon on contact."
    );

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_shows_evolution_information() -> Result<(), Box<dyn std::error::Error>> {
    let pokemon_name = "charizard";

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_pokemon()
        .with(mockall::predicate::eq(pokemon_name))
        .once()
        .returning(move |_args| Ok(static_resources::get_pokemon()));

    mock_client
        .expect_fetch_pokemon_species()
        .with(mockall::predicate::eq(pokemon_name))
        .once()
        .returning(move |_args| Ok(static_resources::get_pokemon_species()));

    mock_client
        .expect_fetch_ability()
        .with(mockall::predicate::eq("blaze"))
        .once()
        .returning(move |_args| Ok(static_resources::get_ability()));

    mock_client
        .expect_fetch_ability()
        .with(mockall::predicate::eq("solar-power"))
        .once()
        .returning(move |_args| Ok(static_resources::get_ability()));

    mock_client
        .expect_fetch_evolution_chain_from_url()
        .with(mockall::predicate::eq(
            "https://pokeapi.co/api/v2/evolution-chain/2/",
        ))
        .once()
        .returning(move |_args| Ok(static_resources::get_evolution_chain()));

    let cli = parse_args(vec!["pokemon", pokemon_name, "-e"]);

    let expected = r#"Evolution Chain:
  Stage 1: Charmander
  Stage 2: Charmeleon (Level Up - Level 16)
  Stage 3: Charizard (Level Up - Level 36)"#;

    let actual = run(&mock_client, cli).await.to_string();

    assert!(actual.contains(expected));

    Ok(())
}

#[tokio::test]
async fn pokemon_uncertain_suggestion() -> Result<(), Box<dyn std::error::Error>> {
    let correct_name = "pikachu";
    let incorrect_name = "peacachu";

    let mock_client = MockClientImplementation::new();
    let cli = parse_args(vec!["pokemon", incorrect_name]);
    let expected = matcher::build_suggested_name("pokemon", incorrect_name, correct_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}
