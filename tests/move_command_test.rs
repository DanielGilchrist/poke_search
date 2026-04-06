mod utils;

use poke_search::{
    client::MockClientImplementation, formatter::utils as fmt, name_matcher::matcher, run,
    type_badge,
};
use rustemon::static_resources;
use unicode_width::UnicodeWidthStr;
use utils::parse_args;

#[tokio::test]
async fn pokemon_move_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
    let incorrect_name = "kfdslskfls";

    let mock_client = MockClientImplementation::new();
    let cli = parse_args(vec!["move", incorrect_name]);
    let expected = matcher::build_unknown_name("move", incorrect_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_move_uncertain_suggestion() -> Result<(), Box<dyn std::error::Error>> {
    let correct_name = "flamethrower";
    let incorrect_name = "flaymthowaer";

    let mock_client = MockClientImplementation::new();
    let cli = parse_args(vec!["move", incorrect_name]);
    let expected = matcher::build_suggested_name("move", incorrect_name, correct_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_move_autocorrect_if_similar_enough() -> Result<(), Box<dyn std::error::Error>> {
    let correct_name = "fire-blast";
    let similar_name = "Fire Blast";

    let mut mock_client = MockClientImplementation::new();
    let mock_move = static_resources::get_move();

    mock_client
        .expect_fetch_move()
        .with(mockall::predicate::eq(correct_name))
        .once()
        .returning(move |_args| Ok(mock_move.clone()));

    let cli = parse_args(vec!["move", similar_name]);

    let fire = type_badge::fetch("fire");
    let expected = format!(
        "{}
  {}: Fire Blast
  {}: {fire}
  {}: Special
  {}: 110
  {}: 85
  {}: 5
  {}: 0
  {}: An attack that may cause a burn.
  {}: Has a 10% chance to burn the target.",
        fmt::white("Move"),
        fmt::white("Name"),
        fmt::white("Type"),
        fmt::white("Damage Type"),
        fmt::white("Power"),
        fmt::white("Accuracy"),
        fmt::white("PP"),
        fmt::white("Priority"),
        fmt::white("Description"),
        fmt::white("Effect")
    );

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn move_learned_by_types_are_aligned_with_names() -> Result<(), Box<dyn std::error::Error>> {
    let mock_move = static_resources::get_move();
    let pokemon_names: Vec<String> = mock_move
        .learned_by_pokemon
        .iter()
        .map(|p| p.name.clone())
        .collect();

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_move()
        .with(mockall::predicate::eq("fire-blast"))
        .once()
        .returning(move |_| Ok(static_resources::get_move()));

    mock_client
        .expect_fetch_pokemon()
        .times(pokemon_names.len())
        .returning(move |_| Ok(static_resources::get_pokemon()));

    let cli = parse_args(vec!["move", "Fire Blast", "--learned-by"]);
    let actual = run(&mock_client, cli).await.to_string();

    let pokemon = static_resources::get_pokemon();
    let name = "Charizard";
    let name_width = UnicodeWidthStr::width(name);

    let type_names: Vec<String> = pokemon.types.iter().map(|t| t.type_.name.clone()).collect();
    let type_width = UnicodeWidthStr::width(
        type_names
            .iter()
            .map(|t| type_badge::format_type_name(t))
            .collect::<Vec<_>>()
            .join(" | ")
            .as_str(),
    );

    let column_width = name_width.max(type_width) + 4;

    let type_badge = type_names
        .iter()
        .map(|t| type_badge::fetch(t))
        .collect::<Vec<_>>()
        .join(" | ");
    let name_padding = " ".repeat(column_width - name_width);
    let type_padding = " ".repeat(column_width.saturating_sub(type_width));

    let expected_name_row = format!("  {name}{name_padding}  {name}{name_padding}");
    let expected_type_row = format!("  {type_badge}{type_padding}  {type_badge}{type_padding}");

    assert_contains!(actual, &expected_name_row);
    assert_contains!(actual, &expected_type_row);

    Ok(())
}
