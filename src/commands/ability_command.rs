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
    verbose: bool,
}

impl AbilityCommand<'_> {
    pub async fn execute(
        client: &dyn ClientImplementation,
        ability_name: String,
        show_pokemon: bool,
        verbose: bool,
    ) -> Builder {
        let mut builder = Builder::default();

        AbilityCommand {
            builder: &mut builder,
            client,
            ability_name,
            show_pokemon,
            verbose,
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

        self.builder.appendln(formatter::white("Ability"));

        let format_ability = FormatAbility::new(ability.clone()).with_verbose(self.verbose);
        self.builder.append(format_ability.format());

        if self.show_pokemon {
            self.builder.newline();

            let mut pokemon = ability.pokemon;
            pokemon.sort_by_key(|p| p.pokemon.name.clone());

            self.builder
                .appendln(formatter::white(&format!("Pokemon ({})", pokemon.len())));

            let pokemon_names = pokemon
                .iter()
                .map(|ability_pokemon| {
                    formatter::split_and_capitalise(&ability_pokemon.pokemon.name)
                })
                .collect::<Vec<_>>();

            self.builder
                .append(formatter::format_columns(&pokemon_names, 4));
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
