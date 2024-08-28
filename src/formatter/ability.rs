use super::FormatModel;
use crate::formatter::utils::{extract_effect, formatln, split_and_capitalise, white};

use std::rc::Rc;

use rustemon::model::pokemon::{Ability, Pokemon};

pub struct FormatAbility {
    ability: Ability,
    pokemon: Option<Rc<Pokemon>>,
}

impl FormatAbility {
    pub fn new(ability: Ability, pokemon: Option<Rc<Pokemon>>) -> Self {
        FormatAbility { ability, pokemon }
    }

    fn build_description(&self, output: &mut String) {
        if let Some(hidden_value) = self.hidden_value() {
            output.push_str(&formatln(&white("Hidden"), &hidden_value));
        }

        if let Some(ability_effect) = self.ability_effect() {
            output.push_str(&formatln(&white("Description"), &ability_effect));
        }
    }

    fn hidden_value(&self) -> Option<String> {
        let pokemon = self.pokemon.clone()?;

        let is_hidden = self.ability.pokemon.iter().find_map(|ability_pokemon| {
            if ability_pokemon.pokemon.name == pokemon.name {
                Some(ability_pokemon.is_hidden)
            } else {
                None
            }
        })?;

        Some(is_hidden.to_string())
    }

    fn ability_effect(&self) -> Option<String> {
        let effect_entries = &self.ability.effect_entries;
        extract_effect(effect_entries)
    }
}

impl FormatModel for FormatAbility {
    fn format(&self) -> String {
        let mut output = String::new();

        let ability_name = split_and_capitalise(&self.ability.name);
        output.push_str(&formatln(&white("Name"), &ability_name));

        self.build_description(&mut output);

        output
    }
}
