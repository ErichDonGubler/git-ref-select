use std::io::{stdout, Write};

use git_ref_select::{git::cli::GitCli, ParsingContext, Query};
use miette::miette;

#[derive(Debug, clap::Parser)]
struct Args {
    query: String,
    // unique: bool, // TODO
    // sort: bool, // TODO
}

fn main() -> miette::Result<()> {
    env_logger::init();

    let Args { query } = <Args as clap::Parser>::parse();

    let mut cx = ParsingContext::new();

    let query = Query::parse(&query, &mut cx).map_err(|err| miette!("{:?}", err))?;
    let git_cli = GitCli::new();

    let mut stdout = stdout().lock();
    match query.expand_refs(&git_cli) {
        Ok(iter) => {
            for ref_res in iter {
                let ref_ = ref_res.unwrap();
                writeln!(&mut stdout, "{ref_}").unwrap();
            }
        }
        // TODO: iter over all failures, try to keep successes?
        Err(e) => {
            log::error!("{e:?}");
        }
    }
    Ok(())
}
