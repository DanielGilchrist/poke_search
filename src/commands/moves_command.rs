use crate::{formatter, formatter::FormatModel};

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
}

impl MovesCommand {
    pub async fn execute(client: RustemonClient, pokemon_name: String, type_name: Option<String>) {
        MovesCommand {
            client,
            pokemon_name,
            type_name,
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
            Err(_) => {
                println!("Pokemon \"{}\" doesn't exist", self.pokemon_name);
                exit(1);
            }
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
        let mut filtered_moves = match &self.type_name {
            Some(type_name) => moves
                .into_iter()
                .filter_map(|move_| {
                    if &move_.type_.name == type_name {
                        Some(move_)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            None => moves,
        };

        filtered_moves.sort_by_key(|move_| move_.power);
        filtered_moves.reverse();

        filtered_moves
    }

    fn build_output(&self, moves: Vec<Move>) -> String {
        moves.into_iter().fold(String::new(), |mut output, move_| {
            output.push_str(&move_.format());
            output.push_str("\n\n");

            output
        })
    }
}
