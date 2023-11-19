use once_cell::sync::Lazy;

pub static MOVE_DAMAGE_CLASS_NAMES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        String::from("physical"),
        String::from("special"),
        String::from("status"),
    ]
});
