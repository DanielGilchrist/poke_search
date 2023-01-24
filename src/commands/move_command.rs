use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self, FormatModel, FormatMove},
    name_matcher::matcher,
};

use rustemon::model::moves::Move;

pub struct MoveCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    move_name: String,
    include_learned_by: bool,
}

impl MoveCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        move_name: String,
        include_learned_by: bool,
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
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let Some(move_) = self.fetch_move().await else {
            let suggestion = matcher::try_suggest_name(&self.move_name, matcher::MatcherType::Move);

            self.builder.append(suggestion);
            return;
        };

        let mut format_move = FormatMove::new(move_);

        self.builder.append("Move\n");
        self.builder.append(format_move.format());

        self.maybe_build_learned_by(&mut format_move);
    }

    async fn fetch_move(&self) -> Option<Move> {
        self.client.fetch_move(&self.move_name).await.ok()
    }

    fn maybe_build_learned_by(&mut self, format_move: &mut FormatMove) {
        if !self.include_learned_by {
            return;
        }

        self.builder.append("\nLearned by:\n");

        let learned_by_pokemon = &mut format_move.move_.learned_by_pokemon;
        learned_by_pokemon.sort_by_key(|pokemon| pokemon.name.to_owned());

        let formatted_pokemon = learned_by_pokemon
            .iter_mut()
            .map(|pokemon| format!("  {}", formatter::split_and_capitalise(&pokemon.name)))
            .collect::<Vec<_>>()
            .join("\n");

        self.builder.append(formatted_pokemon);
    }
}
