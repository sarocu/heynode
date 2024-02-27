use std::io::Error;

use tokio_postgres::{self, tls::NoTlsStream, NoTls, Row, Socket};

pub struct DbClient {
    client: tokio_postgres::Client,
}

impl DbClient {
    pub async fn new(host: &str) -> Self {
        let config = format!("{}", host);
        let (client, connection) = tokio_postgres::connect(&config, NoTls)
            .await
            .expect("could not connect to database");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprint!("connection error: {}", e);
            }
        });

        DbClient { client }
    }

    pub async fn get_locks(&self) -> Result<Vec<Row>, tokio_postgres::Error> {
        self.client
            .query("select wait_event, state, query from pg_stat_activity where wait_event is not null and state is not null", &[])
            .await
    }
}
