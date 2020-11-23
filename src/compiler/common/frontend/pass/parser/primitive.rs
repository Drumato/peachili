use super::Parser;

use nom::bytes::complete::tag;
use nom::bytes::complete::{take_while, take_while1};
use nom::character::complete::multispace0;
use nom::sequence::{delimited, tuple};

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
}
