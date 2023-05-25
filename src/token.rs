use chumsky::{
    extra,
    prelude::Rich,
    primitive::{any, choice, just, none_of},
    span::SimpleSpan,
    text::{ident, inline_whitespace},
    IterParser, Parser,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ident<'a> {
    pub(crate) name: &'a str,
    pub(crate) span: SimpleSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ref<'a> {
    pub(crate) value: &'a str,
    pub(crate) span: SimpleSpan,
}

impl<'a> From<Ident<'a>> for Ref<'a> {
    fn from(value: Ident<'a>) -> Self {
        let Ident { name, span } = value;
        Self { value: name, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token<'a> {
    Ident(Ident<'a>),
    LParen(LParen),
    RParen(RParen),
    Ref(Ref<'a>),
    // Dollar(Dollar),
}

impl<'a> Token<'a> {
    pub(crate) fn parser() -> impl Parser<'a, &'a str, Vec<Token<'a>>, extra::Err<Rich<'a, char>>> {
        choice((
            just('(').map_with_span(|_, span| Token::LParen(LParen { span })),
            just(')').map_with_span(|_, span| Token::RParen(RParen { span })),
            // TODO: Everything immediately after
            // TODO: implement escapes, single quoting
            any()
                .filter(|c| char::is_ascii_alphanumeric(c) || char::is_ascii_punctuation(c))
                .filter(|c| *c != '"')
                .repeated()
                .at_least(1)
                // .at_most(9001) // TODO
                .slice()
                .padded_by(just('"'))
                .map_with_span(|value, span| Token::Ref(Ref { value, span })),
            ident()
                .map_with_span(|name, span| Token::Ident(Ident { name, span }))
                .then_ignore(none_of("/-_").rewind()),
            any()
                .filter(|c| char::is_ascii_alphanumeric(c) || matches!(c, '/' | '-' | '_'))
                .repeated()
                .at_least(1)
                // .at_most(9001) // TODO
                .separated_by(just('/'))
                .at_least(1)
                .slice()
                .map_with_span(|value, span| Token::Ref(Ref { value, span })),
            // just('$').map_with_span(|_, span| Token::Dollar(Dollar { span })),
        ))
        .separated_by(inline_whitespace())
        .allow_leading()
        .allow_trailing()
        .collect()
    }
}

#[derive(Debug)]
pub(crate) enum TokenizationError {}

macro_rules! unit_tokens {
        ($($ident:ident),* $(,)?) => {
            $(
                #[derive(Clone, Debug, PartialEq)]
                pub(crate) struct $ident {
                    span: SimpleSpan,
                }
            )*
        };
    }

unit_tokens! {
    LParen,
    RParen,
    // Dollar,
}
