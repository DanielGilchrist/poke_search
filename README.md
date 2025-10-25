# poke_search
<img width="800" alt="image" src="https://github.com/user-attachments/assets/6e3c0ca9-e284-4e2f-ba54-2ae016d2212f">

# Installation
You will need Rust 1.89.0 installed. I recommend using [`rustup`](https://rustup.rs/)
```sh
# From the rustup.rs website
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

And then run
```sh
rustup default 1.89.0
```

Then

1. Clone the repository
2. Run `cargo build --release`

You now have a release build! This can be run with `./target/release/poke_search` assuming you're in the root directory of this repository

This project doesn't currently distrbute or release any binaries. For now I'd recommend using an alias if you intend on using this outside of the repo directly
```sh
alias poke_search="/link/to/poke_search/target/release/poke_search"
```

# Usage
See the `help` command for a list of commands that can be run
```sh
❯ poke_search help
Search for pokemon information from the command line

Usage: poke_search <COMMAND>

Commands:
  ability  See information about an ability
  item     See information about an item
  moves    See moves for a pokemon
  move     See information about a move
  pokemon  See information about a pokemon
  type     See information about a specific type
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
