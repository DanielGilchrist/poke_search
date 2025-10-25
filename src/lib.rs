pub mod builder;
pub mod client;
pub mod commands;
pub mod formatter;
pub mod name_matcher;
pub mod type_badge;

pub use crate::{
    builder::Builder,
    client::{Client, ClientImplementation},
};

use clap::{Parser, Subcommand};

use commands::{
    ability_command::AbilityCommand, generation_command::GenerationCommand,
    item_command::ItemCommand, move_command::MoveCommand, moves_command::MovesCommand,
    pokemon_command::PokemonCommand, type_command::TypeCommand,
};

#[derive(Parser)]
#[command(about = "Search for pokemon information from the command line")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "See information about an ability")]
    Ability {
        #[arg(help = "The name of the ability you want to see information for")]
        ability: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Include a list of pokemon that have the ability")]
        pokemon: bool,
    },

    #[command(about = "Information about a particular generation of pokemon")]
    Generation {
        #[arg(help = "The generation you want to see information for")]
        generation: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Include a list of pokemon in the generation")]
        pokemon: bool,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Include a list of abilities in the generation")]
        abilities: bool,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Include a list of moves in the generation")]
        moves: bool,
    },

    #[command(about = "See information about an item")]
    Item {
        #[arg(help = "The name of the item you want to see information for")]
        item: String,
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

        #[arg(short, long, num_args(0..), requires = "learned_by")]
        #[arg(help = "Filter results of `learned_by` by particular types")]
        types: Option<Vec<String>>,
    },

    #[command(about = "See information about a pokemon")]
    Pokemon {
        #[arg(help = "The name of the pokemon you want to see information for")]
        pokemon: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Show detailed type information")]
        types: bool,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Show evolution information")]
        evolution: bool,
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

pub async fn run(client: &dyn ClientImplementation, cli: Cli) -> Builder {
    match cli.command {
        Commands::Ability { ability, pokemon } => {
            let parsed_ability_name = parse_name(&ability);
            AbilityCommand::execute(client, parsed_ability_name, pokemon).await
        }

        Commands::Generation {
            generation,
            pokemon,
            abilities,
            moves,
        } => {
            let parsed_generation = parse_generation(&generation);
            GenerationCommand::execute(client, parsed_generation, pokemon, abilities, moves).await
        }

        Commands::Item { item } => {
            let parsed_item_name = parse_name(&item);
            ItemCommand::execute(client, parsed_item_name).await
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
            types,
        } => {
            let parsed_move_name = parse_name(&move_name);
            MoveCommand::execute(client, parsed_move_name, learned_by, types).await
        }

        Commands::Pokemon {
            pokemon,
            types,
            evolution,
        } => {
            let parsed_pokemon_name = parse_name(&pokemon);
            PokemonCommand::execute(client, parsed_pokemon_name, types, evolution).await
        }

        Commands::Type {
            type_name,
            second_type_name,
            pokemon,
        } => TypeCommand::execute(client, type_name, second_type_name, pokemon).await,
    }
}

pub fn parse_name(name: &str) -> String {
    name.to_lowercase().split(' ').collect::<Vec<_>>().join("-")
}

pub fn parse_generation(generation_name: &str) -> String {
    if generation_name.contains("generation") {
        parse_name(generation_name)
    } else {
        format!("generation-{generation_name}")
    }
}
