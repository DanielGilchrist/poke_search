use colored::Colorize;
use once_cell::sync::Lazy;
use std::collections::HashMap;

type RGB = (u8, u8, u8);

static TYPE_NAME_TO_RGB: Lazy<HashMap<&'static str, RGB>> = Lazy::new(|| {
    HashMap::from([
        ("bug", (166, 185, 26)),
        ("dark", (112, 87, 70)),
        ("dragon", (111, 53, 252)),
        ("electric", (247, 208, 44)),
        ("fairy", (214, 133, 173)),
        ("fighting", (194, 46, 40)),
        ("fire", (238, 129, 48)),
        ("flying", (169, 143, 243)),
        ("ghost", (115, 87, 151)),
        ("grass", (122, 199, 76)),
        ("ground", (226, 191, 101)),
        ("ice", (150, 217, 214)),
        ("normal", (168, 167, 122)),
        ("poison", (163, 62, 161)),
        ("psychic", (249, 85, 135)),
        ("rock", (182, 161, 54)),
        ("steel", (183, 183, 206)),
        ("water", (99, 144, 240)),
    ])
});

pub fn fetch(type_name: &str) -> String {
    match TYPE_NAME_TO_RGB.get(type_name) {
        Some(&(r, g, b)) => format!("{}", type_name.white().bold().on_truecolor(r, g, b)),
        None => type_name.to_owned(),
    }
}
