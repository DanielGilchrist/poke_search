use crate::{
    builder::Builder,
    client::ClientImplementation,
    commands::type_command::TypeCommand,
    formatter::{self, FormatAbility, FormatModel, FormatPokemon},
    name_matcher::matcher,
};

use futures::{stream, StreamExt};
use std::rc::Rc;

use itertools::Itertools;
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
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    pokemon_name: String,
    show_types: bool,
}

impl PokemonCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        pokemon_name: String,
        show_types: bool,
    ) -> Builder {
        let mut builder = Builder::default();

        PokemonCommand {
            builder: &mut builder,
            client,
            pokemon_name,
            show_types,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let Ok(pokemon) = self.fetch_pokemon().await else {
            let suggestion =
                matcher::try_suggest_name(&self.pokemon_name, matcher::MatcherType::Pokemon);

            self.builder.append(suggestion);
            return;
        };

        let format_pokemon = FormatPokemon::new(pokemon.clone());
        let pokemon_rc = Rc::new(pokemon.clone());

        self.build_summary(&format_pokemon);
        self.build_stat_output(&pokemon_rc);
        self.build_ability_output(&pokemon_rc).await;

        if self.show_types {
            let types = pokemon
                .types
                .into_iter()
                .map(|t| t.type_.name)
                .collect::<Vec<_>>();

            let (type1, type2) = (types[0].to_string(), types.get(1).map(|t| t.to_string()));

            // TODO: We should extract the logic we need from this as it restricts what we can actually do with `TypeCommand`
            let type_builder = TypeCommand::execute(self.client, type1, type2, false).await;

            self.builder.append(formatter::white("Type information\n"));
            self.builder.append_builder(type_builder);
        }
    }

    async fn fetch_pokemon(&self) -> Result<Pokemon, rustemon::error::Error> {
        self.client.fetch_pokemon(&self.pokemon_name).await
    }

    fn build_summary(&mut self, pokemon: &FormatPokemon) {
        self.builder.append(formatter::white("Summary\n"));
        self.builder.append(pokemon.format());
    }

    fn build_stat_output(&mut self, pokemon: &Rc<Pokemon>) {
        self.builder.append(formatter::white("\nStats\n"));
        let mut stat_total = 0;
        pokemon.stats.iter().enumerate().for_each(|(index, stat)| {
            // This assumes the stats returned from the API are always in the same order.
            // Because "PokemonStat" doesn't include the stats name, this is much simplier
            // than requesting for the Stat resource just for the corresponding name
            let stat_name = &formatter::white(STAT_NAMES[index]);
            let stat_amount = stat.base_stat;
            stat_total += stat_amount;
            self.builder
                .append(formatter::formatln(stat_name, &stat_amount.to_string()));
        });
        self.builder.append(formatter::formatln(
            &formatter::white("Total"),
            &stat_total.to_string(),
        ));
    }

    async fn build_ability_output(&mut self, pokemon: &Rc<Pokemon>) {
        self.builder.append(formatter::white("\nAbilities\n"));

        let unique_pokemon_abilities = pokemon
            .abilities
            .iter()
            .unique_by(|pokemon_ability| &pokemon_ability.ability.name)
            .collect::<Vec<_>>();

        stream::iter(&unique_pokemon_abilities)
            .map(|a| {
                let pokemon_ref = &pokemon;
                let client_ref = &self.client;

                async move {
                    // TODO: Gracefully filter out failed requests for an ability
                    let ability = client_ref.fetch_ability(&a.ability.name).await.unwrap();

                    FormatAbility::new(ability, Rc::clone(pokemon_ref))
                }
            })
            .buffer_unordered(2)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|ability| {
                self.builder.append(format!("{}\n", ability.format()));
            });
    }
}
