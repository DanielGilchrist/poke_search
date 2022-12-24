# frozen_string_literal: true
#
# Used to fetch and replace names for src/name_matcher/<resource>_names.rs from the PokeAPI repo
# so that we can match on them for suggestions
#
# requires:
# gem install httparty
#

require "csv"
require "httparty"

def fetch_and_replace(url, file_name)
  parsed_response = HTTParty.get(url).parsed_response
  csv = CSV.parse(parsed_response)
  names = csv.map { |line| line.fetch(1) }.sort

  joined_names = names.map do |name|
    "        String::from(\"#{name}\"),\n"
  end
    .join
    .rstrip

  file_contents = <<~CONTENT
  use once_cell::sync::Lazy;

  pub static #{file_name.upcase}: Lazy<Vec<String>> = Lazy::new(|| {
      vec![
  #{joined_names}
      ]
  });

  CONTENT

  absolute_path = File.expand_path("../src/name_matcher/#{file_name}.rs", __dir__)
  File.open(absolute_path, "w+") do |file|
    file.write(file_contents)
  end

  puts "Successfully saved #{file_name.split("_").join(" ")} to #{absolute_path}"
end

[
  [
    "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/pokemon.csv",
    "pokemon_names"
  ],
  [
    "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/moves.csv",
    "move_names"
  ]
].each do |(url, file_name)|
  fetch_and_replace(url, file_name)
end
