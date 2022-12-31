use crate::{
    client::{Client, ClientImplementation},
    formatter::{self, FormatAbility, FormatModel, FormatPokemon},
    name_matcher::matcher,
};

use futures::{stream, StreamExt};
use std::rc::Rc;

use rustemon::{model::pokemon::Pokemon, pokemon::pokemon, Follow};

static STAT_NAMES: &[&str] = &[
    "HP",
    "Attack",
    "Defense",
    "Special Attack",
    "Special Defense",
    "Speed",
];

pub struct PokemonCommand {
    client: Client,
    pokemon_name: String,
}

impl PokemonCommand {
    pub async fn execute(client: Client, pokemon_name: String) {
        PokemonCommand {
            client,
            pokemon_name,
        }
        ._execute()
        .await;
    }

    async fn _execute(&self) {
        let pokemon = self.fetch_pokemon().await;
        let format_pokemon = FormatPokemon::new(pokemon.clone());
        let pokemon_rc = Rc::new(pokemon);
        let mut output = String::new();

        self.build_summary(&format_pokemon, &mut output);
        self.build_stat_output(&pokemon_rc, &mut output);
        self.build_ability_output(&pokemon_rc, &mut output).await;

        println!("{}", output);
    }

    async fn fetch_pokemon(&self) -> Pokemon {
        match self.client.fetch_pokemon(&self.pokemon_name).await {
            Ok(pokemon) => pokemon,
            Err(_) => matcher::try_suggest_name(&self.pokemon_name, matcher::MatcherType::Pokemon),
        }
    }

    fn build_summary(&self, pokemon: &FormatPokemon, output: &mut String) {
        output.push_str("Summary\n");
        output.push_str(&pokemon.format());
    }

    fn build_stat_output(&self, pokemon: &Rc<Pokemon>, output: &mut String) {
        output.push_str("\nStats\n");
        let mut stat_total = 0;
        pokemon.stats.iter().enumerate().for_each(|(index, stat)| {
            // This assumes the stats returned from the API are always in the same order.
            // Because "PokemonStat" doesn't include the stats name, this is much simplier
            // than requesting for the Stat resource just for the corresponding name
            let stat_name = STAT_NAMES[index];
            let stat_amount = stat.base_stat;
            stat_total += stat_amount;
            output.push_str(&formatter::formatln(stat_name, &stat_amount.to_string()));
        });
        output.push_str(&formatter::formatln("Total", &stat_total.to_string()));
    }

    async fn build_ability_output(&self, pokemon: &Rc<Pokemon>, output: &mut String) {
        output.push_str("\nAbilities\n");
        stream::iter(&pokemon.abilities)
            .map(|a| {
                let pokemon_ref = &pokemon;

                async move {
                    let ability = self.client.fetch_ability(&a.ability.name).await.unwrap();

                    FormatAbility::new(ability, Rc::clone(pokemon_ref))
                }
            })
            .buffer_unordered(2)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|ability| {
                output.push_str(&format!("{}\n", ability.format()));
            });
    }
}
