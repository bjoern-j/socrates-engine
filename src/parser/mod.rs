use super::engine;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::anychar;
use nom::multi::{many0, many_till, separated_list0};
use nom::sequence::delimited;
use nom::IResult;
use nom::Parser;

#[derive(PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Separator {
    Link,
    Node,
}

fn parse<'s>(input: &'s str) -> Result<engine::Dialog, DialogParseError<nom::error::Error<&str>>> {
    let (input, node) = parse_node(input)?;
    let (remainder, nodes) = many0(parse_node).parse(input)?;
    println!("{}", remainder);
    let mut builder = engine::DialogBuilder::new(node);
    for node in nodes {
        builder = builder.add_node(node);
    }
    Ok(builder.build()?)
}

fn parse_node<'s>(input: &'s str) -> IResult<&'s str, engine::DialogNode> {
    let (input, _) = trim(input)?;
    let (input, _) = tag("Name:")(input)?;
    let (input, _) = trim(input)?;
    let (input, node_name) = identifier(input)?;
    let (input, _) = trim(input)?;
    let (input, (text, separator)) =
        many_till(anychar, alt((link_separator, node_separator))).parse(input)?;
    let text: String = text.iter().collect();
    if separator == Separator::Link {
        let (input, choices) =
            separated_list0(tag("|"), take_while(|ch| ch != '|' && ch != '=')).parse(input)?;
        let mut links = Vec::new();
        for choice in choices {
            links.push(parse_link(node_name, choice)?.1);
        }
        let (input, _) = node_separator(input)?;
        Ok((
            input,
            engine::DialogNode::new_with_links(node_name, text, links),
        ))
    } else {
        Ok((input, engine::DialogNode::new(node_name, text)))
    }
}

fn parse_link<'s>(parent: &'s str, input: &'s str) -> IResult<&'s str, engine::DialogLink> {
    let (input, _) = trim(input)?;
    let (input, target) = delimited(tag("{"), identifier, tag("}")).parse(input)?;
    let text = input.trim();
    Ok((
        "",
        engine::DialogLink::new(parent, target, text, engine::DialogLinkCondition::None),
    ))
}

fn link_separator<'s>(input: &'s str) -> IResult<&'s str, Separator> {
    let (input, _) = alt((tag("\r\n---\r\n"), tag("\n---\n"))).parse(input)?;
    Ok((input, Separator::Link))
}

fn node_separator<'s>(input: &'s str) -> IResult<&'s str, Separator> {
    let (input, _) = alt((tag("===\r\n"), tag("===\n"))).parse(input)?;
    Ok((input, Separator::Node))
}

fn identifier<'s>(input: &'s str) -> IResult<&'s str, &'s str> {
    take_while(is_alphanumeric)(input)
}

fn trim<'s>(input: &'s str) -> IResult<&'s str, ()> {
    let (input, _) = take_while(is_whitespace)(input)?;
    Ok((input, ()))
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
{End} Hi! | {End} Bye!
===

Name: End
Goodbye!
===
"#,
        );
        assert!(parse_result.is_ok(), "{:?}", parse_result);
        let dialog = parse_result.unwrap();
        assert_eq!(dialog.start_node().id(), &"Start".into());
        assert_eq!(dialog.start_node().text().as_plain_str(), "Hello, World!");
        let links = dialog.start_node().links();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text().as_plain_str(), "Hi!");
        assert_eq!(links[0].from(), &"Start".into());
        assert_eq!(links[0].to(), &"End".into());
        assert_eq!(links[1].text().as_plain_str(), "Bye!");
        assert_eq!(links[1].from(), &"Start".into());
        assert_eq!(links[1].to(), &"End".into());
        let end_node = dialog.get_node(&"End".into());
        assert_eq!(end_node.links().len(), 0);
    }

    #[test]
    fn error_in_links() {
        let parse_result = parse(
            r#"Name: Start
Hello, World!
---
{En} Hi!
===

Name: End
Bye!
===
"#,
        );
        assert!(parse_result.is_err());
        if let Err(error) = parse_result {
            assert_eq!(
                DialogParseError::DialogBuildError(engine::DialogError::InvalidLink(
                    engine::LinkErrorInfo {
                        missing_source: None,
                        missing_target: Some("En".into())
                    }
                )),
                error
            );
        }
    }
}
