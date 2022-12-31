use crate::client::ClientImplementation;
use crate::{
    formatter::{self, FormatAbility, FormatModel, FormatPokemon},
    name_matcher::matcher,
};

use futures::{stream, StreamExt};
use std::rc::Rc;

use rustemon::model::pokemon::Pokemon;

static STAT_NAMES: &[&str] = &[
    "HP",
    "Attack",
    "Defense",
    "Special Attack",
    "Special Defense",
    "Speed",
];

pub struct PokemonCommand<'a> {
    client: &'a dyn ClientImplementation,
    pokemon_name: String,
}

impl PokemonCommand<'_> {
    pub async fn execute(client: &dyn ClientImplementation, pokemon_name: String) -> String {
        PokemonCommand {
            client,
            pokemon_name,
        }
        ._execute()
        .await
    }

    async fn _execute(&self) -> String {
        let mut output = String::new();

        match self.fetch_pokemon().await {
            Some(pokemon) => {
                let format_pokemon = FormatPokemon::new(pokemon.clone());
                let pokemon_rc = Rc::new(pokemon);

                self.build_summary(&format_pokemon, &mut output);
                self.build_stat_output(&pokemon_rc, &mut output);
                self.build_ability_output(&pokemon_rc, &mut output).await;
            }

            None => {
                let suggestion =
                    matcher::try_suggest_name(&self.pokemon_name, matcher::MatcherType::Pokemon);
                output.push_str(&suggestion);
            }
        };

        output
    }

    async fn fetch_pokemon(&self) -> Option<Pokemon> {
        self.client.fetch_pokemon(&self.pokemon_name).await.ok()
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
