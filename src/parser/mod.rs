use super::engine;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::anychar;
use nom::multi::many_till;
use nom::sequence::terminated;
use nom::IResult;
use nom::Parser;

#[derive(Debug)]
enum DialogParseError<T> {
    DialogBuildError(engine::DialogError),
    NomError(nom::Err<T>),
}

impl<T> From<engine::DialogError> for DialogParseError<T> {
    fn from(value: engine::DialogError) -> Self {
        DialogParseError::DialogBuildError(value)
    }
}

impl<T> From<nom::Err<T>> for DialogParseError<T> {
    fn from(value: nom::Err<T>) -> Self {
        DialogParseError::NomError(value)
    }
}

fn parse<'s>(input: &'s str) -> Result<engine::Dialog, DialogParseError<nom::error::Error<&str>>> {
    let (input, _) = tag("Name:")(input)?;
    let (input, _) = take_while(is_whitespace)(input)?;
    let (input, node_name) = identifier(input)?;
    let (input, _) = take_while(is_whitespace)(input)?;
    let (input, (text, _)) = many_till(anychar, |inp| {
        alt((tag("\r\n---\r\n"), tag("\n---\n"))).parse(inp)
    })
    .parse(input)?;
    let text: String = text.iter().collect();
    Ok(engine::DialogBuilder::new(engine::DialogNode::new(node_name, text)).build()?)
}

fn identifier<'s>(input: &'s str) -> IResult<&'s str, &'s str> {
    take_while(is_alphanumeric)(input)
}

fn is_alphanumeric(ch: char) -> bool {
    match ch {
        'A'..='Z' => true,
        'a'..='z' => true,
        '0'..='9' => true,
        _ => false,
    }
}

fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_dialog_step() {
        let parse_result = parse(
            r#"Name: Start
Hello, World!
---
1. Hi!
===

Name: End
Goodbye!
==="#,
        );
        assert!(parse_result.is_ok());
        let dialog = parse_result.unwrap();
        assert_eq!(dialog.start_node().id(), &"Start".into());
        assert_eq!(dialog.start_node().text().as_plain_str(), "Hello, World!");
    }
}
