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
        let successful_match =
            match matcher::match_name(&self.move_name, matcher::MatcherType::Move) {
                Ok(successful_match) => successful_match,
                Err(no_match) => {
                    self.builder.append(no_match.0);
                    return;
                }
            };

        let Ok(move_) = self.fetch_move(&successful_match.suggested_name).await else {
          let output = matcher::build_unknown_name(&successful_match.suggested_name, &successful_match.keyword);
          self.builder.append(output);
          return
        };

        let mut format_move = FormatMove::new(move_);

        self.builder.append(formatter::white("Move\n"));
        self.builder.append(format_move.format());

        if self.include_learned_by {
            self.build_learned_by(&mut format_move);
        }
    }

    async fn fetch_move(&self, move_name: &str) -> Result<Move, rustemon::error::Error> {
        self.client.fetch_move(move_name).await
    }

    fn build_learned_by(&mut self, format_move: &mut FormatMove) {
        let learned_by_pokemon = &mut format_move.move_.learned_by_pokemon;
        learned_by_pokemon.sort_by_key(|pokemon| pokemon.name.to_owned());

        let formatted_pokemon = learned_by_pokemon
            .iter_mut()
            .map(|pokemon| format!("  {}", formatter::split_and_capitalise(&pokemon.name)))
            .collect::<Vec<_>>()
            .join("\n");

        let header = formatter::white(&format!("\nLearned by: ({})\n", learned_by_pokemon.len()));
        self.builder.append(header);
        self.builder.append(formatted_pokemon);
    }
}
