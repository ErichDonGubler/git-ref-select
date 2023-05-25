use chumsky::{
    extra, prelude::Rich, primitive::choice, recursive::recursive, select, IterParser, Parser,
};

use crate::token::{Ident, Ref, Token};

#[derive(Debug)]
pub(crate) struct Ast<'a> {
    pub(crate) nodes: Vec<AstNode<'a>>,
}

impl<'a> Ast<'a> {
    pub(crate) fn parser(
    ) -> impl Parser<'a, &'a [Token<'a>], Ast<'a>, extra::Err<Rich<'a, Token<'a>>>> {
        AstNode::parser()
            .repeated()
            .collect::<Vec<_>>()
            .map(|nodes| Ast { nodes })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AstNode<'a> {
    Ref(Ref<'a>),
    Call(Call<'a>),
}

impl<'a> AstNode<'a> {
    pub(crate) fn parser(
    ) -> impl Parser<'a, &'a [Token<'a>], AstNode<'a>, extra::Err<Rich<'a, Token<'a>>>> {
        recursive(|node| {
            choice((
                select!(Token::Ident(s) => s)
                    .then(
                        node.repeated()
                            .collect::<Vec<_>>()
                            .delimited_by(select!(Token::LParen(_)), select!(Token::RParen(_))),
                    )
                    .map(|(identifier, args)| Self::Call(Call { identifier, args })),
                select!(Token::Ident(s) => s).map(|value| Self::Ref(value.into())),
                select!(Token::Ref(s) => s).map(|value| Self::Ref(value)),
            ))
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Call<'a> {
    pub(crate) identifier: Ident<'a>,
    pub(crate) args: Vec<AstNode<'a>>,
    // pub(crate) span: SimpleSpan // TODO
}
