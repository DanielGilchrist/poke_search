mod utils;

use poke_search::{client::MockClientImplementation, formatter::utils as fmt, run};
use rustemon::static_resources;
use utils::parse_args;

#[tokio::test]
async fn generation_basic_info_without_flags() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i"]);

    let actual = run(&mock_client, cli).await.to_string();
    let expected = format!(
        "{}
  {}: Generation I
  {}: Kanto
  {}: 151
  {}: 165
  {}: 0",
        fmt::white("Generation"),
        fmt::white("Name"),
        fmt::white("Main Region"),
        fmt::white("Pokemon"),
        fmt::white("Moves"),
        fmt::white("Abilities")
    );

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn generation_with_pokemon_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--pokemon"]);

    let actual = run(&mock_client, cli).await.to_string();

    assert_contains!(actual, "Pokemon (151)");
    assert_contains!(actual, "Bulbasaur");
    assert_contains!(actual, "Pikachu");
    assert_contains!(actual, "Charizard");

    Ok(())
}

#[tokio::test]
async fn generation_with_moves_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--moves"]);

    let actual = run(&mock_client, cli).await.to_string();

    assert_contains!(actual, "Moves (165)");
    assert_contains!(actual, "Tackle");
    assert_contains!(actual, "Thunderbolt");

    Ok(())
}

#[tokio::test]
async fn generation_with_all_flags() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_generation()
        .with(mockall::predicate::eq("generation-i"))
        .once()
        .returning(move |_args| Ok(static_resources::get_generation()));

    let cli = parse_args(vec!["generation", "i", "--pokemon", "--moves"]);

    let actual = run(&mock_client, cli).await.to_string();

    // TODO: Figure out how to mock NamedAPIResource<Ability> so we can test abilities too
    assert_contains!(actual, "Pokemon (151)");
    assert_contains!(actual, "Bulbasaur");
    assert_contains!(actual, "Moves (165)");
    assert_contains!(actual, "Tackle");

    Ok(())
}

#[tokio::test]
async fn generation_displays_in_columns() -> Result<(), Box<dyn std::error::Error>> {
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
