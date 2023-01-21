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
fn parser() -> impl Parser<char, Vec<Spanned<Token>>, Error = Simple<char>> {
    use Token::*;

    let code = choice((none_of("\n\r}"), just('}').padded_by(none_of("}"))))
        .repeated()
        .at_least(1);

    let content = take_until(just("{").rewind()).map_with_span(|_, span| Spanned(Content, span));

    let r#if = text::keyword("if").then_ignore(code.clone()).to(If);

    let r#else = text::keyword("else")
        .then_ignore(code.clone().or_not())
        .to(Else);

    let end_if = just("/if").to(EndIf);

    let comment = just("!").then_ignore(code.clone().or_not()).to(Comment);

    let var = text::keyword("var").then_ignore(code.clone()).to(Var);

    let each = text::keyword("each").then_ignore(code.clone()).to(Each);

    let end_each = just("/each").to(EndEach);

    let html = text::keyword("html").then_ignore(code.clone()).to(Html);

    let tmpl = text::keyword("tmpl").then_ignore(code.clone()).to(Tmpl);

    let print = just("=").then_ignore(code.clone()).to(Print);

    let unknown = code.clone().map(|s| Unknown(String::from_iter(s.iter())));

    let directive = choice((
        r#if, r#else, end_if, each, end_each, html, comment, var, tmpl, print, unknown,
    ))
    .delimited_by(just("{{"), just("}}"))
    .map_with_span(|t, span| Spanned(t, span));

    choice((directive, content))
        // .recover_with(skip_then_retry_until(['{', '}']).consume_end())
        .repeated()
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn empty_string() {
        let result = parser().parse("");
        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn if_() {
        let result = parser().parse("{{if }}");
        assert_eq!(result, Ok(vec![Spanned(If, 0..7)]));
    }

    #[test]
    fn else_() {
        let result = parser().parse("{{else}}");
        assert_eq!(result, Ok(vec![Spanned(Else, 0..8)]));
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
                Spanned(If, 0..10),
                Spanned(Content, 10..35),
                Spanned(Else, 35..43),
                Spanned(Content, 43..70),
                Spanned(EndIf, 70..77),
            ])
        );
    }
}
