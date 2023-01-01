mod builder;
mod client;
mod formatter;
mod name_matcher;
mod type_colours;

use crate::{
    builder::Builder,
    client::{Client, ClientImplementation},
};

use clap::{arg, ArgMatches, Command};

mod commands;
use commands::{
    move_command::MoveCommand, moves_command::MovesCommand, pokemon_command::PokemonCommand,
    type_command::TypeCommand,
};

#[tokio::main]
async fn main() {
    let client = Client::default();
    let matches = parse_commands().get_matches();

    run(&client, matches).await.print();
}

fn parse_name(name: Option<&String>) -> Option<String> {
    name.map(|n| n.to_lowercase().split(' ').collect::<Vec<_>>().join("-"))
}

fn get_optional_string(command: &str, sub_matches: &ArgMatches) -> Option<String> {
    parse_name(sub_matches.get_one::<String>(command))
}

fn get_required_string(command: &str, sub_matches: &ArgMatches) -> String {
    get_optional_string(command, sub_matches).unwrap()
}

fn get_optional_bool(command: &str, sub_matches: &ArgMatches) -> bool {
    sub_matches
        .get_one::<bool>(command)
        .unwrap_or(&false)
        .to_owned()
}

fn parse_commands() -> Command {
    Command::new("poke_search_cli")
        .about("Search for pokemon information from the command line")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            parse_moves_command(),
            parse_move_command(),
            parse_pokemon_command(),
            parse_type_command(),
        ])
}

fn parse_moves_command() -> Command {
    Command::new("moves")
        .about("See moves for a pokemon")
        .args([
            arg!(-p --pokemon <POKEMON_NAME> "The name of the pokemon you want to see moves for")
                .required(true),
            arg!(-t --type_name <TYPE_NAME> "The type of moves you want to see").required(false),
            arg!(-c --category <CATEGORY> "Only show moves for a specific category")
                .required(false),
        ])
        .arg_required_else_help(true)
}

fn parse_move_command() -> Command {
    Command::new("move")
        .about("See information about a move")
        .args([
            arg!(move: <MOVE_NAME>).required(true),
            arg!(-l --learned_by "Include a list of pokemon that learn the move").required(false),
        ])
        .arg_required_else_help(true)
}

fn parse_pokemon_command() -> Command {
    Command::new("pokemon")
        .about("See information about a pokemon")
        .args([arg!(pokemon: <POKEMON_NAME>).required(true)])
}

fn parse_type_command() -> Command {
    Command::new("type")
        .about("See information about a specific type")
        .args([
            arg!(type_name: <TYPE_NAME>).required(true),
            arg!(-s --second_type_name <SECOND_TYPE_NAME>).required(false),
        ])
}

async fn run(client: &dyn ClientImplementation, matches: ArgMatches) -> Builder {
    match matches.subcommand() {
        Some(("moves", sub_matches)) => {
            let pokemon_name = get_required_string("pokemon", sub_matches);
            let type_name = get_optional_string("type_name", sub_matches);
            let category = get_optional_string("category", sub_matches);

            MovesCommand::execute(client, pokemon_name, type_name, category).await
        }

        Some(("move", sub_matches)) => {
            let move_name = get_required_string("move", sub_matches);
            let include_learned_by = get_optional_bool("learned_by", sub_matches);

            MoveCommand::execute(client, move_name, include_learned_by).await
        }

        Some(("pokemon", sub_matches)) => {
            let pokemon_name = get_required_string("pokemon", sub_matches);

            PokemonCommand::execute(client, pokemon_name).await
        }

        Some(("type", sub_matches)) => {
            let type_name = get_required_string("type_name", sub_matches);
            let second_type_name = get_optional_string("second_type_name", sub_matches);

            TypeCommand::execute(client, type_name, second_type_name).await
        }

        _ => Builder::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockClientImplementation;

    use rustemon::{error::Error, model::pokemon::Type};

    #[tokio::test]
    async fn pokemon_move_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
        let incorrect_name = "flamhrower";

        let mut mock_client = MockClientImplementation::new();
        mock_client
            .expect_fetch_move()
            .with(mockall::predicate::eq(incorrect_name))
            .once()
            .returning(|_args| Err(Error::FollowEmptyURL));

        let command = parse_commands().no_binary_name(true);
        let matches = command.get_matches_from(vec!["move", incorrect_name]);

        let expected = build_suggestion("move", incorrect_name, "flamethrower");
        let actual = run(&mock_client, matches).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

    #[tokio::test]
    async fn pokemon_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
        let incorrect_name = "pikchuy";

        let mut mock_client = MockClientImplementation::new();
        mock_client
            .expect_fetch_pokemon()
            .with(mockall::predicate::eq(incorrect_name))
            .once()
            .returning(|_args| Err(Error::FollowEmptyURL));

        let command = parse_commands().no_binary_name(true);
        let matches = command.get_matches_from(vec!["pokemon", incorrect_name]);

        let expected = build_suggestion("pokemon", incorrect_name, "pikachu");
        let actual = run(&mock_client, matches).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

    #[tokio::test]
    async fn pokemon_single_type_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
        let incorrect_name = "drraggon";

        let mut mock_client = MockClientImplementation::new();
        mock_client
            .expect_fetch_type()
            .with(mockall::predicate::eq(incorrect_name))
            .once()
            .returning(|_args| Err(Error::FollowEmptyURL));

        let command = parse_commands().no_binary_name(true);
        let matches = command.get_matches_from(vec!["type", incorrect_name]);

        let expected = build_suggestion("type", incorrect_name, "dragon");
        let actual = run(&mock_client, matches).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

    #[tokio::test]
    async fn pokemon_dual_type_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
        let correct_name = "water";
        let incorrect_name = "pschi";

        let mut mock_client = MockClientImplementation::new();

        mock_client
            .expect_fetch_type()
            .with(mockall::predicate::eq(correct_name))
            .once()
            .returning(|_args| Ok(Type::default()));

        mock_client
            .expect_fetch_type()
            .with(mockall::predicate::eq(incorrect_name))
            .once()
            .returning(|_args| Err(Error::FollowEmptyURL));

        let command = parse_commands().no_binary_name(true);
        let matches = command.get_matches_from(vec!["type", correct_name, "-s", incorrect_name]);

        let expected = build_suggestion("type", incorrect_name, "psychic");
        let actual = run(&mock_client, matches).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

    fn build_suggestion(keyword: &str, name: &str, correct_name: &str) -> String {
        format!(
            "Unknown {} \"{}\"\nDid you mean \"{}\"?",
            keyword, name, correct_name
        )
    }
}
