use std::{collections::HashMap, iter};

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
        let mut pokemon_names = format_move
            .move_
            .learned_by_pokemon
            .iter()
            .map(|pokemon| pokemon.name.clone())
            .collect::<Vec<_>>();

        let corrected_types = self.types.clone().map(|type_names| {
            type_names
                .iter()
                .map(
                    |type_name| match matcher::match_name(type_name, matcher::MatcherType::Type) {
                        Ok(successful_match) => successful_match.suggested_name,
                        Err(_) => type_name.to_owned(),
                    },
                )
                .collect::<Vec<_>>()
        });

        if let Some(corrected_types) = &corrected_types {
            let pokemon_by_name = stream::iter(&pokemon_names)
                .map(|pokemon_name| {
                    let client_ref = &self.client;

                    async move { client_ref.fetch_pokemon(&pokemon_name).await.unwrap() }
                })
                .buffer_unordered(50)
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .map(|pokemon| (pokemon.name.clone(), pokemon))
                .collect::<HashMap<String, Pokemon>>();

            pokemon_names.retain(|pokemon_name| match pokemon_by_name.get(pokemon_name) {
                Some(pokemon) => itertools::any(&pokemon.types, |pokemon_type| {
                    corrected_types.contains(&pokemon_type.type_.name)
                }),
                None => false,
            });
        }

        pokemon_names.sort();

        let formatted_pokemon = pokemon_names
            .iter_mut()
            .map(|name| format!("  {}", formatter::split_and_capitalise(&name)))
            .collect::<Vec<_>>()
            .join("\n");

        let header = formatter::white(&format!("\nLearned by: ({})\n", pokemon_names.len()));
        self.builder.append(header);
        self.builder.append(formatted_pokemon);
    }
}
