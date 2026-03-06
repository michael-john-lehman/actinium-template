# Driver Interfaces

### Example 

Note: **IExampleTransaction** is a useful pattern to create another layer of indirection.

```rust,noplayground
use futures_util::future::LocalBoxFuture;

#[derive(thiserror::Error, Debug)]
pub enum IExampleError {
    #[error("IExample Error: DriverError<{driver}>({error})")]
    DriverError {
        driver: &'static str,
        error: Box<dyn std::error::Error>,
    },
}

pub trait IExampleTransaction<'trans> {

    fn commit(self) -> LocalBoxFuture<'trans, Result<(), IExampleError>>;

}

pub trait IExample: Send + Sync + 'static {

    fn begin_transaction<'repo>(
        &'repo self,
    ) -> LocalBoxFuture<'repo, Result<Box<dyn IExampleTransaction<'repo> + 'repo>, IExampleError>>;

}

```

```rust,noplayground
use super::*;
use futures_util::future::LocalBoxFuture;
use crate::outbound::interfaces::example::*;

pub(crate) struct PgExampleTransaction<'trans> {
    pub(crate) transaction: sqlx::Transaction<'trans, Postgres>
}

impl<'trans> IExampleTransaction<'trans> for PgExampleTransaction<'trans> {

    fn commit(self) -> LocalBoxFuture<'trans, Result<(), IExampleError>> {
        Box::pin(async move {
            self.transaction.commit().await
                .map_err(|err| IExampleError::DriverError {driver:"pg", error: Box::new(err)})?;
            Ok(())
        })
    }

}

impl IExample for PgDriver {

    fn begin_transaction<'repo>(&'repo self) -> LocalBoxFuture<'repo, Result<Box<dyn IExampleTransaction<'repo> + 'repo>, IExampleError>> {
        Box::pin(async move {
            let pg_transaction = self.pg.begin().await
                .map_err(|err| IExampleError::DriverError {driver:"pg", error: Box::new(err)})?;
            let example_transaction = PgExampleTransaction {
                transaction: pg_transaction
            };
            Ok(Box::new(example_transaction) as Box<dyn IExampleTransaction<'repo> + 'repo>)
        })
    }
    
}
```
