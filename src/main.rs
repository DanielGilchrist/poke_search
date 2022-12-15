mod formatter;

use clap::{arg, Command};

mod commands;
use commands::{move_command::MoveCommand, moves_command::MovesCommand};

#[tokio::main]
async fn main() {
    let client = rustemon::client::RustemonClient::default();

    match parse_commands().get_matches().subcommand() {
        Some(("moves", sub_matches)) => {
            let pokemon_name = parse_name(sub_matches.get_one::<String>("pokemon")).unwrap();

            let type_name = sub_matches
                .get_one::<String>("type_name")
                .map(|s| s.to_owned());

            MovesCommand::execute(client, pokemon_name, type_name).await;
        }
        Some(("move", sub_matches)) => {
            let move_name = parse_name(sub_matches.get_one::<String>("move")).unwrap();

            let include_learned_by = sub_matches
                .get_one::<bool>("learned_by")
                .unwrap_or(&false)
                .to_owned();

            MoveCommand::execute(client, move_name, include_learned_by).await;
        }
        _ => (),
    };
}

fn parse_commands() -> Command {
    Command::new("poke_search_cli")
        .about("Search for pokemon information from the command line")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([parse_moves_command(), parse_move_command()])
}

fn parse_moves_command() -> Command {
    Command::new("moves")
        .about("See moves for a pokemon")
        .args([
            arg!(-p --pokemon <POKEMON_NAME> "The name of the pokemon you want to see moves for")
                .required(true),
            arg!(-t --type_name <TYPE_NAME> "The type of moves you want to see").required(false),
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

fn parse_name(name: Option<&String>) -> Option<String> {
    name.map(|n| n.to_lowercase().split(' ').collect::<Vec<_>>().join("-"))
}
