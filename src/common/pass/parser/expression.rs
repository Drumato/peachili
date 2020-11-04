// use crate::common::analyze_resource::ast;
use nom::{
    bytes::complete::take_while1,
    character::complete::char as parse_char,
    // multi::separated_list0,
    sequence::preceded,
};

/// [a-z]+
fn identifier(i: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    take_while1(nom::character::is_alphabetic)(i)
}

/// "[0-9]+"
fn integer_literal(i: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    take_while1(nom::character::is_digit)(i)
}
/// "'u' [0-9]+"
fn unsigned_integer_literal(i: &[u8]) -> nom::IResult<&[u8], &[u8]> {
    preceded(parse_char('u'), integer_literal)(i)
}

#[cfg(test)]
mod expression_parser_test {
    use super::*;

    #[test]
    fn identifier_with_valid_input() {
        let result = identifier(b"drumato;");
        assert_eq!(Ok((&b";"[..], &b"drumato"[..])), result);
    }

    #[test]
    fn unsigned_integer_literal_with_valid_input() {
        let result = unsigned_integer_literal(b"u300;");
        assert_eq!(Ok((&b";"[..], &b"300"[..])), result);
    }

    #[test]
    fn unsigned_integer_literal_with_invalid_input() {
        let result = unsigned_integer_literal(b"100;");
        assert!(result.is_err());
    }

    #[test]
    fn integer_literal_with_valid_input() {
        let result = integer_literal(b"300;");
        assert_eq!(Ok((&b";"[..], &b"300"[..])), result);
    }

    #[test]
    fn integer_literal_with_invalid_input() {
        let result = integer_literal(b"abc;");
        assert!(result.is_err());
    }
}
