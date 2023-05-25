use std::convert::Infallible;

use ezcmd::EasyCommand;

use crate::ir::Ref;

use super::Git;

pub struct GitCli {
    _no_pub_ctor: (),
}

impl GitCli {
    pub fn new() -> Self {
        Self { _no_pub_ctor: () }
    }
}

impl Git for GitCli {
    type QueryInitError = QueryInitError;
    type QueryResultIterError = Infallible;
    type QueryIter = QueryIterator;

    fn locals(&self) -> Result<Self::QueryIter, Self::QueryInitError> {
        EasyCommand::new_with("git", |cmd| {
            cmd.args(["branch", "--list", "--format=%(refname:short)"])
        });
        todo!();
    }

    fn query_refs<'a>(
        &self,
        refs: &'a dyn Iterator<Item = &'a str>,
    ) -> Result<Self::QueryIter, Self::QueryInitError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct QueryInitError {}

pub struct QueryIterator {}

impl Iterator for QueryIterator {
    type Item = Result<String, Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
