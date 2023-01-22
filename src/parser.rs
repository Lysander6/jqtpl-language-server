use chumsky::prelude::*;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Comment,
    Content,
    Each,
    EachEnd,
    Else,
    Html,
    If,
    IfEnd,
    Print,
    Tmpl,
    Unknown(String),
    Var,
}

#[derive(Debug, PartialEq)]
pub struct Spanned<T>(T, Range<usize>);

pub fn parser() -> impl Parser<char, Vec<Spanned<Stmt>>, Error = Simple<char>> {
    use Stmt::*;

    let code = choice((none_of("\n\r}"), just('}').padded_by(none_of("\n\r}"))))
        .repeated()
        .at_least(1);
    let content = choice((none_of("{"), just('{').padded_by(none_of("{"))))
        .repeated()
        .at_least(1)
        .map_with_span(|_, span| Spanned(Content, span));

    let comment = just("!").then_ignore(code.clone().or_not()).to(Comment);
    let each = text::keyword("each").then_ignore(code.clone()).to(Each);
    let each_end = just("/each").to(EachEnd);
    let r#else = text::keyword("else")
        .then_ignore(code.clone().or_not())
        .to(Else);
    let html = text::keyword("html").then_ignore(code.clone()).to(Html);
    let r#if = text::keyword("if").then_ignore(code.clone()).to(If);
    let if_end = just("/if").to(IfEnd);
    let print = just("=").then_ignore(code.clone()).to(Print);
    let tmpl = text::keyword("tmpl").then_ignore(code.clone()).to(Tmpl);
    let unknown = code.clone().map(|s| Unknown(String::from_iter(s.iter())));
    let var = text::keyword("var").then_ignore(code.clone()).to(Var);

    let statement = choice((
        comment, each, each_end, r#else, html, r#if, if_end, print, tmpl, unknown, var,
    ))
    .delimited_by(just("{{"), just("}}"))
    .map_with_span(|t, span| Spanned(t, span));

    choice((statement, content))
        // .recover_with(skip_then_retry_until(['{', '}']).consume_end())
        .repeated()
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use Stmt::*;

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
                Spanned(IfEnd, 70..77),
            ])
        );
    }

    #[test]
    fn trailing_content() {
        let src = r###"{{if 1 = 1}}
    "###;

        let result = parser().parse(src);

        assert_eq!(
            result,
            Ok(vec![Spanned(If, 0..12), Spanned(Content, 12..17)])
        );
    }
}
