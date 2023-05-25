pub mod cli;

pub trait Git {
    type QueryInitError;
    type QueryResultIterError;
    type QueryIter: Iterator<Item = Result<String, Self::QueryResultIterError>>;

    fn locals(&self) -> Result<Self::QueryIter, Self::QueryInitError>;
    fn query_refs<'a>(
        &self,
        refs: &'a dyn Iterator<Item = &'a str>,
    ) -> Result<Self::QueryIter, Self::QueryInitError>;
}
