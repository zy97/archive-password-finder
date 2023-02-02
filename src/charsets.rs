use parse_display::{Display, FromStr};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Display, FromStr)]
#[display(style = "snake_case")]
pub enum CharsetChoice {
    Number,
    Lower,
    Upper,
    Special,
}
impl CharsetChoice {
    pub fn to_charset(self) -> Vec<char> {
        match self {
            CharsetChoice::Number => charset_digits(),
            CharsetChoice::Lower => charset_lowercase_letters(),
            CharsetChoice::Upper => charset_uppercase_letters(),
            CharsetChoice::Special => charset_punctuations(),
        }
    }
    pub fn to_string() -> String {
        format!(
            "{},{},{},{}",
            CharsetChoice::Number,
            CharsetChoice::Lower,
            CharsetChoice::Upper,
            CharsetChoice::Special
        )
    }
}
fn charset_lowercase_letters() -> Vec<char> {
    vec![
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ]
}

fn charset_uppercase_letters() -> Vec<char> {
    vec![
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ]
}

fn charset_digits() -> Vec<char> {
    vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
}

fn charset_punctuations() -> Vec<char> {
    vec![
        ' ', '-', '=', '!', '@', '#', '$', '%', '^', '&', '*', '_', '+', '<', '>', '/', '?', '.',
        ';', ':', '{', '}',
    ]
}
#[cfg(test)]
mod test {
    use super::CharsetChoice;

    #[test]
    fn test1() {
        let number = CharsetChoice::Number;
        assert_eq!(number.to_string(), "number");
        let number_str = String::from("number");
        assert_eq!(number_str.parse(), Ok(number));
        println!(
            "charset_number_to_string = {:?}",
            CharsetChoice::to_string()
        );
    }
}
