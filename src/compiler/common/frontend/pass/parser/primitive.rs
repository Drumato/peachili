use super::Parser;

use nom::bytes::complete::{take_while, take_while1};
use nom::character::complete::multispace0;
use nom::sequence::{delimited, tuple};
use nom::{bytes::complete::tag, multi::separated_list0};

pub fn keyword(pattern: &str) -> impl FnMut(&str) -> nom::IResult<&str, &str> + '_ {
    move |i: &str| ws(tag(pattern))(i)
}

pub fn symbol(pattern: &str) -> impl FnMut(&str) -> nom::IResult<&str, &str> + '_ {
    move |i: &str| ws(tag(pattern))(i)
}

pub fn ws<'a, P>(parser: P) -> impl FnMut(&'a str) -> nom::IResult<&'a str, &'a str>
where
    P: FnMut(&'a str) -> nom::IResult<&'a str, &'a str>,
{
    delimited(multispace0, parser, multispace0)
}

impl<'a> Parser<'a> {
    /// [a-zA-Z] ('_' | [a-zA-Z0-9])*
    pub fn identifier_string(&'a self) -> impl Fn(&'a str) -> nom::IResult<&'a str, String> {
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

    pub fn list_structure<T, F>(
        &'a self,
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
            };

            delimited(
                symbol(start),
                separated_list0(symbol(separator), &sub_parser),
                symbol(end),
            )(i)
        }
    }
}

pub enum Delimiter {
    Paren,
}
#[cfg(test)]
mod primitive_tests {
    use super::*;

    #[test]
    fn primitive_test_main() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);

        let _ = list_structure_test(&parser, "(+, +, +, +);", ";", |i: &str| symbol("+")(i));
        let _ = list_structure_test(&parser, "(+++, +++, +++, +++);", ";", |i: &str| {
            symbol("+++")(i)
        });
    }

    fn list_structure_test<'a, T, F>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
        sub_parser: F,
    ) -> Vec<T>
    where
        F: Fn(&'a str) -> nom::IResult<&'a str, T>,
    {
        let result = parser.list_structure(Delimiter::Paren, ",", sub_parser)(input);
        assert!(result.is_ok());

        let (r, list) = result.unwrap();

        assert_eq!(rest, r);

        list
    }
}
