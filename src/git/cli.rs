use std::{
    convert::Infallible,
    io::{self, Cursor},
    process::Output,
    string::FromUtf8Error,
};

use ezcmd::{EasyCommand, ExecuteError};

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
        let head_is_detached = stdout_lines(EasyCommand::new_with("git", |cmd| {
            cmd.args(["branch", "--show-current"])
        }))
        .map_err(|source| QueryInitErrorKind::ShowCurrentBranch { source })?
        .is_empty();
        log::trace!("`HEAD` is detached: {head_is_detached:?}");

        let mut format = "--format=".to_owned();
        if head_is_detached {
            format.push_str("%(if)%(HEAD)%(then)HEAD%(else)");
        }
        format.push_str("%(refname:short)");
        let mut include_in_format = |prop_name: &str| {
            format += &format!("%(if)%({prop_name})%(then)\n%({prop_name}:short)%(end)");
        };
        // if select_upstreams {
        //     include_in_format("upstream");
        // }
        // if select_pushes {
        //     include_in_format("push");
        // }
        if head_is_detached {
            format.push_str("%(end)");
        }

        let branches = stdout_lines(EasyCommand::new_with("git", |cmd| {
            cmd.args(["branch", "--list"]).arg(format)
        }))
        .map_err(|source| QueryInitErrorKind::ListBranches { source })?;

        Ok(QueryIterator {
            branches: branches.into_iter(),
        })
    }
}

#[derive(Debug)]
pub struct QueryInitError {
    kind: QueryInitErrorKind,
}

#[derive(Debug, thiserror::Error)]
enum QueryInitErrorKind {
    #[error(transparent)]
    ShowCurrentBranch { source: StdoutLinesError },
    #[error(transparent)]
    ListBranches { source: StdoutLinesError },
}

impl From<QueryInitErrorKind> for QueryInitError {
    fn from(kind: QueryInitErrorKind) -> Self {
        Self { kind }
    }
}

pub struct QueryIterator {
    branches: std::vec::IntoIter<String>,
}

impl Iterator for QueryIterator {
    type Item = Result<String, Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        self.branches.next().map(Ok)
    }
}

fn stdout_lines(mut cmd: EasyCommand) -> Result<Vec<String>, StdoutLinesError> {
    let output = cmd
        .output()
        .map_err(|source| StdoutLinesError::FailedToSpawnSubprocess { source })?;
    let Output {
        stdout,
        stderr,
        status,
    } = output;

    let code = status.code();
    if code != Some(0) {
        io::copy(&mut Cursor::new(stderr), &mut io::stderr()).unwrap();
        return Err(StdoutLinesError::NonZeroExitStatus { code });
    }

    let stdout =
        String::from_utf8(stdout).map_err(|source| StdoutLinesError::StdoutNotUtf8 { source })?;
    Ok(stdout.lines().map(|line| line.trim().to_owned()).collect())
}

#[derive(Debug, thiserror::Error)]
enum StdoutLinesError {
    #[error(transparent)]
    FailedToSpawnSubprocess { source: ExecuteError<io::Error> },
    #[error("expected exit code of 0, got {code:?}")]
    NonZeroExitStatus { code: Option<i32> },
    #[error("`stdout` was not UTF-8")]
    StdoutNotUtf8 { source: FromUtf8Error },
}
