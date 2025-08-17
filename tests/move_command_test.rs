mod utils;

use poke_search::{client::MockClientImplementation, name_matcher::matcher, run};
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
    colored::control::set_override(false);

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

    let expected = r#"Move
  Name: Fire Blast
  Type: Fire
  Damage Type: Special
  Power: 110
  Accuracy: 85
  PP: 5
  Priority: 0
  Description: An attack that may cause a burn.
  Effect: Has a 10% chance to burn the target.
"#;

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}
