use chumsky::prelude::*;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Directive(String),
    Content,
}

#[derive(Debug, PartialEq)]
struct Spanned<T>(T, Range<usize>);

#[allow(dead_code)]
fn parser() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let code = choice((none_of("\n\r}"), just('}').padded_by(none_of("}"))))
        .repeated()
        .at_least(1);

    let content = take_until(just("{").rewind());

    let directive = code.delimited_by(just("{{"), just("}}"));

    choice((
        directive.map(|code| Token::Directive(String::from_iter(code.iter()))),
        content.to(Token::Content),
    ))
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
        assert_eq!(result, Ok(vec![Token::Directive("if ".to_string())]));
    }

    #[test]
    fn else_() {
        let result = parser().parse("{{else}}");
        assert_eq!(result, Ok(vec![Token::Directive("else".to_string())]));
    }

    #[test]
    fn if_else() {
        let src = "{{if bla}}
          hello
        {{else}}
          goodbye
        {{/if}}";
        let result = parser().parse(src);
        assert_eq!(result, Ok(vec![Token::Directive("else".to_string())]));
    }
}
