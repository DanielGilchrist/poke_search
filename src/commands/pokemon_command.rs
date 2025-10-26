use crate::{
    builder::Builder,
    client::ClientImplementation,
    commands::type_command::TypeCommand,
    formatter::{self, FormatAbility, FormatModel, FormatPokemon},
    name_matcher::matcher,
};

use futures::{StreamExt, stream};
use std::{collections::BTreeMap, rc::Rc};

use colored::Colorize;
use itertools::Itertools;
use rustemon::model::{
    evolution::{ChainLink, EvolutionChain, EvolutionDetail},
    pokemon::{Pokemon, PokemonSpecies},
};

static STAT_NAMES: &[&str] = &[
    "HP",
    "Attack",
    "Defense",
    "Special Attack",
    "Special Defense",
    "Speed",
];

#[derive(Debug)]
struct NormalisedEvolutionDetail {
    item: Option<String>,
    trigger: String,
    gender: Option<String>,
    held_item: Option<String>,
    known_move: Option<String>,
    known_move_type: Option<String>,
    location: Option<String>,
    min_level: Option<i64>,
    min_happiness: Option<i64>,
    min_beauty: Option<i64>,
    min_affection: Option<i64>,
    needs_overworld_rain: bool,
    party_species: Option<String>,
    party_type: Option<String>,
    relative_physical_stats: Option<String>,
    time_of_day: Option<String>,
    trade_species: Option<String>,
    turn_upside_down: bool,
}

impl NormalisedEvolutionDetail {
    fn gender_id_to_string(gender_id: i64) -> String {
        match gender_id {
            1 => String::from("Female"),
            2 => String::from("Male"),
            _ => String::from("Genderless"),
        }
    }

    fn physical_stat_number_to_string(num: i64) -> String {
        match num {
            1 => String::from("Attack > Defense"),
            -1 => String::from("Attack < Defense"),
            _ => String::from("Attack = Defense"),
        }
    }
}

impl From<&EvolutionDetail> for NormalisedEvolutionDetail {
    fn from(detail: &EvolutionDetail) -> Self {
        Self {
            item: detail.item.as_ref().map(|item| item.name.clone()),
            trigger: detail.trigger.name.clone(),
            gender: detail.gender.map(Self::gender_id_to_string),
            held_item: detail.held_item.as_ref().map(|item| item.name.clone()),
            known_move: detail.known_move.as_ref().map(|move_| move_.name.clone()),
            known_move_type: detail
                .known_move_type
                .as_ref()
                .map(|type_| type_.name.clone()),
            location: detail.location.as_ref().map(|loc| loc.name.clone()),
            min_level: detail.min_level,
            min_happiness: detail.min_happiness,
            min_beauty: detail.min_beauty,
            min_affection: detail.min_affection,
            needs_overworld_rain: detail.needs_overworld_rain,
            party_species: detail
                .party_species
                .as_ref()
                .map(|species| species.name.clone()),
            party_type: detail.party_type.as_ref().map(|type_| type_.name.clone()),
            relative_physical_stats: detail
                .relative_physical_stats
                .map(Self::physical_stat_number_to_string),
            time_of_day: if detail.time_of_day.is_empty() {
                None
            } else {
                Some(detail.time_of_day.clone())
            },
            trade_species: detail
                .trade_species
                .as_ref()
                .map(|species| species.name.clone()),
            turn_upside_down: detail.turn_upside_down,
        }
    }
}

#[derive(Debug)]
struct NormalisedEvolutionPokemon {
    name: String,
    stage: u8,
    evolution_details: Vec<NormalisedEvolutionDetail>,
}

impl NormalisedEvolutionPokemon {
    fn from(chain: &ChainLink, stage: u8) -> Self {
        Self {
            name: chain.species.name.clone(),
            stage,
            evolution_details: chain
                .evolution_details
                .iter()
                .map(NormalisedEvolutionDetail::from)
                .collect(),
        }
    }
}

