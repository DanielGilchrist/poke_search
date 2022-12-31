use crate::{
    client::{Client, ClientImplementation},
    formatter::{self, FormatModel, FormatMove},
    name_matcher::matcher,
};

use rustemon::model::moves::Move;

pub struct MoveCommand {
    client: Client,
    move_name: String,
    include_learned_by: bool,
}

impl MoveCommand {
    pub async fn execute(client: Client, move_name: String, include_learned_by: bool) {
        MoveCommand {
            client,
            move_name,
            include_learned_by,
        }
        ._execute()
        .await;
    }

    async fn _execute(&self) {
        let format_move = FormatMove::new(self.fetch_move().await);

        let mut output = String::from("Move\n");

        output.push_str(&format_move.format());

        if self.include_learned_by {
            output.push_str("\nLearned by:\n");

            let mut learned_by_pokemon = format_move.move_.learned_by_pokemon;
            learned_by_pokemon.sort_by_key(|pokemon| pokemon.name.to_owned());

            let formatted_pokemon = learned_by_pokemon
                .into_iter()
                .map(|pokemon| format!("  {}", formatter::split_and_capitalise(&pokemon.name)))
                .collect::<Vec<_>>()
                .join("\n");

            output.push_str(&formatted_pokemon);
        }

        println!("{}", output);
    }

    async fn fetch_move(&self) -> Move {
        match self.client.fetch_move(&self.move_name).await {
            Ok(move_) => move_,
            Err(_) => matcher::try_suggest_name(&self.move_name, matcher::MatcherType::Move),
        }
    }
}
