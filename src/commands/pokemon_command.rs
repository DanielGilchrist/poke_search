use crate::formatter::{FormatModel};

use std::process::exit;
use futures::{stream, StreamExt};

use rustemon::{client::RustemonClient, model::pokemon::Pokemon, pokemon::pokemon, Follow};

pub struct PokemonCommand {
    client: RustemonClient,
    pokemon_name: String,
}

impl PokemonCommand {
    pub async fn execute(client: RustemonClient, pokemon_name: String) {
        PokemonCommand {
            client,
            pokemon_name,
        }
        ._execute()
        .await;
    }

    async fn _execute(&self) {
        let pokemon = self.fetch_pokemon().await;
        let output = pokemon.format();

        println!("{}", "Pokemon");
        println!("{}", output);

        println!("{}", "Stats");
        let full_stats = stream::iter(pokemon.stats)
          .map(|pokemon_stat| {
            let client_ref = &self.client;

            async move {
              pokemon_stat
                .stat
                .follow(client_ref)
                .await
                .unwrap()
            }
          })
          .buffer_unordered(6)
          .collect::<Vec<_>>()
          .await;

        println!("{:?}", full_stats.into_iter().map(|s| s.name).collect::<Vec<_>>());
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
}
