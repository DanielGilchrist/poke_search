mod builder;
mod client;
mod formatter;
mod name_matcher;
mod type_colours;

use crate::{
    builder::Builder,
    client::{Client, ClientImplementation},
};

use clap::{Parser, Subcommand};

mod commands;
use commands::{
    ability_command::AbilityCommand, move_command::MoveCommand, moves_command::MovesCommand,
    pokemon_command::PokemonCommand, type_command::TypeCommand,
};

#[derive(Parser)]
#[command(about = "Search for pokemon information from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "See information about an ability")]
    Ability {
        #[arg(help = "The name of the ability you want to see information for")]
        ability: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Include a list of pokemon that have the ability")]
        pokemon: bool,
    },

    #[command(about = "See moves for a pokemon")]
    Moves {
        #[arg(help = "The name of the pokemon you want to see moves for")]
        pokemon: String,

        #[arg(short, long, num_args(0..))]
        #[arg(help = "The types of moves you want to see")]
        type_names: Option<Vec<String>>,

        #[arg(short, long, num_args(0..))]
        #[arg(help = "Only show moves for specific categories (physical, special, status)")]
        categories: Option<Vec<String>>,
    },

    #[command(about = "See information about a move")]
    Move {
        #[arg(help = "The name of the move you want to see information for")]
        move_name: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Include a list of pokemon that learn the move")]
        learned_by: bool,
    },

    #[command(about = "See information about a pokemon")]
    Pokemon {
        #[arg(help = "The name of the pokemon you want to see information for")]
        pokemon: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Show detailed type information")]
        types: bool,
    },

    #[command(about = "See information about a specific type")]
    Type {
        #[arg(help = "The name of the type you want to see information for")]
        type_name: String,

        #[arg(short, long)]
        #[arg(help = "Specify a second type for dual type information")]
        second_type_name: Option<String>,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "List pokemon that have the specified type/s")]
        pokemon: bool,
    },
}

#[tokio::main]
async fn main() {
    let client = Client::default();
    let cli = Cli::parse();

    run(&client, cli).await.print();
}

fn parse_name(name: &str) -> String {
    name.to_lowercase().split(' ').collect::<Vec<_>>().join("-")
}

async fn run(client: &dyn ClientImplementation, cli: Cli) -> Builder {
    match cli.command {
        Commands::Ability { ability, pokemon } => {
            let parsed_ability_name = parse_name(&ability);
            AbilityCommand::execute(client, parsed_ability_name, pokemon).await
        }

        Commands::Moves {
            pokemon,
            type_names,
            categories,
        } => {
            let parsed_pokemon_name = parse_name(&pokemon);
            MovesCommand::execute(client, parsed_pokemon_name, type_names, categories).await
        }

        Commands::Move {
            move_name,
            learned_by,
        } => {
            let parsed_move_name = parse_name(&move_name);
            MoveCommand::execute(client, parsed_move_name, learned_by).await
        }

        Commands::Pokemon { pokemon, types } => {
            let parsed_pokemon_name = parse_name(&pokemon);
            PokemonCommand::execute(client, parsed_pokemon_name, types).await
        }

        Commands::Type {
            type_name,
            second_type_name,
            pokemon,
        } => TypeCommand::execute(client, type_name, second_type_name, pokemon).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockClientImplementation;
    use crate::name_matcher::matcher;

    use rustemon::model::pokemon::Type;

    const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

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
    async fn pokemon_cant_be_found() -> Result<(), Box<dyn std::error::Error>> {
        let incorrect_name = "lkfdjslsdkjfkls";

        let mock_client = MockClientImplementation::new();
        let cli = parse_args(vec!["pokemon", incorrect_name]);
        let expected = matcher::build_unknown_name("pokemon", incorrect_name);
        let actual = run(&mock_client, cli).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

    #[tokio::test]
    async fn pokemon_uncertain_suggestion() -> Result<(), Box<dyn std::error::Error>> {
        let correct_name = "pikachu";
        let incorrect_name = "peacachu";

        let mock_client = MockClientImplementation::new();
        let cli = parse_args(vec!["pokemon", incorrect_name]);
        let expected = matcher::build_suggested_name("pokemon", incorrect_name, correct_name);
        let actual = run(&mock_client, cli).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

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

        mock_client
            .expect_fetch_type()
            .with(mockall::predicate::eq(correct_name))
            .once()
            .returning(|_args| Ok(Type::default()));

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

        mock_client
            .expect_fetch_type()
            .with(mockall::predicate::eq(correct_name))
            .once()
            .returning(|_args| Ok(Type::default()));

        let cli = parse_args(vec!["type", correct_name, "-s", incorrect_name]);
        let expected = matcher::build_suggested_name("type", incorrect_name, "psychic");
        let actual = run(&mock_client, cli).await.to_string();

        assert_eq!(expected, actual);

        Ok(())
    }

    fn parse_args(args: Vec<&str>) -> Cli {
        let mut full_args = vec![PACKAGE_NAME];
        full_args.extend(args);

        Cli::parse_from(full_args)
    }
}
