use colored::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static TYPE_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    HashMap::from([
        (
            String::from("bug"),
            format!("{}", "bug".white().bold().on_truecolor(166, 185, 26)),
        ),
        (
            String::from("dark"),
            format!("{}", "dark".white().on_truecolor(112, 87, 70)),
        ),
        (
            String::from("dragon"),
            format!("{}", "dragon".white().on_truecolor(111, 53, 252)),
        ),
        (
            String::from("electric"),
            format!("{}", "electric".white().on_truecolor(247, 208, 44)),
        ),
        (
            String::from("fairy"),
            format!("{}", "fairy".white().on_truecolor(214, 133, 173)),
        ),
        (
            String::from("fighting"),
            format!("{}", "fighting".white().on_truecolor(194, 46, 40)),
        ),
        (
            String::from("fire"),
            format!("{}", "fire".white().on_truecolor(238, 129, 48)),
        ),
        (
            String::from("flying"),
            format!("{}", "flying".white().on_truecolor(169, 143, 243)),
        ),
        (
            String::from("ghost"),
            format!("{}", "ghost".white().on_truecolor(115, 87, 151)),
        ),
        (
            String::from("grass"),
            format!("{}", "grass".white().on_truecolor(122, 199, 76)),
        ),
        (
            String::from("ground"),
            format!("{}", "ground".white().on_truecolor(226, 191, 101)),
        ),
        (
            String::from("ice"),
            format!("{}", "ice".white().on_truecolor(150, 217, 214)),
        ),
        (
            String::from("normal"),
            format!("{}", "normal".white().on_truecolor(168, 167, 122)),
        ),
        (
            String::from("poison"),
            format!("{}", "poison".white().on_truecolor(163, 62, 161)),
        ),
        (
            String::from("psychic"),
            format!("{}", "psychic".white().on_truecolor(249, 85, 135)),
        ),
        (
            String::from("rock"),
            format!("{}", "rock".white().on_truecolor(182, 161, 54)),
        ),
        (
            String::from("steel"),
            format!("{}", "steel".white().on_truecolor(183, 183, 206)),
        ),
        (
            String::from("water"),
            format!("{}", "water".white().on_truecolor(99, 144, 240)),
        ),
    ])
});
