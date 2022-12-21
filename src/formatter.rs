use std::rc::Rc;

use rustemon::model::{
    moves::Move,
    pokemon::{Ability, Pokemon, PokemonMove},
};

pub struct FormatAbility {
    ability: Ability,
    pokemon: Rc<Pokemon>,
}

impl FormatAbility {
    pub fn new(ability: Ability, pokemon: Rc<Pokemon>) -> Self {
        FormatAbility { ability, pokemon }
    }
}

pub struct FormatMove {
  pub move_: Move,
  pokemon_move: Option<PokemonMove>,
}

impl FormatMove {
  pub fn new(move_: Move, pokemon_move: Option<PokemonMove>) -> Self {
    FormatMove {
      move_,
      pokemon_move
    }
  }
}

pub trait FormatModel {
    fn format(&self) -> String;
}

impl FormatModel for FormatMove {
    fn format(&self) -> String {
        let mut output = String::new();

        let formatted_name = split_and_capitalise(&self.move_.name);

        output.push_str(&formatln("Name", &formatted_name));
        output.push_str(&formatln("Type", &self.move_.type_.name));
        output.push_str(&formatln("Damage Type", &self.move_.damage_class.name));

        let power = parse_maybe_i64(self.move_.power);
        output.push_str(&formatln("Power", &power));
        output.push_str(&formatln("Accuracy", &parse_maybe_i64(self.move_.accuracy)));
        output.push_str(&formatln("PP", &parse_maybe_i64(self.move_.pp)));
        output.push_str(&formatln("Priority", &self.move_.priority.to_string()));

        let flavour_text = self.move_
            .flavor_text_entries
            .iter()
            .cloned()
            .find_map(|entry| {
                if entry.language.name == "en" {
                    Some(entry.flavor_text)
                } else {
                    None
                }
            })
            .unwrap()
            .replace('\n', " ");

        output.push_str(&formatln("Description", &flavour_text));

        let effect_chance = format!("{}%", parse_maybe_i64(self.move_.effect_chance));
        self.move_.effect_entries.iter().for_each(|entry| {
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
            .cloned()
            .map(|pokemon_type| capitalise(&pokemon_type.type_.name))
            .collect::<Vec<_>>()
            .join(" | ");

        output.push_str(&formatln("Type", &joined_types));

        let joined_abilities = self
            .abilities
            .iter()
            .cloned()
            .map(|pokemon_ability| split_and_capitalise(&pokemon_ability.ability.name))
            .collect::<Vec<_>>()
            .join(" | ");

        output.push_str(&formatln("Abilities", &joined_abilities));

        output
    }
}

impl FormatModel for FormatAbility {
    fn format(&self) -> String {
        let mut output = String::new();

        let ability_name = split_and_capitalise(&self.ability.name);
        output.push_str(&formatln("Name", &ability_name));

        let hidden_value = self
            .ability
            .pokemon
            .iter()
            .cloned()
            .find_map(|ability_pokemon| {
                if ability_pokemon.pokemon.name == self.pokemon.name {
                    Some(ability_pokemon.is_hidden)
                } else {
                    None
                }
            })
            .unwrap()
            .to_string();

        output.push_str(&formatln("Hidden", &hidden_value));

        let ability_entry = self
            .ability
            .effect_entries
            .iter()
            .cloned()
            .find_map(|verbose_effect| {
                if verbose_effect.language.name == "en" {
                    Some(verbose_effect.effect)
                } else {
                    None
                }
            })
            .unwrap()
            .replace('\n', " ");

        output.push_str(&formatln("Description", &ability_entry));

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
