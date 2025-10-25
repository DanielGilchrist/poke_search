use std::sync::LazyLock;

static VALUES: LazyLock<Vec<(i32, &'static str)>> = LazyLock::new(|| {
    vec![
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ]
});

pub fn integer_to_roman(mut num: i32) -> String {
    let mut result = String::new();

    for (value, numeral) in VALUES.iter() {
        while num >= *value {
            result.push_str(numeral);
            num -= value;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_numerals() {
        assert_eq!(integer_to_roman(1), "i");
        assert_eq!(integer_to_roman(5), "v");
        assert_eq!(integer_to_roman(10), "x");
        assert_eq!(integer_to_roman(50), "l");
        assert_eq!(integer_to_roman(100), "c");
        assert_eq!(integer_to_roman(500), "d");
        assert_eq!(integer_to_roman(1000), "m");
    }

    #[test]
    fn test_additive_cases() {
        assert_eq!(integer_to_roman(2), "ii");
        assert_eq!(integer_to_roman(3), "iii");
        assert_eq!(integer_to_roman(6), "vi");
        assert_eq!(integer_to_roman(7), "vii");
        assert_eq!(integer_to_roman(8), "viii");
        assert_eq!(integer_to_roman(11), "xi");
        assert_eq!(integer_to_roman(12), "xii");
        assert_eq!(integer_to_roman(15), "xv");
        assert_eq!(integer_to_roman(20), "xx");
        assert_eq!(integer_to_roman(30), "xxx");
    }

    #[test]
    fn test_subtractive_cases() {
        assert_eq!(integer_to_roman(4), "iv");
        assert_eq!(integer_to_roman(9), "ix");
        assert_eq!(integer_to_roman(40), "xl");
        assert_eq!(integer_to_roman(90), "xc");
        assert_eq!(integer_to_roman(400), "cd");
        assert_eq!(integer_to_roman(900), "cm");
    }

    #[test]
    fn test_complex_numerals() {
        assert_eq!(integer_to_roman(1990), "mcmxc");
        assert_eq!(integer_to_roman(2014), "mmxiv");
        assert_eq!(integer_to_roman(2024), "mmxxiv");
        assert_eq!(integer_to_roman(1994), "mcmxciv");
        assert_eq!(integer_to_roman(3888), "mmmdccclxxxviii");
    }
}
