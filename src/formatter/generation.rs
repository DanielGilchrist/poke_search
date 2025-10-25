use crate::formatter::{self, FormatModel};
use rustemon::model::games::Generation;

pub struct FormatGeneration {
    generation: Generation,
    show_pokemon: bool,
    show_abilities: bool,
    show_moves: bool,
}

impl FormatGeneration {
    pub fn new(
        generation: Generation,
        show_pokemon: bool,
        show_abilities: bool,
        show_moves: bool,
    ) -> Self {
        Self {
            generation,
            show_pokemon,
            show_abilities,
            show_moves,
        }
    }

    fn name(&self) -> String {
        self.generation
            .names
            .iter()
            .find(|n| n.language.name == "en")
            .map(|n| n.name.clone())
            .unwrap_or_else(|| self.generation.name.clone())
    }

    fn main_region(&self) -> String {
        formatter::split_and_capitalise(&self.generation.main_region.name)
    }

    fn pokemon_count(&self) -> usize {
        self.generation.pokemon_species.len()
    }

    fn moves_count(&self) -> usize {
        self.generation.moves.len()
    }

    fn abilities_count(&self) -> usize {
        self.generation.abilities.len()
    }

    fn build_pokemon_list(&self, output: &mut String) {
        let generation = &self.generation;

        output.push_str(&formatter::white(&format!(
            "Pokemon ({})",
            generation.pokemon_species.len()
        )));
        output.push('\n');

        let mut pokemon_species = generation.pokemon_species.clone();
        pokemon_species.sort_by_key(|p| p.name.clone());

        let pokemon_names: Vec<String> = pokemon_species
            .iter()
            .map(|species| formatter::split_and_capitalise(&species.name))
            .collect();

        output.push_str(&formatter::format_columns(&pokemon_names, 4));
    }

    fn build_ability_list(&self, output: &mut String) {
        let generation = &self.generation;

        output.push_str(&formatter::white(&format!(
            "Abilities ({})",
            generation.abilities.len()
        )));
        output.push('\n');

        let mut abilities = generation.abilities.clone();
        abilities.sort_by_key(|a| a.name.clone());

        let ability_names: Vec<String> = abilities
            .iter()
            .map(|ability| formatter::split_and_capitalise(&ability.name))
            .collect();

        output.push_str(&formatter::format_columns(&ability_names, 4));
    }

    fn build_move_list(&self, output: &mut String) {
        let generation = &self.generation;

        output.push_str(&formatter::white(&format!(
            "Moves ({})",
            generation.moves.len()
        )));
        output.push('\n');

        let mut moves = generation.moves.clone();
        moves.sort_by_key(|m| m.name.clone());

        let move_names: Vec<String> = moves
            .iter()
            .map(|move_| formatter::split_and_capitalise(&move_.name))
            .collect();

        output.push_str(&formatter::format_columns(&move_names, 4));
    }
}

impl FormatModel for FormatGeneration {
    fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&formatter::white("Generation"));
        output.push('\n');

        output.push_str(&formatter::formatln(
            &formatter::white("Name"),
            &self.name(),
        ));
        output.push_str(&formatter::formatln(
            &formatter::white("Main Region"),
            &self.main_region(),
        ));
        output.push_str(&formatter::formatln(
            &formatter::white("Pokemon"),
            &self.pokemon_count().to_string(),
        ));
        output.push_str(&formatter::formatln(
            &formatter::white("Moves"),
            &self.moves_count().to_string(),
        ));
        output.push_str(&formatter::formatln(
            &formatter::white("Abilities"),
            &self.abilities_count().to_string(),
        ));

        if self.show_pokemon {
            output.push('\n');
            self.build_pokemon_list(&mut output);
        }

        if self.show_abilities {
            output.push('\n');
            self.build_ability_list(&mut output);
        }

        if self.show_moves {
            output.push('\n');
            self.build_move_list(&mut output);
        }

        output
    }
}
