use crate::type_colours::{self};

use std::rc::Rc;

use colored::*;
use itertools::Itertools;
use rustemon::model::{
    moves::{Move, MoveLearnMethod},
    pokemon::{Ability, Pokemon, PokemonMoveVersion},
};

pub trait FormatModel {
    fn format(&self) -> String;
}

struct MoveDetails {
    level_learned_at: i64,
    move_learn_method: MoveLearnMethod,
}

pub struct FormatMove {
    pub move_: Move,
    move_details: Option<MoveDetails>,
}

impl FormatMove {
    pub fn new(move_: Move) -> Self {
        FormatMove {
            move_,
            move_details: None,
        }
    }

    pub fn with_maybe_details(
        move_: Move,
        move_learn_method: Option<MoveLearnMethod>,
        version_group_details: Option<&PokemonMoveVersion>,
    ) -> Self {
        if let (Some(move_learn_method), Some(version_group_details)) =
            (move_learn_method, version_group_details)
        {
            return FormatMove {
                move_,
                move_details: Some(MoveDetails {
                    move_learn_method,
                    level_learned_at: version_group_details.level_learned_at,
                }),
            };
        }

        Self::new(move_)
    }

    fn build_summary(&self, output: &mut String) {
        let formatted_name = split_and_capitalise(&self.move_.name);

        output.push_str(&formatln(&white("Name"), &formatted_name));
        output.push_str(&formatln(
            &white("Type"),
            &type_colours::fetch(&self.move_.type_.name),
        ));
        output.push_str(&formatln(
            &white("Damage Type"),
            &self.move_.damage_class.name,
        ));
    }

    fn build_details(&self, output: &mut String) {
        let power = parse_maybe_i64(self.move_.power);

        output.push_str(&formatln(&white("Power"), &power));
        output.push_str(&formatln(
            &white("Accuracy"),
            &parse_maybe_i64(self.move_.accuracy),
        ));
        output.push_str(&formatln(&white("PP"), &parse_maybe_i64(self.move_.pp)));
        output.push_str(&formatln(
            &white("Priority"),
            &self.move_.priority.to_string(),
        ));

        if let Some(flavour_text) = self.flavour_text() {
            output.push_str(&formatln(&white("Description"), &flavour_text));
        }

        self.build_effects(power, output);
    }

    fn flavour_text(&self) -> Option<String> {
        let text = self.move_.flavor_text_entries.iter().find_map(|entry| {
            if entry.language.name == "en" {
                Some(&entry.flavor_text)
            } else {
                None
            }
        })?;

        Some(text.replace('\n', " "))
    }

    fn build_effects(&self, power: String, output: &mut String) {
        let effect_chance = format!("{}%", parse_maybe_i64(self.move_.effect_chance));
        self.move_.effect_entries.iter().for_each(|entry| {
            let description = if power == "-" {
                entry.effect.replace('\n', " ").replace("  ", " ")
            } else {
                entry
                    .short_effect
                    .replace("$effect_chance%", &effect_chance)
            };

            output.push_str(&formatln(&white("Effect"), &description));
        });
    }

    fn build_move_learn_details(&self, output: &mut String) {
        if let Some(move_details) = &self.move_details {
            if let Some(description) =
                self.find_move_learn_description(&move_details.move_learn_method)
            {
                output.push_str(&formatln(&white("Learn Method"), &description));
            }

            let level_learned_at = move_details.level_learned_at;
            if level_learned_at > 0 {
                output.push_str(&formatln(
                    &white("Learn Level"),
                    &level_learned_at.to_string(),
                ));
            }
        }
    }

    fn find_move_learn_description(&self, move_learn_method: &MoveLearnMethod) -> Option<String> {
        let english_description = move_learn_method
            .descriptions
            .iter()
            .find(|description| description.language.name == "en")?;

        Some(english_description.description.to_owned())
    }
}

impl FormatModel for FormatMove {
    fn format(&self) -> String {
        let mut output = String::new();

        self.build_summary(&mut output);
        self.build_details(&mut output);
        self.build_move_learn_details(&mut output);

        output
    }
}

pub struct FormatPokemon(pub Pokemon);

impl FormatPokemon {
    pub fn new(pokemon: Pokemon) -> Self {
        FormatPokemon(pokemon)
    }

    fn build_summary(&self, output: &mut String) {
        let formatted_name = split_and_capitalise(&self.0.name);
        output.push_str(&formatln(&white("Name"), &formatted_name));

        self.build_joined_types(output);
        self.build_joined_abilities(output);
    }

    fn build_joined_types(&self, output: &mut String) {
        let joined_types = self
            .0
            .types
            .iter()
            .map(|pokemon_type| type_colours::fetch(&pokemon_type.type_.name))
            .collect::<Vec<_>>()
            .join(" | ");

        output.push_str(&formatln(&white("Type"), &joined_types));
    }

    fn build_joined_abilities(&self, output: &mut String) {
        let unique_abilities = self
            .0
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
}

impl FormatModel for FormatPokemon {
    fn format(&self) -> String {
        let mut output = String::new();

        self.build_summary(&mut output);

        output
    }
}

pub struct FormatAbility {
    ability: Ability,
    pokemon: Rc<Pokemon>,
}

impl FormatAbility {
    pub fn new(ability: Ability, pokemon: Rc<Pokemon>) -> Self {
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
        let is_hidden = self.ability.pokemon.iter().find_map(|ability_pokemon| {
            if ability_pokemon.pokemon.name == self.pokemon.name {
                Some(ability_pokemon.is_hidden)
            } else {
                None
            }
        })?;

        Some(is_hidden.to_string())
    }

    fn ability_effect(&self) -> Option<String> {
        let effect = self
            .ability
            .effect_entries
            .iter()
            .find_map(|verbose_effect| {
                if verbose_effect.language.name == "en" {
                    Some(&verbose_effect.effect)
                } else {
                    None
                }
            })?;

        Some(effect.replace('\n', " "))
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

pub fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn split_and_capitalise(s: &str) -> String {
    s.split('-').map(capitalise).collect::<Vec<_>>().join(" ")
}

pub fn formatln(title: &str, value: &str) -> String {
    format!("  {}{}{}\n", title, ": ", capitalise(value))
}

fn parse_maybe_i64(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => String::from("-"),
    }
}

// Colours
pub fn white(str: &str) -> String {
    format_colour(str.white())
}

pub fn green(str: &str) -> String {
    format_colour(str.green())
}

pub fn yellow(str: &str) -> String {
    format_colour(str.yellow())
}

pub fn red(str: &str) -> String {
    format_colour(str.red())
}

pub fn bright_red(str: &str) -> String {
    format_colour(str.bright_red())
}

pub fn bright_green(str: &str) -> String {
    format_colour(str.bright_green())
}

fn format_colour(coloured_string: ColoredString) -> String {
    format!("{coloured_string}")
}
