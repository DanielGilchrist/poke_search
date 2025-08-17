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
    ability_command::AbilityCommand, item_command::ItemCommand, move_command::MoveCommand,
    moves_command::MovesCommand, pokemon_command::PokemonCommand, type_command::TypeCommand,
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

#[tokio::main]
async fn main() {
    let client = Client::try_build().unwrap_or_else(|e| {
        eprintln!("Failed to initialise client: {e}");
        std::process::exit(1);
    });

    let cli = Cli::parse();

    run(&client, cli).await.print();
}

async fn run(client: &dyn ClientImplementation, cli: Cli) -> Builder {
    match cli.command {
        Commands::Ability { ability, pokemon } => {
            let parsed_ability_name = parse_name(&ability);
            AbilityCommand::execute(client, parsed_ability_name, pokemon).await
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

fn parse_name(name: &str) -> String {
    name.to_lowercase().split(' ').collect::<Vec<_>>().join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockClientImplementation;
    use crate::name_matcher::matcher;

    use rustemon::static_resources;

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
    async fn pokemon_move_autocorrect_if_similar_enough() -> Result<(), Box<dyn std::error::Error>>
    {
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
    async fn pokemon_autocorrect_if_similar_enough() -> Result<(), Box<dyn std::error::Error>> {
        let similar_name = "Charzard";
        let correct_name = "charizard";

        let mut mock_client = MockClientImplementation::new();
        let mock_pokemon = static_resources::get_pokemon();

        mock_client
            .expect_fetch_pokemon()
            .with(mockall::predicate::eq(correct_name))
            .once()
            .returning(move |_args| Ok(mock_pokemon.clone()));

        mock_client
            .expect_fetch_ability()
            .with(mockall::predicate::eq("blaze"))
            .once()
            .returning(move |_args| Ok(static_resources::get_ability()));

        mock_client
            .expect_fetch_ability()
            .with(mockall::predicate::eq("solar-power"))
            .once()
            .returning(move |_args| Ok(static_resources::get_ability()));

        let cli = parse_args(vec!["pokemon", similar_name]);

        let expected = r#"Summary
  Name: Charizard
  Type: Fire | flying
  Abilities: Blaze | Solar Power

Stats
  HP: 78
  Attack: 84
  Defense: 78
  Special Attack: 109
  Special Defense: 85
  Speed: 100
  Total: 534

Abilities
  Name: Static
  Description: Whenever a move makes contact with this Pokémon, the move's user has a 30%
    chance of being paralyzed.

    Pokémon that are immune to electric-type moves can still be paralyzed by this
    ability.

    Overworld: If the lead Pokémon has this ability, there is a 50% chance that
    encounters will be with an electric Pokémon, if applicable.

  Name: Static
  Description: Whenever a move makes contact with this Pokémon, the move's user has a 30%
    chance of being paralyzed.

    Pokémon that are immune to electric-type moves can still be paralyzed by this
    ability.

    Overworld: If the lead Pokémon has this ability, there is a 50% chance that
    encounters will be with an electric Pokémon, if applicable.

"#;

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
    async fn pokemon_single_type_autocorrect_if_similar_enough()
    -> Result<(), Box<dyn std::error::Error>> {
        colored::control::set_override(false);

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
        let expected = r#"fire

Offence
0.5x
  dragon | fire | rock | water
1x
  dark | electric | fairy | fighting | flying | ghost | ground | normal | poison | psychic | stellar
2x
  bug | grass | ice | steel

Defence
0.5x
  bug | fairy | fire | grass | ice | steel
1x
  dark | dragon | electric | fighting | flying | ghost | normal | poison | psychic | stellar
2x
  ground | rock | water
"#;
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

        mock_client
            .expect_fetch_type()
            .with(mockall::predicate::eq(correct_name))
            .once()
            .returning(move |_args| Ok(mock_type.clone()));

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
            .returning(|_args| Ok(static_resources::get_type()));

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
