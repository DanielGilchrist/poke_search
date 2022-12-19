use crate::formatter::{FormatAbility, FormatModel};

use futures::{stream, StreamExt};
use std::{process::exit, rc::Rc};

use rustemon::{client::RustemonClient, model::pokemon::Pokemon, pokemon::pokemon, Follow};

static STAT_NAMES: &[&str] = &[
    "HP",
    "Attack",
    "Defense",
    "Special Attack",
    "Special Defense",
    "Speed",
];

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
        let pokemon = Rc::new(self.fetch_pokemon().await);
        let output = pokemon.format();

        println!("Summary");
        println!("{}", output);

        println!("Stats");
        pokemon.stats.iter().enumerate().for_each(|(index, stat)| {
            // This assumes the stats returned from the API are always in the same order.
            // Because "PokemonStat" doesn't include the stats name, this is much simplier
            // than requesting for the Stat resource just for the corresponding name
            let stat_name = STAT_NAMES[index];
            println!("{}: {}", stat_name, stat.base_stat);
        });

        println!("Abilities");
        stream::iter(&pokemon.abilities)
            .map(|a| {
                let client_ref = &self.client;
                let pokemon_ref = &pokemon;

                async move {
                    let ability = a.ability.follow(client_ref).await.unwrap();

                    FormatAbility::new(ability, Rc::clone(pokemon_ref))
                }
            })
            .buffer_unordered(2)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|ability| {
                println!("{}", ability.format());
            });
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
