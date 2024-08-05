use std::sync::LazyLock;

pub static MOVE_DAMAGE_CLASS_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        String::from("physical"),
        String::from("special"),
        String::from("status"),
    ]
});
