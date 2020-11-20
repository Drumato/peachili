use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::sequence::delimited;
pub fn keyword<'a>(pattern: &'a str) -> impl FnMut(&'a str) -> nom::IResult<&'a str, &'a str> {
    move |i: &str| ws(tag(pattern))(i)
}

pub fn symbol<'a>(pattern: &'a str) -> impl FnMut(&'a str) -> nom::IResult<&'a str, &'a str> {
    move |i: &str| ws(tag(pattern))(i)
}

pub fn ws<'a, P>(parser: P) -> impl FnMut(&'a str) -> nom::IResult<&'a str, &'a str>
where
    P: FnMut(&'a str) -> nom::IResult<&'a str, &'a str>,
{
    delimited(multispace0, parser, multispace0)
}
