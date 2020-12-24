use nom::bytes::complete::{take_while, take_while1};
use nom::character::complete::multispace0;
use nom::sequence::{delimited, tuple};
use nom::{bytes::complete::tag, multi::separated_list0};

pub fn keyword(pattern: &str) -> impl Fn(&str) -> nom::IResult<&str, &str> + '_ {
    move |i: &str| ws(tag(pattern))(i)
}

pub fn symbol(pattern: &str) -> impl Fn(&str) -> nom::IResult<&str, &str> + '_ {
    move |i: &str| ws(tag(pattern))(i)
}

pub fn string_literal_str() -> impl Fn(&str) -> nom::IResult<&str, &str> {
    move |i: &str| {
        let (rest, contents) = ws(delimited(
            symbol("\""),
            take_while(|b: char| b != '"'),
            symbol("\""),
        ))(i)?;

        Ok((rest, contents))
    }
}

pub fn ws<'a, P>(parser: P) -> impl FnMut(&'a str) -> nom::IResult<&'a str, &'a str>
where
    P: FnMut(&'a str) -> nom::IResult<&'a str, &'a str>,
{
    delimited(multispace0, parser, multispace0)
}

/// [a-zA-Z] ('_' | [a-zA-Z0-9])*
pub fn identifier_string<'a>() -> impl Fn(&'a str) -> nom::IResult<&'a str, String> {
    move |i: &str| {
        let (rest, (_, head, last, _)) = tuple((
            multispace0,
            take_while1(|b: char| b.is_alphabetic()),
            take_while(|b: char| b.is_alphanumeric() || b == '_'),
            multispace0,
        ))(i)?;
        Ok((rest, format!("{}{}", head, last)))
    }
}

pub fn list_structure<'a, T, F>(
    delimiter: Delimiter,
    separator: &'a str,
    sub_parser: F,
) -> impl Fn(&'a str) -> nom::IResult<&'a str, Vec<T>>
where
    F: Fn(&'a str) -> nom::IResult<&'a str, T>,
{
    move |i: &str| {
        let (start, end) = match delimiter {
            Delimiter::Paren => ("(", ")"),
            Delimiter::Bracket => ("{", "}"),
        };

        delimited(
            symbol(start),
            separated_list0(symbol(separator), &sub_parser),
            symbol(end),
        )(i)
    }
}

pub enum Delimiter {
    Paren,
    Bracket,
}
#[cfg(test)]
mod primitive_tests {
    use super::*;

    #[test]
    fn primitive_test_main() {
        let _ = string_literal_str_test("\"movq $60, %rax\";", ";");
        let _ = list_structure_test("(+, +, +, +);", ";", |i: &str| symbol("+")(i));
        let _ = list_structure_test("(+++, +++, +++, +++);", ";", |i: &str| symbol("+++")(i));
        let _ = identifier_string_with_invalid_input("100yen;");
        let _ = identifier_string_with_invalid_input("!foafekajl;");
    }

    fn list_structure_test<'a, T, F>(input: &'a str, rest: &'a str, sub_parser: F) -> Vec<T>
    where
        F: Fn(&'a str) -> nom::IResult<&'a str, T>,
    {
        let result = list_structure(Delimiter::Paren, ",", sub_parser)(input);
        assert!(result.is_ok());

        let (r, list) = result.unwrap();

        assert_eq!(rest, r);

        list
    }

    fn string_literal_str_test<'a>(input: &'a str, rest: &'a str) {
        let result = string_literal_str()(input);
        assert!(result.is_ok());

        let (r, _) = result.unwrap();
        assert_eq!(rest, r);
    }

    fn identifier_string_with_invalid_input<'a>(input: &'a str) {
        let result = identifier_string()(input);
        assert!(result.is_err());
    }
}