pub struct PokemonCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    pokemon_name: String,
    matched_pokemon_name: Option<String>,
    show_types: bool,
    show_evolution: bool,
}

impl PokemonCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        pokemon_name: String,
        show_types: bool,
        show_evolution: bool,
    ) -> Builder {
        let mut builder = Builder::default();

        PokemonCommand {
            builder: &mut builder,
            client,
            pokemon_name,
            matched_pokemon_name: None,
            show_types,
            show_evolution,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let (pokemon, matched_name) = match self.fetch_pokemon().await {
            Ok(result) => result,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };
        self.matched_pokemon_name = Some(matched_name);

        let species_name = &pokemon.species.name;
        let species = match self.fetch_pokemon_species(species_name).await {
            Ok(species) => species,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };

        let format_pokemon = FormatPokemon::new(pokemon.clone(), species.clone());
        let pokemon_rc = Rc::new(pokemon.clone());

        self.build_summary(&format_pokemon);
        self.builder.newline();

        self.build_stat_output(&pokemon_rc);
        self.builder.newline();

        self.build_ability_output(&pokemon_rc).await;

        if self.show_evolution {
            let evolution_chain = self.fetch_evolution_chain(&species).await;
            if let Some(evolution_chain) = evolution_chain {
                let normalised_evolution_chains = self.normalised_evolution_chains(evolution_chain);
                self.builder.newline();
                self.build_evolution_output(normalised_evolution_chains);
            };
        }

        if self.show_types {
            let types = pokemon
                .types
                .into_iter()
                .map(|t| t.type_.name)
                .collect::<Vec<_>>();

            let (type1, type2) = (types[0].to_string(), types.get(1).map(ToString::to_string));

            // TODO: We should extract the logic we need from this as it restricts what we can actually do with `TypeCommand`
            let type_builder = TypeCommand::execute(self.client, type1, type2, false).await;

            self.builder.newline();
            self.builder.appendln(formatter::white("Type information"));
            self.builder.append(type_builder);
        }
    }

    fn normalised_evolution_chains(
        &self,
        evolution_chain: EvolutionChain,
    ) -> Vec<NormalisedEvolutionPokemon> {
        let mut normalised_evolution_pokemon = Vec::new();

        self.extract_and_normalize_chain_links(
            &evolution_chain.chain,
            &mut normalised_evolution_pokemon,
            1,
        );

        normalised_evolution_pokemon.sort_unstable_by_key(|ep| (ep.stage, ep.name.clone()));

        normalised_evolution_pokemon
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extract_and_normalize_chain_links(
        &self,
        chain_link: &ChainLink,
        normalized_links: &mut Vec<NormalisedEvolutionPokemon>,
        stage: u8,
    ) {
        normalized_links.push(NormalisedEvolutionPokemon::from(chain_link, stage));
        let next_stage = stage + 1;
        for sub_chain_link in &chain_link.evolves_to {
            self.extract_and_normalize_chain_links(sub_chain_link, normalized_links, next_stage);
        }
    }

    async fn fetch_pokemon(&self) -> Result<(Pokemon, String), String> {
        let successful_match =
            matcher::match_name(&self.pokemon_name, matcher::MatcherType::Pokemon)
                .map_err(|no_match| no_match.0)?;

        let matched_name = successful_match.suggested_name.clone();
        let pokemon = self
            .client
            .fetch_pokemon(&successful_match.suggested_name)
            .await
            .map_err(|error| error.to_string())?;

        Ok((pokemon, matched_name))
    }

    async fn fetch_pokemon_species(&self, species_name: &str) -> Result<PokemonSpecies, String> {
        self.client
            .fetch_pokemon_species(species_name)
            .await
            .map_err(|error| error.to_string())
    }

    async fn fetch_evolution_chain(&self, species: &PokemonSpecies) -> Option<EvolutionChain> {
        let chain_url = &species.evolution_chain.as_ref()?.url;

        self.client
            .fetch_evolution_chain_from_url(chain_url)
            .await
            .ok()
    }

    fn build_summary(&mut self, pokemon: &FormatPokemon) {
        self.builder.appendln(formatter::white("Summary"));
        self.builder.append(pokemon.format());
    }

    fn build_stat_output(&mut self, pokemon: &Rc<Pokemon>) {
        self.builder.appendln(formatter::white("Stats"));
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
        self.builder.appendln(formatter::white("Abilities"));

        let unique_pokemon_abilities = pokemon
            .abilities
            .iter()
            .unique_by(|pokemon_ability| &pokemon_ability.ability.name)
            .collect::<Vec<_>>();

        let pokemon_ref = &pokemon;
        let client_ref = &self.client;

        stream::iter(&unique_pokemon_abilities)
            .map(|a| async move {
                // TODO: Gracefully filter out failed requests for an ability
                let ability = client_ref.fetch_ability(&a.ability.name).await.unwrap();
                FormatAbility::new(ability).with_pokemon(Rc::clone(pokemon_ref))
            })
            .buffer_unordered(2)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|ability| {
                self.builder.appendln(ability.format());
            });

        self.builder.pop();
    }

    fn formatted_pokemon_name(&self, pokemon_name: &str) -> String {
        let mut formatted_name = formatter::capitalise(pokemon_name);

        if let Some(matched_name) = &self.matched_pokemon_name {
            if matched_name == pokemon_name {
                formatted_name = formatted_name.bold().italic().to_string();
            }
        }

        formatted_name
    }

    fn build_evolution_output(&mut self, evolution_chains: Vec<NormalisedEvolutionPokemon>) {
        self.builder.appendln(formatter::white("Evolution Chain:"));

        let evolution_chains_by_stage = self.group_by_key(evolution_chains, |chain| chain.stage);

        for (stage, chains) in evolution_chains_by_stage {
            self.builder
                .append(formatter::white(&format!("  Stage {stage}:")));

            let mut prefix = String::from(" ");

            if chains.len() > 1 {
                prefix.push_str("   ");
                self.builder.newline();
            }

            for chain in chains {
                let pokemon_name = self.formatted_pokemon_name(&chain.name);
                self.builder.append(&prefix);
                self.builder.append(pokemon_name);

                let evolution_details = chain.evolution_details;
                let evolution_details_by_trigger =
                    self.group_by_key(evolution_details, |detail| detail.trigger.clone());

                let detail_strings = evolution_details_by_trigger
                    .into_iter()
                    .map(|(trigger, details)| {
                        let joined_details = details
                            .iter()
                            .filter_map(|detail| {
                                let mut detail_builder = Builder::default();

                                self.build_detail(&mut detail_builder, detail);

                                if detail_builder.is_empty() {
                                    None
                                } else {
                                    Some(detail_builder.to_string())
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" | ");

                        let mut details_builder = Builder::default();

                        details_builder.append(" (");
                        details_builder
                            .append(formatter::white(&formatter::split_and_capitalise(&trigger)));

                        if !joined_details.is_empty() {
                            details_builder.append(" - ");
                        }

                        details_builder.append(joined_details);
                        details_builder.append(")");

                        details_builder.to_string()
                    })
                    .collect::<Vec<_>>();

                let joined_details = detail_strings.join(" or");
                self.builder.appendln(joined_details);
            }
        }
    }

    fn build_detail(&self, builder: &mut Builder, detail: &NormalisedEvolutionDetail) {
        let mut details: Vec<String> = Vec::new();

        self.maybe_transform_and_append_detail(&mut details, &detail.item, |item_str| {
            formatter::split_and_capitalise(item_str)
        });

        self.maybe_append_detail(&mut details, &detail.gender);

        self.maybe_transform_and_append_detail(&mut details, &detail.held_item, |held_item_str| {
            formatter::split_and_capitalise(held_item_str)
        });

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.known_move,
            |known_move_str| {
                let known_move_name = formatter::split_and_capitalise(known_move_str);
                format!("Move {known_move_name}")
            },
        );

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.known_move_type,
            |known_move_type_str| {
                let known_move_type_name = formatter::split_and_capitalise(known_move_type_str);
                format!("Move type {known_move_type_name}")
            },
        );

        self.maybe_transform_and_append_detail(&mut details, &detail.location, |location_str| {
            formatter::split_and_capitalise(location_str)
        });

        self.maybe_transform_and_append_detail(&mut details, &detail.min_level, |min_level| {
            format!("Level {min_level}")
        });

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.min_happiness,
            |min_happiness| format!("Happiness {min_happiness}"),
        );

        self.maybe_transform_and_append_detail(&mut details, &detail.min_beauty, |min_beauty| {
            format!("Beauty {min_beauty}")
        });

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.min_affection,
            |min_affection| format!("Affection {min_affection}"),
        );

        self.maybe_transform_and_append_detail(
            &mut details,
            &Some(detail.needs_overworld_rain),
            |needs_overworld_rain| {
                if *needs_overworld_rain {
                    String::from("In the rain")
                } else {
                    String::new()
                }
            },
        );

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.party_species,
            |party_species_str| {
                let party_species_name = formatter::split_and_capitalise(party_species_str);
                format!("{party_species_name} in party")
            },
        );

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.party_type,
            |party_type_str| {
                let party_type_name = formatter::capitalise(party_type_str);
                format!("{party_type_name} type pokemon in party")
            },
        );

        self.maybe_append_detail(&mut details, &detail.relative_physical_stats);

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.time_of_day,
            |time_of_day_str| {
                let time_of_day_name = formatter::capitalise(time_of_day_str);
                format!("During the {time_of_day_name}")
            },
        );

        self.maybe_transform_and_append_detail(
            &mut details,
            &detail.trade_species,
            |trade_species_str| {
                let trade_species_name = formatter::split_and_capitalise(trade_species_str);
                format!("Trade with {trade_species_name}")
            },
        );

        self.maybe_transform_and_append_detail(
            &mut details,
            &Some(detail.turn_upside_down),
            |turn_upside_down| {
                if *turn_upside_down {
                    String::from("Turn upside down")
                } else {
                    String::new()
                }
            },
        );

        builder.append(details.join(" & "));
    }

    fn maybe_append_detail<T: ToString>(&self, details: &mut Vec<String>, value: &Option<T>) {
        self.__maybe_transform_and_append_detail(details, value, None::<fn(&T) -> String>);
    }

    fn maybe_transform_and_append_detail<T, F>(
        &self,
        details: &mut Vec<String>,
        value: &Option<T>,
        transform: F,
    ) where
        T: ToString,
        F: FnOnce(&T) -> String,
    {
        self.__maybe_transform_and_append_detail(details, value, Some(transform))
    }

    fn __maybe_transform_and_append_detail<T, F>(
        &self,
        details: &mut Vec<String>,
        value: &Option<T>,
        transform: Option<F>,
    ) where
        T: ToString,
        F: FnOnce(&T) -> String,
    {
        if let Some(value) = value {
            let detail = match transform {
                Some(transform) => transform(value),
                None => value.to_string(),
            };

            if !detail.is_empty() {
                details.push(formatter::white(&detail));
            }
        }
    }

    fn group_by_key<T, K, F>(&self, items: Vec<T>, key_fn: F) -> BTreeMap<K, Vec<T>>
    where
        K: Ord,
        F: Fn(&T) -> K,
    {
        let mut grouped: BTreeMap<K, Vec<T>> = BTreeMap::new();

        for item in items {
            grouped.entry(key_fn(&item)).or_default().push(item);
        }

        grouped
    }
}
