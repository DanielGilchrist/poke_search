use rustemon::model::moves::Move;

pub fn format(move_: &Move) -> String {
    let mut output = String::new();

    let formatted_name = split_and_capitalise(&move_.name);

    output.push_str(&formatln("Name", &formatted_name));
    output.push_str(&formatln("Type", &move_.type_.name));
    output.push_str(&formatln("Damage Type", &move_.damage_class.name));

    let power = parse_maybe_i64(move_.power);
    output.push_str(&formatln("Power", &power));
    output.push_str(&formatln("Accuracy", &parse_maybe_i64(move_.accuracy)));
    output.push_str(&formatln("PP", &parse_maybe_i64(move_.pp)));

    let flavour_text = move_
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

    let effect_chance = format!("{}%", parse_maybe_i64(move_.effect_chance));
    move_
        .effect_entries
        .iter()
        .for_each(|entry| {
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
