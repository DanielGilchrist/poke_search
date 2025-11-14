mod utils;

use poke_search::{client::MockClientImplementation, formatter::utils as fmt, run};
use rustemon::static_resources;
use utils::parse_args;

#[tokio::test]
async fn item_default_description() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_item()
        .with(mockall::predicate::eq("master-ball"))
        .once()
        .returning(move |_args| Ok(static_resources::get_item()));

    let cli = parse_args(vec!["item", "master-ball"]);

    let expected = format!(
        "{}
  {}: Master Ball
  {}: Standard Balls
  {}: Catches a wild Pokémon every time.",
        fmt::white("Item"),
        fmt::white("Name"),
        fmt::white("Category"),
        fmt::white("Effect"),
    );

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn item_verbose_description() -> Result<(), Box<dyn std::error::Error>> {
    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_item()
        .with(mockall::predicate::eq("master-ball"))
        .once()
        .returning(move |_args| Ok(static_resources::get_item()));

    let cli = parse_args(vec!["item", "master-ball", "--verbose"]);

    let expected = format!(
        "{}
  {}: Master Ball
  {}: Standard Balls
  {}: Used in battle: Catches a wild Pokémon without fail.

    If used in a trainer battle, nothing happens and the ball is lost.",
        fmt::white("Item"),
        fmt::white("Name"),
        fmt::white("Category"),
        fmt::white("Effect"),
    );

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}
