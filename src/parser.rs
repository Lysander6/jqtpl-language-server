use chumsky::prelude::*;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Content,
    Comment,
    Each,
    EndEach,
    Html,
    Var,
    If,
    Else,
    EndIf,
    Tmpl,
    Print,
    Unknown(String),
}

#[derive(Debug, PartialEq)]
struct Spanned<T>(T, Range<usize>);

#[allow(dead_code)]
fn parser() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let code = choice((none_of("\n\r}"), just('}').padded_by(none_of("}"))))
        .repeated()
        .at_least(1);

    let content = take_until(just("{").rewind());

    let r#if = text::keyword("if").then_ignore(code.clone()).to(Token::If);

    let r#else = text::keyword("else")
        .then_ignore(code.clone().or_not())
        .to(Token::Else);

    let end_if = just("/if").to(Token::EndIf);

    let comment = just("!")
        .then_ignore(code.clone().or_not())
        .to(Token::Comment);

    let var = text::keyword("var")
        .then_ignore(code.clone())
        .to(Token::Var);

    let each = text::keyword("each")
        .then_ignore(code.clone())
        .to(Token::Each);

    let end_each = just("/each").to(Token::EndEach);

    let html = text::keyword("html")
        .then_ignore(code.clone())
        .to(Token::Html);

    let tmpl = text::keyword("tmpl")
        .then_ignore(code.clone())
        .to(Token::Tmpl);

    let print = just("=").then_ignore(code.clone()).to(Token::Print);

    let directive = choice((
        r#if,
        r#else,
        end_if,
        each,
        end_each,
        html,
        comment,
        var,
        tmpl,
        print,
        code.clone()
            .map(|s| Token::Unknown(String::from_iter(s.iter()))),
    ))
    .delimited_by(just("{{"), just("}}"));

    choice((directive, content.to(Token::Content)))
        // .recover_with(skip_then_retry_until(['{', '}']).consume_end())
        .repeated()
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let result = parser().parse("");
        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn braces() {
        let result = parser().parse("{{if }}");
        assert_eq!(result, Ok(vec![Token::If]));
    }

    #[test]
    fn else_() {
        let result = parser().parse("{{else}}");
        assert_eq!(result, Ok(vec![Token::Else]));
    }

    #[test]
    fn if_else() {
        let src = "{{if bla}}
          hello
        {{else}}
          goodbye
        {{/if}}";
        let result = parser().parse(src);
        assert_eq!(
            result,
            Ok(vec![
                Token::If,
                Token::Content,
                Token::Else,
                Token::Content,
                Token::EndIf
            ])
        );
    }
}
