use rustemon::model::games::Generation;

use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{FormatGeneration, FormatModel},
};

pub struct GenerationCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    generation: String,
    show_pokemon: bool,
    show_abilities: bool,
    show_moves: bool,
}

impl GenerationCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        generation: String,
        show_pokemon: bool,
        show_abilities: bool,
        show_moves: bool,
    ) -> Builder {
        let mut builder = Builder::default();

        GenerationCommand {
            builder: &mut builder,
            client,
            generation,
            show_pokemon,
            show_abilities,
            show_moves,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let generation = match self.fetch_generation().await {
            Ok(generation) => generation,
            Err(error) => {
                self.builder.append(error);
                return;
            }
        };

        let format_generation = FormatGeneration::new(
            generation,
            self.show_pokemon,
            self.show_abilities,
            self.show_moves,
        );

        self.builder.append(format_generation.format());
    }

    async fn fetch_generation(&self) -> Result<Generation, String> {
        self.client
            .fetch_generation(&self.generation)
            .await
            .map_err(|error| error.to_string())
    }
}
