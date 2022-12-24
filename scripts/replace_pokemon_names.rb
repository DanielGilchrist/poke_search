# frozen_string_literal: true
#
# Used to fetch and replace pokemon names for src/name_matcher/pokemon_names.rs from the PokeAPI repo
#
# requires:
# gem install httparty
#

require "csv"
require "httparty"

URL = "https://raw.githubusercontent.com/PokeAPI/pokeapi/master/data/v2/csv/pokemon.csv"
FILE_LOCATION = File.expand_path("../src/name_matcher/pokemon_names.rs", __dir__)

parsed_response = HTTParty.get(URL).parsed_response
csv = CSV.parse(parsed_response)
pokemon_names = csv.map { |line| line.fetch(1) }.sort

joined_names = pokemon_names.map do |name|
  "        String::from(\"#{name}\"),\n"
end
  .join
  .rstrip

file_contents = <<~CONTENT
use once_cell::sync::Lazy;

pub static POKEMON_NAMES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
#{joined_names}
    ]
});


CONTENT

File.open(FILE_LOCATION, "w+") do |file|
  file.write(file_contents)
end

puts "Successfully saved pokemon names to #{FILE_LOCATION}"
