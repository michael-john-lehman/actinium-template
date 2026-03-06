# Outbound

A driver that provides access to external resources through a standardized interface.

Drivers act as bridges between the system and external services, databases, or APIs.
They encapsulate the logic for connecting to, authenticating with, and operating on
external resources while providing a consistent interface for the rest of the application.

**Requirements**

Implementations of this trait must:
* Be thread-safe (`Send + Sync`)
* Have a static lifetime (`'static`)
* Provide a unique name that identifies the driver
* Implement all *interfaces*

Note: we borrow the 'Driver' terminology because it sounds cool

```rust,noplayground

pub trait Driver: Send + Sync + 'static
    + interfaces::example::IExample
{

    /// Returns the unique identifier for this driver.
    fn name(&self) -> &'static str;

}

```

## Interface Example

Note: **IExampleTransaction** is a useful pattern to create another layer of indirection, which is also useful for edge cases such as fallback logic if connection is dropped etc

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
