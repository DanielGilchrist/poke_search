mod utils;

use poke_search::{client::MockClientImplementation, run};
use rustemon::static_resources;
use utils::parse_args;

#[tokio::test]
async fn generation_basic_info_without_flags() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i"]);

    let actual = run(&mock_client, cli).await.to_string();
    let expected = r#"Generation
  Name: Generation I
  Main Region: Kanto
  Pokemon: 151
  Moves: 165
  Abilities: 0"#;

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn generation_with_pokemon_flag() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--pokemon"]);

    let actual = run(&mock_client, cli).await.to_string();

    assert!(actual.contains("Pokemon (151)"));
    assert!(actual.contains("Bulbasaur"));
    assert!(actual.contains("Pikachu"));
    assert!(actual.contains("Charizard"));

    Ok(())
}

#[tokio::test]
async fn generation_with_moves_flag() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--moves"]);

    let actual = run(&mock_client, cli).await.to_string();

    assert!(actual.contains("Moves (165)"));
    assert!(actual.contains("Tackle"));
    assert!(actual.contains("Thunderbolt"));

    Ok(())
}

#[tokio::test]
async fn generation_with_all_flags() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--pokemon", "--moves"]);

    let actual = run(&mock_client, cli).await.to_string();

    // TODO: Figure out how to mock NamedAPIResource<Ability> so we can test abilities too
    assert!(actual.contains("Pokemon (151)"));
    assert!(actual.contains("Bulbasaur"));
    assert!(actual.contains("Moves (165)"));
    assert!(actual.contains("Tackle"));

    Ok(())
}

#[tokio::test]
async fn generation_displays_in_columns() -> Result<(), Box<dyn std::error::Error>> {
    colored::control::set_override(false);

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--pokemon"]);

    let actual = run(&mock_client, cli).await.to_string();

    let pokemon_section = actual.split("Pokemon (151)").nth(1).unwrap();
    let first_line = pokemon_section.lines().nth(1).unwrap();

    let pokemon_count = first_line.split_whitespace().count();
    assert_eq!(pokemon_count, 4, "Should display 4 Pokemon per line");

    Ok(())
}
