use crate::{formatter, formatter::FormatModel, name_matcher::matcher};

use std::process::exit;

use rustemon::{
    client::RustemonClient,
    model::{
        moves::Move,
        pokemon::{Pokemon, PokemonMove},
    },
    pokemon::pokemon,
    Follow,
};

use futures::{stream, StreamExt};

pub struct MovesCommand {
    client: RustemonClient,
    pokemon_name: String,
    type_name: Option<String>,
    category: Option<String>,
}

impl MovesCommand {
    pub async fn execute(
        client: RustemonClient,
        pokemon_name: String,
        type_name: Option<String>,
        category: Option<String>,
    ) {
        MovesCommand {
            client,
            pokemon_name,
            type_name,
            category,
        }
        ._execute()
        .await;
    }

    async fn _execute(&self) {
        let pokemon = self.fetch_pokemon().await;
        let moves = self.process_moves(self.fetch_moves(pokemon.moves).await);
        let move_output = self.build_output(moves);

        let pokemon_name = formatter::capitalise(&pokemon.name);
        println!("Pokemon: {}", pokemon_name);

        if !move_output.is_empty() {
            println!("Moves:");
            println!("{}", move_output);
        } else {
            match &self.type_name {
                Some(type_name) => {
                    println!(
                        "{} has no {} type moves",
                        pokemon_name,
                        formatter::capitalise(type_name)
                    );
                }
                None => (),
            };
        }
    }

    async fn fetch_pokemon(&self) -> Pokemon {
        match pokemon::get_by_name(&self.pokemon_name, &self.client).await {
            Ok(pokemon) => pokemon,
            Err(_) => matcher::try_suggest_name(&self.pokemon_name, matcher::MatcherType::Pokemon),
        }
    }

    async fn fetch_moves(&self, pokemon_moves: Vec<PokemonMove>) -> Vec<Move> {
        stream::iter(pokemon_moves)
            .map(|move_resource| {
                let client_ref = &self.client;

                async move { move_resource.move_.follow(client_ref).await.unwrap() }
            })
            .buffer_unordered(100)
            .collect::<Vec<_>>()
            .await
    }

    fn process_moves(&self, moves: Vec<Move>) -> Vec<Move> {
        let mut processed_moves = moves;

        processed_moves = match &self.type_name {
            Some(type_name) => processed_moves
                .into_iter()
                .filter_map(|move_| {
                    if &move_.type_.name == type_name {
                        Some(move_)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            None => processed_moves,
        };

        processed_moves = match &self.category {
            Some(category) => processed_moves
                .into_iter()
                .filter_map(|move_| {
                    if &move_.damage_class.name == category {
                        Some(move_)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            None => processed_moves,
        };

        processed_moves.sort_by_key(|move_| move_.power);
        processed_moves.reverse();

        processed_moves
    }

    fn build_output(&self, moves: Vec<Move>) -> String {
        moves.into_iter().fold(String::new(), |mut output, move_| {
            output.push_str(&move_.format());
            output.push_str("\n\n");

            output
        })
    }
}
