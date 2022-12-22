use crate::type_colours::{self};

use std::rc::Rc;

use colored::*;
use rustemon::model::{
    moves::Move,
    pokemon::{Ability, Pokemon},
};

pub trait FormatModel {
    fn format(&self) -> String;
}

impl FormatModel for Move {
    fn format(&self) -> String {
        let mut output = String::new();

        let formatted_name = split_and_capitalise(&self.name);

        output.push_str(&formatln("Name", &formatted_name));
        output.push_str(&formatln("Type", &type_colours::fetch(&self.type_.name)));
        output.push_str(&formatln("Damage Type", &self.damage_class.name));

        let power = parse_maybe_i64(self.power);
        output.push_str(&formatln("Power", &power));
        output.push_str(&formatln("Accuracy", &parse_maybe_i64(self.accuracy)));
        output.push_str(&formatln("PP", &parse_maybe_i64(self.pp)));

        let flavour_text = self
            .flavor_text_entries
            .iter()
            .find_map(|entry| {
                if entry.language.name == "en" {
                    Some(&entry.flavor_text)
                } else {
                    None
                }
            })
            .unwrap()
            .replace('\n', " ");

        output.push_str(&formatln("Description", &flavour_text));

        let effect_chance = format!("{}%", parse_maybe_i64(self.effect_chance));
        self.effect_entries.iter().for_each(|entry| {
            let description = if power == "-" {
                entry.effect.replace('\n', " ").replace("  ", " ")
            } else {
                entry
                    .short_effect
                    .replace("$effect_chance%", &effect_chance)
            };

            output.push_str(&formatln("Effect", &description));
        });

        output
    }
}

impl FormatModel for Pokemon {
    fn format(&self) -> String {
        let mut output = String::new();
        let formatted_name = split_and_capitalise(&self.name);

        output.push_str(&formatln("Name", &formatted_name));

        let joined_types = self
            .types
            .iter()
            .map(|pokemon_type| type_colours::fetch(&pokemon_type.type_.name))
            .collect::<Vec<_>>()
            .join(" | ");

        output.push_str(&formatln("Type", &joined_types));

        let joined_abilities = self
            .abilities
            .iter()
            .map(|pokemon_ability| split_and_capitalise(&pokemon_ability.ability.name))
            .collect::<Vec<_>>()
            .join(" | ");

        output.push_str(&formatln("Abilities", &joined_abilities));

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
        let hidden_value = self.hidden_value();
        output.push_str(&formatln("Hidden", &hidden_value));

        let ability_effect = self.ability_effect();
        output.push_str(&formatln("Description", &ability_effect));
    }

    fn hidden_value(&self) -> String {
        self.ability
            .pokemon
            .iter()
            .find_map(|ability_pokemon| {
                if ability_pokemon.pokemon.name == self.pokemon.name {
                    Some(ability_pokemon.is_hidden)
                } else {
                    None
                }
            })
            .unwrap()
            .to_string()
    }

    fn ability_effect(&self) -> String {
        self.ability
            .effect_entries
            .iter()
            .find_map(|verbose_effect| {
                if verbose_effect.language.name == "en" {
                    Some(&verbose_effect.effect)
                } else {
                    None
                }
            })
            .unwrap()
            .replace('\n', " ")
    }
}

impl FormatModel for FormatAbility {
    fn format(&self) -> String {
        let mut output = String::new();

        let ability_name = split_and_capitalise(&self.ability.name);
        output.push_str(&formatln("Name", &ability_name));

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
    s.split('-')
        .into_iter()
        .map(capitalise)
        .collect::<Vec<_>>()
        .join(" ")
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
    format!("{}", coloured_string)
}
