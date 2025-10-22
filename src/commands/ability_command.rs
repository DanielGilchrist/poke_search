use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self, FormatAbility, FormatModel},
    name_matcher::matcher,
};

use rustemon::model::pokemon::Ability;

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
        let ability = match self.fetch_ability().await {
            Ok(ability) => ability,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };

        self.builder.append(formatter::white("Ability"));
        self.builder.append('\n');
        self.builder
            .append(FormatAbility::new(ability.clone(), None).format());

        if self.show_pokemon {
            self.builder.append('\n');

            let mut pokemon = ability.pokemon;
            pokemon.sort_by_key(|p| p.pokemon.name.clone());

            self.builder
                .append(formatter::white(&format!("Pokemon ({})\n", pokemon.len())));
            pokemon.iter().for_each(|ability_pokemon| {
                self.builder.append(formatter::split_and_capitalise(
                    &ability_pokemon.pokemon.name,
                ));
                self.builder.append('\n');
            });
        }
    }

    async fn fetch_ability(&self) -> Result<Ability, String> {
        let successful_match =
            match matcher::match_name(&self.ability_name, matcher::MatcherType::Ability) {
                Ok(successful_match) => Ok(successful_match),
                Err(no_match) => Err(no_match.0),
            }?;

        let result = self
            .client
            .fetch_ability(&successful_match.suggested_name)
            .await;

        match result {
            Ok(ability) => Ok(ability),
            Err(_) => {
                let output = matcher::build_unknown_name(
                    &successful_match.keyword,
                    &successful_match.suggested_name,
                );
                Err(output)
            }
        }
    }
}
