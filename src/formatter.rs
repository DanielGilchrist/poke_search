use rustemon::model::{moves::Move, pokemon::Pokemon};

pub trait FormatModel {
  fn format(&self) -> String;
}

impl FormatModel for Move {
  fn format(&self) -> String {
    let mut output = String::new();

    let formatted_name = split_and_capitalise(&self.name);

    output.push_str(&formatln("Name", &formatted_name));
    output.push_str(&formatln("Type", &self.type_.name));
    output.push_str(&formatln("Damage Type", &self.damage_class.name));

    let power = parse_maybe_i64(self.power);
    output.push_str(&formatln("Power", &power));
    output.push_str(&formatln("Accuracy", &parse_maybe_i64(self.accuracy)));
    output.push_str(&formatln("PP", &parse_maybe_i64(self.pp)));

    let flavour_text = self
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

    let joined_types = self.types
      .iter()
      .cloned()
      .map(|pokemon_type| capitalise(&pokemon_type.type_.name))
      .collect::<Vec<_>>()
      .join(" | ");

    output.push_str(&formatln("Type", &joined_types));

    let joined_abilities = self.abilities
      .iter()
      .cloned()
      .map(|pokemon_ability| split_and_capitalise(&pokemon_ability.ability.name))
      .collect::<Vec<_>>()
      .join(" | ");

    output.push_str(&formatln("Abilities", &joined_abilities));

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
