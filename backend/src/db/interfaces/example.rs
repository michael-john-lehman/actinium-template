use crate::db::error::RepositoryError;
use futures_util::future::LocalBoxFuture;

pub trait IExampleTransaction<'trans> {

    fn commit(self) -> LocalBoxFuture<'trans, Result<(), RepositoryError>>;

}

pub trait IExample: Send + Sync + 'static {

    fn begin_transaction<'repo>(&'repo self) -> LocalBoxFuture<'repo, Result<Box<dyn IExampleTransaction<'repo> + 'repo>, RepositoryError>>;

}
