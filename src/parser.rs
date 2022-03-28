use crate::entity::Markdown;
use crate::entity::MarkdownInline;
use crate::entity::MarkdownText;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_until, take_while1},
    character::complete::alphanumeric0,
    character::complete::line_ending,
    character::is_digit,
    combinator::{map, not},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

pub fn parse_markdown(i: &str) -> IResult<&str, Vec<Markdown>> {
    many1(alt((
        map(parse_horizontal_rule, |_| Markdown::HorizontalRule),
        map(parse_header, |e| Markdown::Heading(e.0, e.1)),
        map(parse_unordered_list, |e| Markdown::UnorderedList(e)),
        map(parse_ordered_list, |e| Markdown::OrderedList(e)),
        map(parse_code_block, |(lang, code)| {
            Markdown::Codeblock(lang.to_string(), code.to_string())
        }),
        map(parse_markdown_text, |e| Markdown::Line(e)),
    )))(i)
}

fn parse_horizontal_rule(i: &str) -> IResult<&str, &str> {
    preceded(tag("---"), line_ending)(i)
}

fn parse_boldtext(i: &str) -> IResult<&str, &str> {
    delimited(tag("**"), is_not("**"), tag("**"))(i)
}

fn parse_italics(i: &str) -> IResult<&str, &str> {
    delimited(tag("*"), is_not("*"), tag("*"))(i)
}

fn parse_strike(i: &str) -> IResult<&str, &str> {
    delimited(tag("~"), is_not("~"), tag("~"))(i)
}

fn parse_inline_code(i: &str) -> IResult<&str, &str> {
    delimited(tag("`"), is_not("`"), tag("`"))(i)
}

