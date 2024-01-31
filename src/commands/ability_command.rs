use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self, FormatAbility, FormatModel},
};

pub struct AbilityCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    ability_name: String,
    show_pokemon: bool,
}

impl AbilityCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        ability_name: String,
        show_pokemon: bool,
    ) -> Builder {
        let mut builder = Builder::default();

        AbilityCommand {
            builder: &mut builder,
            client,
            ability_name,
            show_pokemon,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let ability = match self.client.fetch_ability(&self.ability_name).await {
            Ok(ability) => ability,
            Err(error_message) => {
                self.builder.append(error_message.to_string());
                return;
            }
        };

        self.builder.append(formatter::white("Ability"));
        self.builder.append_c('\n');
        self.builder
            .append(FormatAbility::new(ability.clone(), None).format());

        if self.show_pokemon {
            self.builder.append_c('\n');

            let mut pokemon = ability.pokemon;
            pokemon.sort_by_key(|p| p.pokemon.name.clone());

            self.builder
                .append(formatter::white(&format!("Pokemon ({})\n", pokemon.len())));
            pokemon.iter().for_each(|ability_pokemon| {
                self.builder.append(formatter::split_and_capitalise(
                    &ability_pokemon.pokemon.name,
                ));
                self.builder.append_c('\n');
            });
        }
    }
}
