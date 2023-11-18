use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter,
    formatter::{FormatModel, FormatMove},
    name_matcher::matcher,
};

use rustemon::model::pokemon::{Pokemon, PokemonMove};

use futures::{stream, StreamExt};

const BUILDER_CAPACITY: usize = 50000;

pub struct MovesCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    pokemon_name: String,
    type_names: Option<Vec<String>>,
    categories: Option<Vec<String>>,
}

impl MovesCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        pokemon_name: String,
        type_names: Option<Vec<String>>,
        categories: Option<Vec<String>>,
    ) -> Builder {
        let mut builder = Builder::new(BUILDER_CAPACITY);

        MovesCommand {
            builder: &mut builder,
            client,
            pokemon_name,
            type_names,
            categories,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let pokemon = match self.fetch_pokemon().await {
            Ok(pokemon) => pokemon,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };

        let moves = self.process_moves(self.fetch_moves(pokemon.moves).await);
        let move_output = self.build_output(&moves);
        let pokemon_name = formatter::capitalise(&pokemon.name);

        if let Some(type_names) = &self.type_names {
            // move_output can be empty only if a type_name filter is passed and there are no moves of that type
            if move_output.is_empty() {
                let empty_output = format!(
                    "{} has no {} type moves",
                    pokemon_name,
                    type_names
                        .iter()
                        .map(|t| formatter::capitalise(t))
                        .collect::<Vec<_>>()
                        .join(" or ")
                );

                self.builder.append(empty_output);

                return;
            }
        };

        self.builder
            .append(format!("{} {pokemon_name}\n", formatter::white("Pokemon:")));

        self.builder
            .append(formatter::white(&format!("Moves: ({})\n", moves.len())));
        self.builder.append(move_output);
    }

    async fn fetch_pokemon(&self) -> Result<Pokemon, String> {
        let successful_match =
            match matcher::match_name(&self.pokemon_name, matcher::MatcherType::Pokemon) {
                Ok(successful_match) => Ok(successful_match),
                Err(no_match) => Err(no_match.0),
            }?;

        match self
            .client
            .fetch_pokemon(&successful_match.suggested_name)
            .await
        {
            Ok(pokemon) => Ok(pokemon),
            Err(_) => {
                let output = matcher::build_unknown_name(
                    &successful_match.suggested_name,
                    &successful_match.keyword,
                );
                Err(output)
            }
        }
    }

    async fn fetch_moves(&self, pokemon_moves: Vec<PokemonMove>) -> Vec<FormatMove> {
        stream::iter(pokemon_moves)
            .map(|pokemon_move| async move {
                let version_group_details = pokemon_move.version_group_details.last();

                let move_learn_method = if let Some(version_group_details) = version_group_details {
                    self.client
                        .fetch_move_learn_method(&version_group_details.move_learn_method.name)
                        .await
                        .ok()
                } else {
                    None
                };

                // TODO: Gracecfully filter out failed requests for a move
                let move_ = self
                    .client
                    .fetch_move(&pokemon_move.move_.name)
                    .await
                    .unwrap();

                FormatMove::with_maybe_details(move_, move_learn_method, version_group_details)
            })
            .buffer_unordered(100)
            .collect::<Vec<_>>()
            .await
    }

    fn process_moves(&self, moves: Vec<FormatMove>) -> Vec<FormatMove> {
        let mut processed_moves = moves;

        processed_moves = match &self.type_names {
            Some(type_names) => {
                let corrected_type_names = type_names
                    .iter()
                    .map(|type_name| {
                        match matcher::match_name(type_name, matcher::MatcherType::Type) {
                            Ok(successful_match) => successful_match.suggested_name,
                            Err(_) => type_name.to_owned(),
                        }
                    })
                    .collect::<Vec<_>>();

                processed_moves
                    .into_iter()
                    .filter_map(|format_move| {
                        let move_ = &format_move.move_;

                        if corrected_type_names.contains(&move_.type_.name) {
                            Some(format_move)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            }
            None => processed_moves,
        };

        processed_moves = match &self.categories {
            Some(categories) => processed_moves
                .into_iter()
                .filter_map(|format_move| {
                    let move_ = &format_move.move_;

                    if categories.contains(&move_.damage_class.name) {
                        Some(format_move)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            None => processed_moves,
        };

        processed_moves.sort_by_key(|format_move| format_move.move_.power);
        processed_moves.reverse();

        processed_moves
    }

    fn build_output(&self, moves: &[FormatMove]) -> String {
        moves.iter().fold(String::new(), |mut output, format_move| {
            output.push_str(&format_move.format());
            output.push('\n');

            output
        })
    }
}
