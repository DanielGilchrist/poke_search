use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self, FormatModel, FormatMove},
    name_matcher::matcher,
    type_badge,
};

use futures::{StreamExt, stream};
use itertools::{Itertools, any};
use rustemon::model::{moves::Move, pokemon::Pokemon};
use unicode_width::UnicodeWidthStr;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct FormattedPokemon {
    name: String,
    types: Vec<String>,
}

impl FormattedPokemon {
    pub fn formatted_type(&self) -> String {
        self.types
            .iter()
            .map(|type_name| type_badge::fetch(type_name))
            .join(" | ")
    }
}

impl From<Pokemon> for FormattedPokemon {
    fn from(pokemon: Pokemon) -> Self {
        let Pokemon {
            name: pokemon_name,
            types: pokemon_types,
            ..
        } = pokemon;

        let formatted_name = formatter::split_and_capitalise(&pokemon_name);
        let types = pokemon_types
            .into_iter()
            .map(|pokemon_type| pokemon_type.type_.name)
            .collect::<Vec<_>>();

        Self {
            name: formatted_name,
            types,
        }
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

        self.builder.appendln(formatter::white("Move"));
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
                any(&pokemon.types, |type_name| {
                    corrected_types.contains(type_name)
                })
            })
        }

        pokemon_list.sort();

        let max_name_width = pokemon_list
            .iter()
            .map(|pokemon| UnicodeWidthStr::width(pokemon.name.as_str()))
            .max()
            .unwrap_or(0);

        let num_columns = 2;
        let column_width = max_name_width + 4;

        let mut learned_by_output = String::new();
        for chunk in pokemon_list.chunks(num_columns) {
            for pokemon in chunk {
                let name_width = UnicodeWidthStr::width(pokemon.name.as_str());
                let padding = " ".repeat(column_width - name_width);
                learned_by_output.push_str("  ");
                learned_by_output.push_str(&pokemon.name);
                learned_by_output.push_str(&padding);
            }
            learned_by_output.push('\n');

            for pokemon in chunk {
                let type_str = pokemon.formatted_type();
                let plain_type = pokemon.types.join(" | ");
                let type_visual_width = UnicodeWidthStr::width(plain_type.as_str());
                let padding = " ".repeat(column_width - type_visual_width);
                learned_by_output.push_str("  ");
                learned_by_output.push_str(&type_str);
                learned_by_output.push_str(&padding);
            }
            learned_by_output.push_str("\n\n");
        }

        self.builder.newline();

        let header = formatter::white(&format!("Learned by: ({})", pokemon_list.len()));
        self.builder.appendln(header);
        self.builder.append(learned_by_output);
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
        self.types.as_ref().map(|type_names| {
            type_names
                .iter()
                .map(|type_name| self.try_correct_type(type_name))
                .collect::<Vec<_>>()
        })
    }

    fn try_correct_type(&self, type_name: &str) -> String {
        match matcher::match_name(type_name, matcher::MatcherType::Type) {
            Ok(successful_match) => successful_match.suggested_name,
            Err(_) => type_name.to_owned(),
        }
    }

    async fn fetch_formatted_pokemon(&self, pokemon_names: &Vec<String>) -> Vec<FormattedPokemon> {
        let client_ref = &self.client;
        stream::iter(pokemon_names)
            .map(|pokemon_name| async move {
                let pokemon = client_ref.fetch_pokemon(pokemon_name).await.unwrap();
                FormattedPokemon::from(pokemon)
            })
            .buffer_unordered(50)
            .collect::<Vec<_>>()
            .await
    }
}
