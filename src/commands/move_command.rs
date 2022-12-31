use crate::{
    formatter::{self, FormatModel, FormatMove},
    name_matcher::matcher,
};
use poke_search_cli::client::{ClientImplementation};

use rustemon::model::moves::Move;

pub struct MoveCommand<'a> {
    client: &'a dyn ClientImplementation,
    move_name: String,
    include_learned_by: bool,
}

impl MoveCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        move_name: String,
        include_learned_by: bool,
    ) -> String {
        MoveCommand {
            client,
            move_name,
            include_learned_by,
        }
        ._execute()
        .await
    }

    async fn _execute(&self) -> String {
        let mut output = String::new();

        match self.fetch_move().await {
            Some(move_) => {
                let format_move = FormatMove::new(move_);

                output.push_str("Move\n");
                output.push_str(&format_move.format());

                if self.include_learned_by {
                    output.push_str("\nLearned by:\n");

                    let mut learned_by_pokemon = format_move.move_.learned_by_pokemon;
                    learned_by_pokemon.sort_by_key(|pokemon| pokemon.name.to_owned());

                    let formatted_pokemon = learned_by_pokemon
                        .into_iter()
                        .map(|pokemon| {
                            format!("  {}", formatter::split_and_capitalise(&pokemon.name))
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    output.push_str(&formatted_pokemon);
                }
            }

            None => {
                let suggestion =
                    matcher::try_suggest_name(&self.move_name, matcher::MatcherType::Move);
                output.push_str(&suggestion);
            }
        }

        output
    }

    async fn fetch_move(&self) -> Option<Move> {
        self.client.fetch_move(&self.move_name).await.ok()
    }
}