fn parse_link(i: &str) -> IResult<&str, (&str, &str)> {
    pair(
        delimited(tag("["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(i)
}

fn parse_image(i: &str) -> IResult<&str, (&str, &str)> {
    pair(
        delimited(tag("!["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(i)
}

// we want to match many things that are not any of our specail tags
// but since we have no tools available to match and consume in the negative case (without regex)
// we need to match against our tags, then consume one char
// we repeat this until we run into one of our special characters
// then we join our array of characters into a String
fn parse_plaintext(i: &str) -> IResult<&str, String> {
    let safe_one_char = preceded(
        not(alt((
            tag("*"),
            tag("`"),
            tag("~"),
            tag("["),
            tag("!["),
            tag("\\"),
            tag("\n"),
            tag("\r"),
        ))),
        take(1u8),
    );
    let escaped_char = map(
        alt((
            tag("\\*"),
            tag("\\`"),
            tag("\\["),
            tag("\\]"),
            tag("\\~"),
            tag("\\!"),
        )),
        |e: &str| &e[1..2],
    );

    map(many1(alt((safe_one_char, escaped_char))), |v| v.join(""))(i)
}

fn parse_markdown_inline(i: &str) -> IResult<&str, MarkdownInline> {
    alt((
        map(parse_italics, |s: &str| {
            MarkdownInline::Italic(s.to_string())
        }),
        map(parse_strike, |s: &str| {
            MarkdownInline::Strike(s.to_string())
        }),
        map(parse_inline_code, |s: &str| {
            MarkdownInline::InlineCode(s.to_string())
        }),
        map(parse_boldtext, |s: &str| {
            MarkdownInline::Bold(s.to_string())
        }),
        map(parse_image, |(tag, url): (&str, &str)| {
            MarkdownInline::Image(tag.to_string(), url.to_string())
        }),
        map(parse_link, |(tag, url): (&str, &str)| {
            MarkdownInline::Link(tag.to_string(), url.to_string())
        }),
        map(parse_plaintext, |s| MarkdownInline::Plaintext(s)),
    ))(i)
}

fn parse_markdown_text(i: &str) -> IResult<&str, MarkdownText> {
    terminated(many0(parse_markdown_inline), tag("\n"))(i)
}

// this guy matches the literal character #
fn parse_header_tag(i: &str) -> IResult<&str, usize> {
    map(
        terminated(take_while1(|c| c == '#'), tag(" ")),
        |s: &str| s.len(),
    )(i)
}

// this combines a tuple of the header tag and the rest of the line
fn parse_header(i: &str) -> IResult<&str, (usize, MarkdownText)> {
    tuple((parse_header_tag, parse_markdown_text))(i)
}

fn parse_unordered_list_tag(i: &str) -> IResult<&str, &str> {
    terminated(tag("-"), tag(" "))(i)
}

fn parse_unordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
    preceded(parse_unordered_list_tag, parse_markdown_text)(i)
}

fn parse_unordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
    many1(parse_unordered_list_element)(i)
}

fn parse_ordered_list_tag(i: &str) -> IResult<&str, &str> {
    terminated(
        terminated(take_while1(|d| is_digit(d as u8)), tag(".")),
        tag(" "),
    )(i)
}

fn parse_ordered_list_element(i: &str) -> IResult<&str, MarkdownText> {
    preceded(parse_ordered_list_tag, parse_markdown_text)(i)
}

fn parse_ordered_list(i: &str) -> IResult<&str, Vec<MarkdownText>> {
    many1(parse_ordered_list_element)(i)
}

fn parse_code_block(i: &str) -> IResult<&str, (&str, &str)> {
    let f = tuple((
        tag("```"),
        alphanumeric0,
        line_ending,
        take_until("```"),
        tag("```"),
    ));
    map(f, |(_, language, _, code, _)| (language, code))(i)
}

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use nom::error::ErrorKind;

    macro_rules! err {
        ($x:expr, $y:expr) => {
            Err(nom::Err::Error(nom::error::Error::new($x, $y)))
        };
    }

    #[test]
    fn test_parse_italics() {
        assert_eq!(
            parse_italics("*here is italic*"),
            Ok(("", "here is italic"))
        );
        assert_eq!(parse_italics("*here is italic"), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_italics("here is italic*"),
            err!("here is italic*", ErrorKind::Tag)
        );
        assert_eq!(
            parse_italics("here is italic"),
            err!("here is italic", ErrorKind::Tag)
        );
        assert_eq!(parse_italics("*"), err!("", ErrorKind::IsNot));
        assert_eq!(parse_italics("**"), err!("*", ErrorKind::IsNot));
        assert_eq!(parse_italics(""), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_italics("**we are doing bold**"),
            err!("*we are doing bold**", ErrorKind::IsNot)
        );
    }

    #[test]
    fn test_parse_boldtext() {
        assert_eq!(parse_boldtext("**here is bold**"), Ok(("", "here is bold")));
        assert_eq!(parse_boldtext("**here is bold"), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_boldtext("here is bold**"),
            err!("here is bold**", ErrorKind::Tag)
        );
        assert_eq!(
            parse_boldtext("here is bold"),
            err!("here is bold", ErrorKind::Tag)
        );
        assert_eq!(parse_boldtext("****"), err!("**", ErrorKind::IsNot));
        assert_eq!(parse_boldtext("**"), err!("", ErrorKind::IsNot));
        assert_eq!(parse_boldtext("*"), err!("*", ErrorKind::Tag));
        assert_eq!(parse_boldtext(""), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_boldtext("*this is italic*"),
            err!("*this is italic*", ErrorKind::Tag)
        );
    }

    #[test]
    fn test_parse_inline_code() {
        assert_eq!(
            parse_boldtext("**here is bold**\n"),
            Ok(("\n", "here is bold"))
        );
        assert_eq!(parse_inline_code("`here is code"), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_inline_code("here is code`"),
            err!("here is code`", ErrorKind::Tag)
        );
        assert_eq!(parse_inline_code("``"), err!("`", ErrorKind::IsNot));
        assert_eq!(parse_inline_code("`"), err!("", ErrorKind::IsNot));
        assert_eq!(parse_inline_code(""), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_link() {
        assert_eq!(
            parse_link("[title](https://www.example.com)"),
            Ok(("", ("title", "https://www.example.com")))
        );
        assert_eq!(parse_inline_code(""), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_image() {
        assert_eq!(
            parse_image("![alt text](image.jpg)"),
            Ok(("", ("alt text", "image.jpg")))
        );
        assert_eq!(parse_inline_code(""), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_plaintext() {
        assert_eq!(
            parse_plaintext("1234567890"),
            Ok(("", String::from("1234567890")))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!"),
            Ok(("", String::from("oh my gosh!")))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!["),
            Ok(("![", String::from("oh my gosh")))
        );
        assert_eq!(
            parse_plaintext("oh my gosh!*"),
            Ok(("*", String::from("oh my gosh!")))
        );
        assert_eq!(
            parse_plaintext("*bold babey bold*"),
            err!("*bold babey bold*", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("[link babey](and then somewhat)"),
            err!("[link babey](and then somewhat)", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("`codeblock for bums`"),
            err!("`codeblock for bums`", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("![ but wait theres more](jk)"),
            err!("![ but wait theres more](jk)", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("here is plaintext"),
            Ok(("", String::from("here is plaintext")))
        );
        assert_eq!(
            parse_plaintext("here is plaintext!"),
            Ok(("", String::from("here is plaintext!")))
        );
        assert_eq!(
            parse_plaintext("here is plaintext![image starting"),
            Ok(("![image starting", String::from("here is plaintext")))
        );
        assert_eq!(
            parse_plaintext("here is plaintext\n"),
            Ok(("\n", String::from("here is plaintext")))
        );
        assert_eq!(
            parse_plaintext("*here is italic*"),
            err!("*here is italic*", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("**here is bold**"),
            err!("**here is bold**", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("`here is code`"),
            err!("`here is code`", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("[title](https://www.example.com)"),
            err!("[title](https://www.example.com)", ErrorKind::Tag)
        );
        assert_eq!(
            parse_plaintext("![alt text](image.jpg)"),
            err!("![alt text](image.jpg)", ErrorKind::Tag)
        );
        assert_eq!(parse_plaintext(""), err!("", ErrorKind::Tag));
        assert_eq!(parse_plaintext("\\*\\[\\]"), Ok(("", String::from("*[]"))));
    }

    #[test]
    fn test_parse_markdown_inline() {
        assert_eq!(
            parse_markdown_inline("*here is italic*"),
            Ok(("", MarkdownInline::Italic(String::from("here is italic"))))
        );
        assert_eq!(
            parse_markdown_inline("**here is bold**"),
            Ok(("", MarkdownInline::Bold(String::from("here is bold"))))
        );
        assert_eq!(
            parse_markdown_inline("`here is code`"),
            Ok(("", MarkdownInline::InlineCode(String::from("here is code"))))
        );
        assert_eq!(
            parse_markdown_inline("[title](https://www.example.com)"),
            Ok((
                "",
                (MarkdownInline::Link(
                    String::from("title"),
                    String::from("https://www.example.com")
                ))
            ))
        );
        assert_eq!(
            parse_markdown_inline("![alt text](image.jpg)"),
            Ok((
                "",
                (MarkdownInline::Image(String::from("alt text"), String::from("image.jpg")))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is plaintext!"),
            Ok((
                "",
                MarkdownInline::Plaintext(String::from("here is plaintext!"))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext *but what if we italicize?"),
            Ok((
                "*but what if we italicize?",
                MarkdownInline::Plaintext(String::from("here is some plaintext "))
            ))
        );
        assert_eq!(
            parse_markdown_inline("here is some plaintext \n*but what if we italicize?"),
            Ok((
                "\n*but what if we italicize?",
                MarkdownInline::Plaintext(String::from("here is some plaintext "))
            ))
        );
        assert_eq!(parse_markdown_inline("\n"), err!("\n", ErrorKind::Tag));
        assert_eq!(parse_markdown_inline(""), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_markdown_text() {
        assert_eq!(parse_markdown_text("\n"), Ok(("", vec![])));
        assert_eq!(
            parse_markdown_text("here is some plaintext\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext(String::from(
                    "here is some plaintext"
                ))]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?* I guess it doesnt **matter** in my `code`\n"),
            Ok(("", vec![
                MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                MarkdownInline::Italic(String::from("but what if we italicize?")),
                MarkdownInline::Plaintext(String::from(" I guess it doesnt ")),
                MarkdownInline::Bold(String::from("matter")),
                MarkdownInline::Plaintext(String::from(" in my ")),
                MarkdownInline::InlineCode(String::from("code")),
            ]))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?*\n"),
            Ok((
                "",
                vec![
                    MarkdownInline::Plaintext(String::from("here is some plaintext ")),
                    MarkdownInline::Italic(String::from("but what if we italicize?")),
                ]
            ))
        );
        assert_eq!(
            parse_markdown_text("here is some plaintext *but what if we italicize?"),
            err!("*but what if we italicize?", ErrorKind::Tag)
        );
    }

    #[test]
    fn test_parse_header_tag() {
        assert_eq!(parse_header_tag("# "), Ok(("", 1)));
        assert_eq!(parse_header_tag("### "), Ok(("", 3)));
        assert_eq!(parse_header_tag("# h1"), Ok(("h1", 1)));
        assert_eq!(parse_header_tag("# h1"), Ok(("h1", 1)));
        assert_eq!(parse_header_tag(" "), err!(" ", ErrorKind::TakeWhile1));
        assert_eq!(parse_header_tag("#"), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_header() {
        assert_eq!(
            parse_header("# h1\n"),
            Ok(("", (1, vec![MarkdownInline::Plaintext(String::from("h1"))])))
        );
        assert_eq!(
            parse_header("## h2\n"),
            Ok(("", (2, vec![MarkdownInline::Plaintext(String::from("h2"))])))
        );
        assert_eq!(
            parse_header("###  h3\n"),
            Ok((
                "",
                (3, vec![MarkdownInline::Plaintext(String::from(" h3"))])
            ))
        );
        assert_eq!(parse_header("###h3"), err!("h3", ErrorKind::Tag));
        assert_eq!(parse_header("###"), err!("", ErrorKind::Tag));
        assert_eq!(parse_header(""), err!("", ErrorKind::TakeWhile1));
        assert_eq!(parse_header("#"), err!("", ErrorKind::Tag));
        assert_eq!(parse_header("# \n"), Ok(("", (1, vec![]))));
        assert_eq!(parse_header("# test"), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_unordered_list_tag() {
        assert_eq!(parse_unordered_list_tag("- "), Ok(("", "-")));
        assert_eq!(
            parse_unordered_list_tag("- and some more"),
            Ok(("and some more", "-"))
        );
        assert_eq!(parse_unordered_list_tag("-"), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_unordered_list_tag("-and some more"),
            err!("and some more", ErrorKind::Tag)
        );
        assert_eq!(parse_unordered_list_tag("--"), err!("-", ErrorKind::Tag));
        assert_eq!(parse_unordered_list_tag(""), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_unordered_list_element() {
        assert_eq!(
            parse_unordered_list_element("- this is an element\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_unordered_list_element("- this is an element\n- this is another element\n"),
            Ok((
                "- this is another element\n",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(parse_unordered_list_element(""), err!("", ErrorKind::Tag));
        assert_eq!(parse_unordered_list_element("- \n"), Ok(("", vec![])));
        assert_eq!(parse_unordered_list_element("- "), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_unordered_list_element("- test"),
            err!("", ErrorKind::Tag)
        );
        assert_eq!(parse_unordered_list_element("-"), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_unordered_list() {
        assert_eq!(
            parse_unordered_list("- this is an element"),
            err!("", ErrorKind::Tag)
        );
        assert_eq!(
            parse_unordered_list("- this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]]
            ))
        );
        assert_eq!(
            parse_unordered_list("- this is an element\n- here is another\n"),
            Ok((
                "",
                vec![
                    vec![MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))],
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_ordered_list_tag() {
        assert_eq!(parse_ordered_list_tag("1. "), Ok(("", "1")));
        assert_eq!(parse_ordered_list_tag("1234567. "), Ok(("", "1234567")));
        assert_eq!(
            parse_ordered_list_tag("3. and some more"),
            Ok(("and some more", "3"))
        );
        assert_eq!(parse_ordered_list_tag("1"), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_ordered_list_tag("1.and some more"),
            err!("and some more", ErrorKind::Tag)
        );
        assert_eq!(parse_ordered_list_tag("1111."), err!("", ErrorKind::Tag));
        assert_eq!(parse_ordered_list_tag(""), err!("", ErrorKind::TakeWhile1));
    }

    #[test]
    fn test_parse_ordered_list_element() {
        assert_eq!(
            parse_ordered_list_element("1. this is an element\n"),
            Ok((
                "",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element("1. this is an element\n1. here is another\n"),
            Ok((
                "1. here is another\n",
                vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]
            ))
        );
        assert_eq!(
            parse_ordered_list_element(""),
            err!("", ErrorKind::TakeWhile1)
        );
        assert_eq!(
            parse_ordered_list_element(""),
            err!("", ErrorKind::TakeWhile1)
        );
        assert_eq!(parse_ordered_list_element("1. \n"), Ok(("", vec![])));
        assert_eq!(
            parse_ordered_list_element("1. test"),
            err!("", ErrorKind::Tag)
        );
        assert_eq!(parse_ordered_list_element("1. "), err!("", ErrorKind::Tag));
        assert_eq!(parse_ordered_list_element("1."), err!("", ErrorKind::Tag));
    }

    #[test]
    fn test_parse_ordered_list() {
        assert_eq!(
            parse_ordered_list("1. this is an element\n"),
            Ok((
                "",
                vec![vec![MarkdownInline::Plaintext(String::from(
                    "this is an element"
                ))]]
            ))
        );
        assert_eq!(parse_ordered_list("1. test"), err!("", ErrorKind::Tag));
        assert_eq!(
            parse_ordered_list("1. this is an element\n2. here is another\n"),
            Ok((
                "",
                vec![
                    vec!(MarkdownInline::Plaintext(String::from(
                        "this is an element"
                    ))),
                    vec![MarkdownInline::Plaintext(String::from("here is another"))]
                ]
            ))
        );
    }

    #[test]
    fn test_parse_codeblock() {
        assert_eq!(
            parse_code_block("```bash\npip install foobar\n```"),
            Ok(("", ("bash", "pip install foobar\n")))
        );
        assert_eq!(
            parse_code_block("```\nimport foobar\n\n```"),
            Ok(("", ("", "import foobar\n\n")))
        );
        assert_eq!(
            parse_code_block("```python\nimport foobar\n\n```"),
            Ok(("", ("python", "import foobar\n\n")))
        );
        assert_eq!(
            parse_code_block("```\npip `install` foobar\n```"),
            Ok(("", ("", "pip `install` foobar\n")))
        );
    }

    #[test]
    fn test_parse_markdown() {
        assert_eq!(
            parse_markdown("# Foobar\n\nFoobar is a Python library for dealing with word pluralization.\n\n```bash\n pip install foobar\n```\n## Installation\n\nUse the package manager [pip](https://pip.pypa.io/en/stable/) to install foobar.\n```python\nimport foobar\n\nfoobar.pluralize('word') # returns 'words'\nfoobar.pluralize('goose') # returns 'geese'\nfoobar.singularize('phenomena') # returns 'phenomenon'\n```"),
            Ok(("", vec![
                Markdown::Heading(1, vec![MarkdownInline::Plaintext(String::from("Foobar"))]),
                Markdown::Line(vec![]),
                Markdown::Line(vec![MarkdownInline::Plaintext(String::from("Foobar is a Python library for dealing with word pluralization."))]),
                Markdown::Line(vec![]),
                Markdown::Codeblock(String::from("bash"), String::from(" pip install foobar\n")),
                Markdown::Line(vec![]),
                Markdown::Heading(2, vec![MarkdownInline::Plaintext(String::from("Installation"))]),
                Markdown::Line(vec![]),
                Markdown::Line(vec![
                    MarkdownInline::Plaintext(String::from("Use the package manager ")),
                    MarkdownInline::Link(String::from("pip"), String::from("https://pip.pypa.io/en/stable/")),
                    MarkdownInline::Plaintext(String::from(" to install foobar.")),
                ]),
                Markdown::Codeblock(String::from("python"), String::from("import foobar\n\nfoobar.pluralize('word') # returns 'words'\nfoobar.pluralize('goose') # returns 'geese'\nfoobar.singularize('phenomena') # returns 'phenomenon'\n")),
            ]))
        )
    }
}
