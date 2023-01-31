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
    type_name: Option<String>,
    category: Option<String>,
}

impl MovesCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        pokemon_name: String,
        type_name: Option<String>,
        category: Option<String>,
    ) -> Builder {
        let mut builder = Builder::new(BUILDER_CAPACITY);

        MovesCommand {
            builder: &mut builder,
            client,
            pokemon_name,
            type_name,
            category,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let Some(pokemon) = self.fetch_pokemon().await else {
            let suggestion =
                matcher::try_suggest_name(&self.pokemon_name, matcher::MatcherType::Pokemon);

            self.builder.append(suggestion);
            return;
        };

        let moves = self.process_moves(self.fetch_moves(pokemon.moves).await);
        let move_output = self.build_output(moves);
        let pokemon_name = formatter::capitalise(&pokemon.name);

        if let Some(type_name) = &self.type_name {
            // move_output can be empty only if a type_name filter is passed and there are no moves of that type
            if move_output.is_empty() {
                println!(
                    "{} has no {} type moves",
                    pokemon_name,
                    formatter::capitalise(type_name)
                );

                return;
            }
        };

        println!("Pokemon: {pokemon_name}");

        self.builder.append("Moves:\n");
        self.builder.append(move_output);
    }

    async fn fetch_pokemon(&self) -> Option<Pokemon> {
        self.client.fetch_pokemon(&self.pokemon_name).await.ok()
    }

    async fn fetch_moves(&self, pokemon_moves: Vec<PokemonMove>) -> Vec<FormatMove> {
        stream::iter(pokemon_moves)
            .map(|pokemon_move| async move {
                let version_group_details = pokemon_move.version_group_details.last().unwrap();

                let move_learn_method = self
                    .client
                    .fetch_move_learn_method(&version_group_details.move_learn_method.name)
                    .await
                    .unwrap();

                let move_ = self
                    .client
                    .fetch_move(&pokemon_move.move_.name)
                    .await
                    .unwrap();

                FormatMove::with_details(
                    move_,
                    move_learn_method,
                    version_group_details.level_learned_at,
                )
            })
            .buffer_unordered(100)
            .collect::<Vec<_>>()
            .await
    }

    fn process_moves(&self, moves: Vec<FormatMove>) -> Vec<FormatMove> {
        let mut processed_moves = moves;

        processed_moves = match &self.type_name {
            Some(type_name) => processed_moves
                .into_iter()
                .filter_map(|format_move| {
                    let move_ = &format_move.move_;

                    if &move_.type_.name == type_name {
                        Some(format_move)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            None => processed_moves,
        };

        processed_moves = match &self.category {
            Some(category) => processed_moves
                .into_iter()
                .filter_map(|format_move| {
                    let move_ = &format_move.move_;

                    if &move_.damage_class.name == category {
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

    fn build_output(&self, moves: Vec<FormatMove>) -> String {
        moves
            .into_iter()
            .fold(String::new(), |mut output, format_move| {
                output.push_str(&format_move.format());
                output.push_str("\n\n");

                output
            })
    }
}
