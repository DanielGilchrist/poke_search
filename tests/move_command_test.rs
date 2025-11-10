mod utils;

use poke_search::{
    client::MockClientImplementation, formatter::utils as fmt, name_matcher::matcher, run,
    type_badge,
};
use rustemon::static_resources;
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
