use crate::{
    formatter::utils::{formatln, split_and_capitalise, white},
    type_colours::{self},
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
    }

    fn build_joined_types(&self, output: &mut String) {
        let joined_types = self
            .pokemon
            .types
            .iter()
            .map(|pokemon_type| type_colours::fetch(&pokemon_type.type_.name))
            .collect::<Vec<_>>()
            .join(" | ");

        output.push_str(&formatln(&white("Type"), &joined_types));
    }

    fn build_joined_abilities(&self, output: &mut String) {
        let unique_abilities = self
            .pokemon
            .abilities
            .iter()
            .unique_by(|pokemon_ability| &pokemon_ability.ability.name)
            .collect::<Vec<_>>();

        let joined_abilities = unique_abilities
            .iter()
            .map(|pokemon_ability| split_and_capitalise(&pokemon_ability.ability.name))
            .collect::<Vec<_>>()
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
}

impl FormatModel for FormatPokemon {
    fn format(&self) -> String {
        let mut output = String::new();

        self.build_summary(&mut output);

        output
    }
}
