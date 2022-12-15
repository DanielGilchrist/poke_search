use clap::{arg, Arg, Command};

mod commands;
use commands::move_command::MoveCommand;

#[tokio::main]
async fn main() {
    let client = rustemon::client::RustemonClient::default();

    match parse_commands().get_matches().subcommand() {
        Some(("moves", sub_matches)) => {
            let pokemon_name = sub_matches.get_one::<String>("pokemon").unwrap().to_owned();
            let type_name = sub_matches
                .get_one::<String>("type_name")
                .map(|s| s.to_owned());

            MoveCommand::execute(client, pokemon_name, type_name).await;
        }
        _ => (),
    };
}

fn parse_commands() -> Command {
    Command::new("poke_search_cli")
        .about("Search for pokemon information from the command line")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(parse_moves_command())
}

fn parse_moves_command() -> Command {
    Command::new("moves")
        .about("See moves for a pokemon")
        .args(moves_args())
        .arg_required_else_help(true)
}

fn moves_args() -> Vec<Arg> {
    vec![
        arg!(-p --pokemon <POKEMON_NAME> "The name of the pokemon you want to see moves for")
            .required(true),
        arg!(-t --type_name <TYPE_NAME> "The type of moves you want to see").required(false),
    ]
}
