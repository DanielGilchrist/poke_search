mod formatter;
mod name_matcher;
mod type_colours;

use clap::{arg, ArgMatches, Command};

mod commands;
use commands::{
    move_command::MoveCommand, moves_command::MovesCommand, pokemon_command::PokemonCommand,
    type_command::TypeCommand,
};

#[tokio::main]
async fn main() {
    let client = rustemon::client::RustemonClient::default();

    match parse_commands().get_matches().subcommand() {
        Some(("moves", sub_matches)) => {
            let pokemon_name = get_required_string("pokemon", sub_matches);
            let type_name = get_optional_string("type_name", sub_matches);
            let category = get_optional_string("category", sub_matches);
            let physical = get_bool("physical", sub_matches);
            let special = get_bool("special", sub_matches);

            MovesCommand::execute(client, pokemon_name, type_name, category, phyisical, special).await;
        }

        Some(("move", sub_matches)) => {
            let move_name = get_required_string("move", sub_matches);
            let include_learned_by = get_bool("learned_by", sub_matches);

            MoveCommand::execute(client, move_name, include_learned_by).await;
        }

        Some(("pokemon", sub_matches)) => {
            let pokemon_name = get_required_string("pokemon", sub_matches);

            PokemonCommand::execute(client, pokemon_name).await;
        }

        Some(("type", sub_matches)) => {
            let type_name = get_required_string("type_name", sub_matches);
            let second_type_name = get_optional_string("second_type_name", sub_matches);

            TypeCommand::execute(client, type_name, second_type_name).await;
        }

        _ => (),
    };
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

fn get_bool(command: &str, sub_matches: &ArgMatches) -> bool {
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
            arg!(-c --category <CATEGORY> "Only show moves for a specific category").required(false),
            arg!(special: -s <SPECIAL> "Filter to only special attacks").required(false),
            arg!(physical: -p <PHYSICAL> "Filter to only physical attacks").required(false),
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
