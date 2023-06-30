use chumsky::{prelude::Rich, primitive::end, Parser};
use git::Git;
use ir::{QueryFromAstError, QueryNode};

use crate::{ast::Ast, token::Token};

mod ast;
pub mod git;
mod ir;
mod token;

#[derive(Clone, Debug, PartialEq)]
pub struct Query<'a> {
    nodes: Vec<QueryNode<'a>>,
}

impl<'a> Query<'a> {
    pub fn parse(
        query: &'a str,
        cx: &'a mut ParsingContext<'a>,
    ) -> Result<&'a Self, QueryStringParseError<'a>> {
        *cx = ParsingContext::new();

        let token_stream = cx.token_stream.insert(
            Token::parser()
                .then_ignore(end())
                .parse(&*query)
                .into_result()
                .map_err(QueryStringParseErrorKind::TokenizeString)?,
        );

        log::trace!("token stream: {token_stream:#?}");

        let ast = cx.ast.insert(
            Ast::parser()
                .then_ignore(end())
                .parse(&*token_stream)
                .into_result()
                .map_err(QueryStringParseErrorKind::ParseAstFromTokens)?,
        );

        log::trace!("parsed AST: {ast:#?}");

        let query = cx.query.insert(
            Query::parser()
                .then_ignore(end())
                .parse(&ast.nodes)
                .into_result()
                .map_err(QueryStringParseErrorKind::ParseQueryFromAst)?,
        );

        log::trace!("parsed query: {query:#?}");

        Ok(query)
    }

    pub fn expand_refs<'g, G>(&self, git: &'g G) -> Result<GitQueryIter<'g, G>, G::QueryInitError>
    where
        G: Git,
    {
        let mut refs = Vec::new();
        let mut queries = Vec::new();

        let Self { nodes } = self;
        for node in nodes {
            match node {
                QueryNode::Ref(ref_) => refs.push(Ok(ref_.value.to_owned())),
                QueryNode::Locals { span } => queries.push(git.locals()?),
            }
        }

        Ok(GitQueryIter {
            refs: refs.into_iter(),
            queries: queries.into_iter(),
            git,
        })
    }
}

#[derive(Debug)]
pub struct QueryStringParseError<'a> {
    source: QueryStringParseErrorKind<'a>,
}

impl<'a> From<QueryStringParseErrorKind<'a>> for QueryStringParseError<'a> {
    fn from(source: QueryStringParseErrorKind<'a>) -> Self {
        Self { source }
    }
}

#[derive(Debug)]
pub(crate) enum QueryStringParseErrorKind<'a> {
    TokenizeString(Vec<Rich<'a, char>>),
    ParseAstFromTokens(Vec<Rich<'a, Token<'a>>>),
    ParseQueryFromAst(Vec<QueryFromAstError<'a>>),
}

#[derive(Default)]
pub struct ParsingContext<'a> {
    token_stream: Option<Vec<Token<'a>>>,
    ast: Option<Ast<'a>>,
    query: Option<Query<'a>>,
}

impl ParsingContext<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct GitQueryIter<'a, G>
where
    G: Git,
{
    refs: std::vec::IntoIter<Result<String, G::QueryResultIterError>>,
    queries: std::vec::IntoIter<G::QueryIter>,
    git: &'a G,
}

impl<'a, G> Iterator for GitQueryIter<'a, G>
where
    G: Git,
{
    type Item = Result<String, G::QueryResultIterError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { refs, queries, git } = self;

        loop {
            while let Some(ref_) = refs.next() {
                return Some(ref_);
            }

            if let Some(query) = queries.next() {
                *refs = query.collect::<Vec<_>>().into_iter();
                continue;
            }

            break None;
        }
    }
}
