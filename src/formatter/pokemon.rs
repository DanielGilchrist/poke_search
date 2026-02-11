use crate::{
    formatter::utils::{formatln, split_and_capitalise, white},
    type_badge::{self},
};

use super::FormatModel;

use itertools::Itertools;
use rustemon::model::pokemon::{Pokemon, PokemonSpecies};

pub struct FormatPokemon {
    pokemon: Pokemon,
    species: PokemonSpecies,
}

impl FormatPokemon {
    pub fn new(pokemon: Pokemon, species: PokemonSpecies) -> Self {
        Self { pokemon, species }
    }

    fn build_summary(&self, output: &mut String) {
        let formatted_name = split_and_capitalise(&self.pokemon.name);
        output.push_str(&formatln(&white("Name"), &formatted_name));

        self.build_joined_types(output);
        self.build_joined_abilities(output);
        self.build_generation(output);
        self.build_effort_values(output);
    }

    fn build_joined_types(&self, output: &mut String) {
        let joined_types = self
            .pokemon
            .types
            .iter()
            .map(|pokemon_type| type_badge::fetch(&pokemon_type.type_.name))
            .join(" | ");

        output.push_str(&formatln(&white("Type"), &joined_types));
    }

    fn build_joined_abilities(&self, output: &mut String) {
        let unique_abilities = self
            .pokemon
            .abilities
            .iter()
            .filter_map(|pokemon_ability| pokemon_ability.ability.as_ref())
            .unique_by(|ability| &ability.name);

        let joined_abilities = unique_abilities
            .map(|ability| split_and_capitalise(&ability.name))
            .join(" | ");

        output.push_str(&formatln(&white("Abilities"), &joined_abilities));
    }

    fn build_generation(&self, output: &mut String) {
        if let Some(generation_numeral) = self.species.generation.name.split('-').next_back() {
            output.push_str(&formatln(
                &white("Generation"),
                &generation_numeral.to_uppercase(),
            ))
        }
    }

    fn build_effort_values(&self, output: &mut String) {
        let mut effort_values = self.pokemon.stats.iter().filter_map(|pokemon_stat| {
            if pokemon_stat.effort > 0 {
                Some(format!(
                    "{} +{}",
                    split_and_capitalise(&pokemon_stat.stat.name),
                    pokemon_stat.effort
                ))
            } else {
                None
            }
        });

        output.push_str(&formatln(
            &white("Effort Values"),
            &effort_values.join(" | "),
        ))
    }
}

impl FormatModel for FormatPokemon {
    fn format(&self) -> String {
        let mut output = String::new();

        self.build_summary(&mut output);

        output
    }
}
