use crate::{name_matcher::matcher, roman_numeral::integer_to_roman};

pub fn parse_name(name: &str) -> String {
    name.to_lowercase().split(' ').collect::<Vec<_>>().join("-")
}

pub fn parse_generation(generation_name: &str) -> Result<String, String> {
    let normalised = parse_name(generation_name);

    let stripped = if let Some(stripped_generation) = normalised.strip_prefix("generation-") {
        stripped_generation.to_owned()
    } else if let Some(stripped_gen) = normalised.strip_prefix("gen-") {
        stripped_gen.to_owned()
    } else {
        normalised
    };

    let roman = if let Ok(num) = stripped.parse::<i32>() {
        integer_to_roman(num)
    } else {
        stripped
    };

    let generation_id = format!("generation-{}", roman);

    if matcher::is_valid(&generation_id, matcher::MatcherType::Generation) {
        Ok(generation_id)
    } else {
        Err(format!(
            "'{}' isn't a valid pokemon generation",
            generation_name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name() {
        assert_eq!(parse_name("Charizard"), "charizard");
        assert_eq!(parse_name("Solar Power"), "solar-power");
        assert_eq!(parse_name("Mr. Mime"), "mr.-mime");
    }

    #[test]
    fn test_parse_generation() {
        assert_eq!(
            parse_generation("Generation IV"),
            Ok(String::from("generation-iv"))
        );
        assert_eq!(
            parse_generation("Gen IV"),
            Ok(String::from("generation-iv"))
        );
        assert_eq!(
            parse_generation("generation iv"),
            Ok(String::from("generation-iv"))
        );
        assert_eq!(parse_generation("4"), Ok(String::from("generation-iv")));
        assert_eq!(
            parse_generation("generation 4"),
            Ok(String::from("generation-iv"))
        );
        assert_eq!(parse_generation("gen 4"), Ok(String::from("generation-iv")));

        assert_eq!(
            parse_generation("What"),
            Err(String::from("'What' isn't a valid pokemon generation"))
        );
        assert_eq!(
            parse_generation("1234"),
            Err(String::from("'1234' isn't a valid pokemon generation"))
        );
        assert_eq!(
            parse_generation("Generation 123"),
            Err(String::from(
                "'Generation 123' isn't a valid pokemon generation"
            ))
        );
    }
}
