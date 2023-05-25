use chumsky::{
    extra, primitive::any, recursive::recursive, span::SimpleSpan, util::MaybeRef, IterParser,
    Parser,
};

use crate::{
    ast::{AstNode, Call},
    token::{self, Ident},
    Query,
};

impl<'a> Query<'a> {
    pub(crate) fn parser(
    ) -> impl Parser<'a, &'a [AstNode<'a>], Query<'a>, extra::Err<QueryFromAstError<'a>>> {
        QueryNode::parser()
            .repeated()
            .collect()
            .map(|nodes| Self { nodes })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum QueryNode<'a> {
    Ref(Ref<'a>),
    Locals { span: SimpleSpan },
    // AndUpstream { input: Box<Query> },
}

impl<'a> QueryNode<'a> {
    pub(crate) fn parser(
    ) -> impl Parser<'a, &'a [AstNode<'a>], QueryNode<'a>, extra::Err<QueryFromAstError<'a>>> {
        recursive(|query| {
            any().try_map(|node, _span| match node {
                AstNode::Ref(token::Ref { value, span }) => Ok(Self::Ref(Ref { value, span })),
                AstNode::Call(call) => {
                    let Call {
                        identifier: Ident { name, span },
                        args,
                    } = &call;
                    match *name {
                        "locals" => {
                            if args.is_empty() {
                                Ok(Self::Locals { span: span.clone() })
                            } else {
                                Err(QueryFromAstError::IncorrectArgCount { call })
                            }
                        }
                        _ => Err(QueryFromAstError::InvalidCall {
                            ident: call.identifier,
                        }),
                    }
                }
            })
        })
    }
}

#[derive(Debug)]
pub(crate) enum QueryFromAstError<'a> {
    Unexpected {
        found: Option<MaybeRef<'a, AstNode<'a>>>,
        expected: Vec<MaybeRef<'a, AstNode<'a>>>,
    },
    IncorrectArgCount {
        call: Call<'a>,
    },
    InvalidCall {
        ident: Ident<'a>,
    },
}

impl<'a> chumsky::error::Error<'a, &'a [AstNode<'a>]> for QueryFromAstError<'a> {
    fn expected_found<E>(
        expected: E,
        found: Option<MaybeRef<'a, AstNode<'a>>>,
        _span: SimpleSpan,
    ) -> Self
    where
        E: IntoIterator<Item = Option<MaybeRef<'a, AstNode<'a>>>>,
    {
        Self::Unexpected {
            expected: expected.into_iter().flatten().collect(),
            found,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Ref<'a> {
    pub(crate) value: &'a str,
    span: SimpleSpan,
}
