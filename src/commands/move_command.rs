use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self, FormatModel, FormatMove},
    name_matcher::matcher,
    type_colours,
};

use futures::{stream, StreamExt};
use itertools::Itertools;
use rustemon::model::{moves::Move, pokemon::Pokemon};

struct FormattedPokemon {
    name: String,
    types: Vec<String>,
}

impl FormattedPokemon {
    pub fn from(pokemon: Pokemon) -> Self {
        let formatted_name = formatter::split_and_capitalise(&pokemon.name);
        let types = pokemon
            .types
            .iter()
            .map(|pokemon_type| pokemon_type.type_.name.clone())
            .collect::<Vec<_>>();

        Self {
            name: formatted_name,
            types,
        }
    }

    pub fn formatted_type(&self) -> String {
        self.types
            .iter()
            .map(|type_name| type_colours::fetch(type_name))
            .join(" | ")
    }
}

pub struct MoveCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    move_name: String,
    include_learned_by: bool,
    types: Option<Vec<String>>,
}

impl MoveCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        move_name: String,
        include_learned_by: bool,
        types: Option<Vec<String>>,
    ) -> Builder {
        let mut builder = if include_learned_by {
            Builder::new(3000)
        } else {
            Builder::default()
        };

        MoveCommand {
            builder: &mut builder,
            client,
            move_name,
            include_learned_by,
            types,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let move_ = match self.fetch_move().await {
            Ok(move_) => move_,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };

        let mut format_move = FormatMove::new(move_);

        self.builder.append(formatter::white("Move\n"));
        self.builder.append(format_move.format());

        if self.include_learned_by {
            self.build_learned_by(&mut format_move).await;
        }
    }

    async fn fetch_move(&self) -> Result<Move, String> {
        let successful_match =
            match matcher::match_name(&self.move_name, matcher::MatcherType::Move) {
                Ok(successful_match) => Ok(successful_match),
                Err(no_match) => Err(no_match.0),
            }?;

        let result = self
            .client
            .fetch_move(&successful_match.suggested_name)
            .await;

        match result {
            Ok(move_) => Ok(move_),
            Err(_) => {
                let output = matcher::build_unknown_name(
                    &successful_match.keyword,
                    &successful_match.suggested_name,
                );
                Err(output)
            }
        }
    }

    async fn build_learned_by(&mut self, format_move: &mut FormatMove) {
        let pokemon_names = self.pokemon_names(format_move);
        let corrected_types = self.corrected_types();
        let mut pokemon_list = self.fetch_formatted_pokemon(&pokemon_names).await;

        if let Some(corrected_types) = &corrected_types {
            pokemon_list.retain(|pokemon| {
                itertools::any(&pokemon.types, |type_name| {
                    corrected_types.contains(type_name)
                })
            })
        }

        pokemon_list.sort_by_key(|pokemon| pokemon.name.clone());

        let max_name_width = pokemon_list
            .iter()
            .map(|pokemon| pokemon.name.len())
            .max()
            .unwrap_or(0);

        let formatted_pokemon = pokemon_list
            .iter_mut()
            .map(|pokemon| {
                let formatted_type = pokemon.formatted_type();
                let name_width_diff = max_name_width - pokemon.name.len();
                let padded_name = format!("{}{}", pokemon.name, " ".repeat(name_width_diff));

                format!("  {} {formatted_type}", padded_name)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let header = formatter::white(&format!("\nLearned by: ({})\n", pokemon_list.len()));
        self.builder.append(header);
        self.builder.append(formatted_pokemon);
    }

    fn pokemon_names(&self, format_move: &FormatMove) -> Vec<String> {
        format_move
            .move_
            .learned_by_pokemon
            .iter()
            .map(|pokemon| pokemon.name.clone())
            .collect::<Vec<_>>()
    }

    fn corrected_types(&self) -> Option<Vec<String>> {
        self.types.clone().map(|type_names| {
            type_names
                .iter()
                .map(
                    |type_name| match matcher::match_name(type_name, matcher::MatcherType::Type) {
                        Ok(successful_match) => successful_match.suggested_name,
                        Err(_) => type_name.to_owned(),
                    },
                )
                .collect::<Vec<_>>()
        })
    }

    async fn fetch_formatted_pokemon(&self, pokemon_names: &Vec<String>) -> Vec<FormattedPokemon> {
        stream::iter(pokemon_names)
            .map(|pokemon_name| {
                let client_ref = &self.client;
                async move {
                    let pokemon = client_ref.fetch_pokemon(pokemon_name).await.unwrap();
                    FormattedPokemon::from(pokemon)
                }
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await
    }
}
