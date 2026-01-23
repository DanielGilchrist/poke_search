mod utils;

use poke_search::{
    client::MockClientImplementation, formatter::utils as fmt, name_matcher::matcher, run,
    type_badge,
};
use rustemon::static_resources;
use utils::parse_args;

#[tokio::test]
async fn pokemon_single_type_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
    let incorrect_name = "lkjfsldfjsdkll";

    let mock_client = MockClientImplementation::new();
    let cli = parse_args(vec!["type", incorrect_name]);
    let expected = matcher::build_unknown_name("type", incorrect_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_single_type_autocorrect_if_similar_enough()
-> Result<(), Box<dyn std::error::Error>> {
    let almost_correct_name = "firre";
    let correct_name = "fire";

    let mut mock_client = MockClientImplementation::new();
    let mock_type = static_resources::get_type();

    mock_client
        .expect_fetch_type()
        .with(mockall::predicate::eq(correct_name))
        .once()
        .returning(move |_args| Ok(mock_type.clone()));

    let cli = parse_args(vec!["type", almost_correct_name]);

    let fire = type_badge::fetch("fire");
    let dragon = type_badge::fetch("dragon");
    let rock = type_badge::fetch("rock");
    let water = type_badge::fetch("water");
    let dark = type_badge::fetch("dark");
    let electr = type_badge::fetch("electric");
    let fairy = type_badge::fetch("fairy");
    let fight = type_badge::fetch("fighting");
    let flying = type_badge::fetch("flying");
    let ghost = type_badge::fetch("ghost");
    let ground = type_badge::fetch("ground");
    let normal = type_badge::fetch("normal");
    let poison = type_badge::fetch("poison");
    let psychc = type_badge::fetch("psychic");
    let bug = type_badge::fetch("bug");
    let grass = type_badge::fetch("grass");
    let ice = type_badge::fetch("ice");
    let steel = type_badge::fetch("steel");

    let expected = format!(
        "{fire}

{}
{}  {dragon} | {fire} | {rock} | {water}
{}  {dark} | {electr} | {fairy} | {fight} | {flying} | {ghost} | {ground} | {normal} | {poison} | {psychc} | stellar
{}  {bug} | {grass} | {ice} | {steel}

{}
{}  {bug} | {fairy} | {fire} | {grass} | {ice} | {steel}
{}  {dark} | {dragon} | {electr} | {fight} | {flying} | {ghost} | {normal} | {poison} | {psychc} | stellar
{}  {ground} | {rock} | {water}",
        fmt::white("Offence"),
        fmt::bright_red("0.5x\n"),
        fmt::yellow("1x\n"),
        fmt::green("2x\n"),
        fmt::white("Defence"),
        fmt::bright_green("0.5x\n"),
        fmt::yellow("1x\n"),
        fmt::red("2x\n")
    );

    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_single_type_uncertain_suggestion() -> Result<(), Box<dyn std::error::Error>> {
    let correct_name = "dragon";
    let incorrect_name = "drahgna";

    let mock_client = MockClientImplementation::new();
    let cli = parse_args(vec!["type", incorrect_name]);
    let expected = matcher::build_suggested_name("type", incorrect_name, correct_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_dual_type_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
    let correct_name = "water";
    let incorrect_name = "ljflkdsfjslkj";

    let mut mock_client = MockClientImplementation::new();
    let mut mock_type = static_resources::get_type();
    mock_type.name = String::from(correct_name);

    mock_client.expect_fetch_type().never();

    let cli = parse_args(vec!["type", correct_name, "-s", incorrect_name]);
    let expected = matcher::build_unknown_name("type", incorrect_name);
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn pokemon_dual_type_uncertain_suggestion() -> Result<(), Box<dyn std::error::Error>> {
    let correct_name = "water";
    let incorrect_name = "sychick";

    let mut mock_client = MockClientImplementation::new();

    mock_client.expect_fetch_type().never();

    let cli = parse_args(vec!["type", correct_name, "-s", incorrect_name]);
    let expected = matcher::build_suggested_name("type", incorrect_name, "psychic");
    let actual = run(&mock_client, cli).await.to_string();

    assert_eq!(expected, actual);

    Ok(())
}

#[tokio::test]
async fn dual_type_same_type_only_shows_once() -> Result<(), Box<dyn std::error::Error>> {
    let type_name = "fire";

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_type()
        .with(mockall::predicate::eq(type_name))
        .once()
        .returning(|_args| Ok(static_resources::get_type()));

    let cli = parse_args(vec!["type", type_name, "-s", type_name]);
    let output = run(&mock_client, cli).await.to_string();
    let first_line = output.lines().next().unwrap();

    assert_eq!(type_badge::fetch(type_name), first_line);

    Ok(())
}

#[tokio::test]
async fn dual_type_same_type_only_shows_once_corrected() -> Result<(), Box<dyn std::error::Error>> {
    let type_name = "fire";
    let second_type_name = "fires";

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_type()
        .with(mockall::predicate::eq(type_name))
        .once()
        .returning(|_args| Ok(static_resources::get_type()));

    let cli = parse_args(vec!["type", type_name, "-s", second_type_name]);
    let output = run(&mock_client, cli).await.to_string();
    let first_line = output.lines().next().unwrap();

    assert_eq!(type_badge::fetch(type_name), first_line);

    Ok(())
}

#[tokio::test]
async fn lists_pokemon_in_columns() -> Result<(), Box<dyn std::error::Error>> {
    let name = "fire";

    let mut mock_client = MockClientImplementation::new();

    mock_client
        .expect_fetch_type()
        .with(mockall::predicate::eq(name))
        .once()
        .returning(|_args| Ok(static_resources::get_type()));

    let cli = parse_args(vec!["type", name, "-p"]);
    let actual = run(&mock_client, cli).await.to_string();

    let pokemon_section = actual.split("Pokemon (103)").nth(1).unwrap();
    let first_line = pokemon_section.lines().nth(1).unwrap();
    assert!(first_line.contains("Arcanine"));
    assert!(first_line.contains("Arcanine Hisui"));
    assert!(first_line.contains("Armarouge"));
    assert!(!first_line.contains("Blacephalon"));

    let second_line = pokemon_section.lines().nth(2).unwrap();
    assert!(second_line.contains("Blacephalon"));

    Ok(())
}
