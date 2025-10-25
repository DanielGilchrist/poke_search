use std::sync::LazyLock;

pub static GENERATION_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        String::from("generation-i"),
        String::from("generation-ii"),
        String::from("generation-iii"),
        String::from("generation-iv"),
        String::from("generation-ix"),
        String::from("generation-v"),
        String::from("generation-vi"),
        String::from("generation-vii"),
        String::from("generation-viii"),
    ]
});
