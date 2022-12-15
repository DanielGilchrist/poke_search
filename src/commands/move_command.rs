use crate::formatter;

use std::process::exit;

use rustemon::{client::RustemonClient, model::moves::Move, moves::move_};

pub struct MoveCommand {
    client: RustemonClient,
    move_name: String,
    include_learned_by: bool,
}

impl MoveCommand {
    pub async fn execute(client: RustemonClient, move_name: String, include_learned_by: bool) {
        MoveCommand {
            client,
            move_name,
            include_learned_by,
        }
        ._execute()
        .await;
    }

    async fn _execute(&self) {
        let move_ = self.fetch_move().await;

        let mut output = String::from("Move\n");

        output.push_str(&formatter::format(&move_));

        if self.include_learned_by {
            output.push_str("\nLearned by:");

            let mut learned_by_pokemon = move_.learned_by_pokemon;
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
        match move_::get_by_name(&self.move_name, &self.client).await {
            Ok(move_) => move_,
            Err(_) => {
                println!("Move \"{}\" doesn't exist", self.move_name);
                exit(1)
            }
        }
    }
}
