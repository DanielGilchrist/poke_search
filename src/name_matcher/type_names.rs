use std::sync::LazyLock;

pub static TYPE_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        String::from("bug"),
        String::from("dark"),
        String::from("dragon"),
        String::from("electric"),
        String::from("fairy"),
        String::from("fighting"),
        String::from("fire"),
        String::from("flying"),
        String::from("ghost"),
        String::from("grass"),
        String::from("ground"),
        String::from("ice"),
        String::from("normal"),
        String::from("poison"),
        String::from("psychic"),
        String::from("rock"),
        String::from("shadow"),
        String::from("steel"),
        String::from("stellar"),
        String::from("unknown"),
        String::from("water"),
    ]
});
