Scripts are run using [rust-script](https://rust-script.org/)

Have a look [here](https://rust-script.org/#installation) for instructions on installation and usage

# `scripts/replace_pokemon_names.rs`
```sh
rust-script scripts/replace_pokemon_names.rs
```

This is used to populate names for `name_matcher` to suggest pokemon, moves, etc. when entered incorrectly with a command.

The script fetches names from the Poke API repo and populates `<type>_names.rs` with a `LazyLock<Vec<String>>` so the names are only initialised when used.
