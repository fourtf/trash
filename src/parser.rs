use nom::character::complete::alpha1;
use nom::character::complete::char;
use nom::character::complete::{none_of, space0, space1};
use nom::combinator::opt;
use nom::combinator::{map, recognize, value};
use nom::sequence::preceded;
use nom::IResult;
use nom::{alt, char, escaped_transform, is_a, many0, map, named, tag, take};
use nom::{branch::alt, multi::many0, sequence::pair};
use nom::{
    bytes::complete::{escaped_transform, is_a, is_not, tag, take},
    combinator::peek,
};
use nom::{character::complete::alpha0, combinator::recognizec};
use std::str;

#[derive(Debug, PartialEq, Eq)]
pub struct Call {
    pub program: String,
    pub args: Vec<String>,
}

pub enum Token<'a> {
    Space(&'a str),
    FunctionCall(&'a str),
    Argument(&'a str),
    // temporary
    Extra(&'a str),
}

// escapes the characters after `\` in a string
fn escape(input: &str) -> IResult<&str, &str> {
    alt((
        tag("\\"),
        tag("\""),
        value("\n", tag("n")),
        value("\r", tag("r")),
    ))(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;
    let (input, res) = opt(escaped_transform(none_of("\\\""), '\\', escape))(input)?;
    let (input, _) = char('"')(input)?;

    Ok((input, res.unwrap_or_else(|| String::new())))
}

fn parse_literal(input: &str) -> IResult<&str, &str> {
    is_not(" \t\n\r\"|&<>#()[]")(input)
}

// parse a random concatination of literal and escaped strings
// e.g. asdf"asdf $asdf \n"
// fn parse_token() -> IResult<&str, String> {}

// for now it's string or literal
fn parse_token(input: &str) -> IResult<&str, String> {
    alt((parse_string, map(parse_literal, |x| x.to_owned())))(input)
}

// program arg arg arg ...
pub fn parse_program_call(input: &str) -> IResult<&str, Call> {
    let (input, program) = parse_token(input)?;
    let (input, args) = many0(preceded(space1, parse_token))(input)?;

    Ok((input, Call { program, args }))
}

fn tokenize_program_call_<'a>(input: &'a str, tokens: &mut Vec<Token<'a>>) -> IResult<&'a str, ()> {
    let (input, token) = recognize(parse_token)(input)?;
    tokens.push(Token::FunctionCall(token));

    let (input, args) = many0(pair(recognize(space1), recognize(parse_token)))(input)?;

    for (space, arg) in args {
        tokens.push(Token::Space(space));
        tokens.push(Token::Argument(arg));
    }

    Ok((input, ()))
}

// tokens used to display in terminal
pub fn tokenize_program_call<'a>(input: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::<Token<'a>>::new();

    let input = tokenize_program_call_(input, &mut tokens)
        .ok()
        .map_or(input, |x| x.0);

    tokens.push(Token::Extra(input));

    tokens
}

// fn parse_script(input: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string("\"asdf\""), Ok(("", "asdf".to_owned())));
    }

    #[test]
    fn test_parse_escaped_string() {
        assert_eq!(parse_string("\"a\\\"sdf\""), Ok(("", "a\"sdf".to_owned())));
        assert_eq!(parse_string("\"\""), Ok(("", "".to_owned())));
        assert_eq!(
            parse_string("\"as\\nd\\\"f\""),
            Ok(("", "as\nd\"f".to_owned()))
        );
    }

    #[test]
    fn test_parse_literal() {
        assert_eq!(
            parse_literal("asdf_asdf_xD1423"),
            Ok(("", "asdf_asdf_xD1423"))
        );
        assert_eq!(parse_literal("asdf "), Ok((" ", "asdf")));
        assert_eq!(parse_literal("«»¥×¥’ "), Ok((" ", "«»¥×¥’")));

        assert!(parse_literal(" ").is_err());
    }

    #[test]
    fn test_parse_call() {
        assert_eq!(
            parse_program_call("ls ."),
            Ok((
                "",
                Call {
                    program: "ls".to_owned(),
                    args: vec![".".to_owned()]
                }
            ))
        );

        assert_eq!(
            parse_program_call("ls . | grep a"),
            Ok((
                " | grep a",
                Call {
                    program: "ls".to_owned(),
                    args: vec![".".to_owned()]
                }
            ))
        );

        assert_eq!(
            parse_program_call("ls . x d \"as\\nd\\\"f\" | grep a"),
            Ok((
                " | grep a",
                Call {
                    program: "ls".to_owned(),
                    args: vec![
                        ".".to_owned(),
                        "x".to_owned(),
                        "d".to_owned(),
                        "as\nd\"f".to_owned()
                    ]
                }
            ))
        );
    }
}
