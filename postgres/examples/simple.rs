use mobc::Error;
use mobc::Pool;
use mobc_postgres::tokio_postgres;
use mobc_postgres::PostgresConnectionManager;
use std::str::FromStr;
use std::time::{Instant};
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio_postgres::Config;
use tokio_postgres::Error as PostgresError;
use tokio_postgres::NoTls;

const MAX: usize = 10000;

async fn simple_query(
    pool: Pool<PostgresConnectionManager<NoTls>>,
    mut sender: mpsc::Sender<()>,
) -> Result<(), Error<PostgresError>> {
    let conn = pool.get().await?;
    // let statement = conn.prepare("SELECT 1").await?;
    let r = conn.execute("SELECT 1", &[]).await?;
    assert_eq!(r, 1);
    sender.send(()).await.unwrap();
    Ok(())
}

async fn do_postgres(sender: mpsc::Sender<()>) -> Result<(), Error<PostgresError>> {
    let config = Config::from_str("postgres://jiaju:jiaju@localhost:5432")?;
    let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::new(manager).await?;

    for _ in 0..MAX {
        let pool = pool.clone();
        let tx = sender.clone();
        tokio::spawn(simple_query(pool, tx).map(|_| ()));
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mark = Instant::now();
    let (tx, mut rx) = mpsc::channel::<()>(MAX);

    if let Err(_) = do_postgres(tx).await {
        println!("some error");
    }

    let mut num: usize = 0;
    while let Some(_) = rx.next().await {
        num += 1;
        if num == MAX {
            break;
        }
    }

    println!("cost {:?}", mark.elapsed());
}
