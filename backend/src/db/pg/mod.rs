use redis::Client;
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;

pub mod example;

#[derive(Clone, Debug)]
pub(crate) struct PgDriver {
    #[allow(unused)]
    pub(crate) pg: Pool<Postgres>,
    #[allow(unused)]
    pub(crate) redis: redis::Client,
}

impl PgDriver {

    pub(crate) async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            pg: PgPoolOptions::new()
                .max_connections(std::env::var("PG_MAX_CONNECTIONS")?.parse()?)
                .connect_with(
                    PgConnectOptions::new()
                        .host(&std::env::var("PG_HOST")?)
                        .port(std::env::var("PG_PORT")?.parse()?)
                        .username(&std::env::var("PG_USER")?)
                        .password(&std::env::var("PG_PASS")?)
                        .database(&std::env::var("PG_DATA")?),
                )
                .await?,
            redis: Client::open(std::env::var("REDIS_URI")?)?,
        })
    }

}

impl super::Repository for PgDriver {}
