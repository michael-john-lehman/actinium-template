use super::*;
use crate::db::interfaces::example::*;
use crate::db::error::RepositoryError;
use futures_util::future::LocalBoxFuture;

pub(crate) struct PgExampleTransaction<'trans> {
    pub(crate) transaction: sqlx::Transaction<'trans, Postgres>
}

impl<'trans> IExampleTransaction<'trans> for PgExampleTransaction<'trans> {

    fn commit(self) -> LocalBoxFuture<'trans, Result<(), RepositoryError>> {
        Box::pin(async move {
            self.transaction.commit().await.map_err(|err| RepositoryError::DriverError { driver: "pg", error: Box::new(err) })?;
            Ok(())
        })
    }

}

impl IExample for PgDriver {

    fn begin_transaction<'repo>(&'repo self) -> LocalBoxFuture<'repo, Result<Box<dyn IExampleTransaction<'repo> + 'repo>, RepositoryError>> {
        Box::pin(async move {
            let pg_transaction = self.pg.begin().await
                .map_err(|err| RepositoryError::DriverError { driver: "pg", error: Box::new(err) })?;
            let example_transaction = PgExampleTransaction {
                transaction: pg_transaction
            };
            Ok(Box::new(example_transaction) as Box<dyn IExampleTransaction<'repo> + 'repo>)
        })
    }
    
}
