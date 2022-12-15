use rustemon::{
  client::RustemonClient,
  error::Error,
  model::{
    moves::Move,
    pokemon::{
      Pokemon,
      PokemonMove
    },
  },
  pokemon::pokemon,
  Follow
};

use futures::{stream, StreamExt};

pub struct MoveCommand {
  client: RustemonClient,
  pokemon_name: String,
  type_name: Option<String>
}

impl MoveCommand {
  pub async fn execute(client: RustemonClient, pokemon_name: String, type_name: Option<String>) {
    MoveCommand {
      client,
      pokemon_name,
      type_name
    }._execute().await;
  }

  async fn _execute(&self) {
    let pokemon = match self.fetch_pokemon().await {
      Ok(pokemon) => pokemon,
      Err(_) => panic!("Pokemon doesn't exist!")
    };

    let moves = self.process_moves(self.fetch_moves(pokemon.moves).await);
    let move_output = self.build_output(moves);

    let pokemon_name = capitalise(&pokemon.name);
    println!("Pokemon: {}", pokemon_name);

    if is_present(&move_output) {
      println!("Moves:");
      println!("{}", move_output);
    } else {
      match &self.type_name {
        Some(type_name) => {
          println!("{} has no {} type moves", pokemon_name, capitalise(type_name));
        },
        None => ()
      };
    }
  }

  async fn fetch_pokemon(&self) -> Result<Pokemon, Error> {
    pokemon::get_by_name(&self.pokemon_name, &self.client).await
  }

  async fn fetch_moves(&self, pokemon_moves: Vec<PokemonMove>) -> Vec<Move> {
    stream::iter(pokemon_moves)
      .map(|move_resource| {
        let client_ref = &self.client;

        async move {
          move_resource.move_.follow(client_ref).await.unwrap()
        }
      })
      .buffer_unordered(100)
      .collect::<Vec<_>>()
      .await
  }

  fn process_moves(&self, moves: Vec<Move>) -> Vec<Move> {
    let mut filtered_moves = match &self.type_name {
      Some(type_name) => {
        moves
          .into_iter()
          .filter_map(|move_| {
            if &move_.type_.name == type_name { Some(move_) } else { None }
          })
          .collect::<Vec<_>>()
      },
      None => moves
    };

    filtered_moves.sort_by_key(|move_| move_.power);
    filtered_moves.reverse();

    filtered_moves
  }

  fn build_output(&self, moves: Vec<Move>) -> String {
    moves
      .into_iter()
      .fold(String::new(), |mut output, move_| {
        let formatted_name = move_
          .name
          .split("-")
          .into_iter()
          .map(|str| capitalise(str))
          .collect::<Vec<_>>()
          .join(" ");

        output.push_str(format("Name", &formatted_name).as_str());
        output.push_str(format("Type", &move_.type_.name).as_str());
        output.push_str(format("Damage Type", &move_.damage_class.name).as_str());

        let power = parse_maybe_i64(move_.power);
        output.push_str(format("Power", &power).as_str());
        output.push_str(format("Accuracy", &parse_maybe_i64(move_.accuracy)).as_str());
        output.push_str(format("PP", &parse_maybe_i64(move_.pp)).as_str());

        let flavour_text = move_.flavor_text_entries.into_iter().find_map(|entry| {
          if entry.language.name == "en" {
            Some(entry.flavor_text)
          } else {
            None
          }
        }).unwrap().replace("\n", " ");

        output.push_str(format("Description", &flavour_text).as_str());

        let effect_chance = format!("{}%", parse_maybe_i64(move_.effect_chance));
        move_.effect_entries.into_iter().for_each(|entry| {
          let description = if power == "-" {
            entry.effect.replace("\n", " ").replace("  ", " ")
          } else {
            entry.short_effect.replace("$effect_chance%", &effect_chance)
          };

          output.push_str(format("Effect", &description).as_str());
        });

        output.push_str("\n\n");

        output
      })
  }
}

pub fn capitalise(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}

pub fn format(title: &str, value: &str) -> String {
  format!("  {}{}{}\n", title, ": ", capitalise(value))
}

pub fn parse_maybe_i64(value: Option<i64>) -> String {
  match value {
    Some(value) => value.to_string(),
    None => String::from("-")
  }
}

pub fn is_blank(str: &str) -> bool {
  str.replace("\n", "").replace(" ", "").len() == 0
}

pub fn is_present(str: &str) -> bool {
  !is_blank(str)
}
