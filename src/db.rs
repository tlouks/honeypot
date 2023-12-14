#![allow(unused)]
use std::{cell::LazyCell, process::exit};

use anyhow::Result;
use chrono::NaiveDateTime;
use futures::TryStreamExt;
use sqlx::{postgres::PgPool, Pool, Postgres, Row};
use uuid::Uuid;

pub async fn get_conn() -> Result<PgPool, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").unwrap();

    PgPool::connect(&url).await
}

pub async fn open_db() -> Result<()> {
    let conn = get_conn().await?;

    println!("Got a connection. {:?}", conn);

    let mut rows = sqlx::query("SELECT * FROM pg_catalog.pg_tables").fetch(&conn);

    while let Some(row) = rows.try_next().await? {
        println!("{:?}", row.columns());
    }

    Ok(())
}

pub async fn add_entry(pool: &PgPool, ip: String) -> usize {
    if let Err(e) = sqlx::query!(
        r#"
        INSERT INTO stats (ip_address, ping_count)
        values ($1, $2)
        returning ip_address, ping_count
        "#,
        ip,
        0
    )
    .fetch_one(pool)
    .await
    {
        match sqlx::query!(
            r#"SELECT ip_address, ping_count FROM stats WHERE ip_address = $1"#,
            ip
        )
        .fetch_one(pool)
        .await
        {
            Ok(r) => {
                let update_result = sqlx::query!(
                    r#"
                    UPDATE stats SET
                    ping_count = $1
                    WHERE ip_address = $2
                    returning ip_address, ping_count
                    "#,
                    r.ping_count + 1,
                    ip,
                )
                .fetch_one(pool)
                .await;

                if let Err(e) = update_result {
                    eprintln!("{e}");
                    return 1;
                }
            }

            Err(e) => {
                eprintln!("{e}");
                return 1;
            }
        };
    };

    0
}

#[derive(Debug, Clone)]
struct Stats {
    uuid: Uuid,
    ip_address: String,
    ping_count: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

async fn update_entry(pool: &mut PgPool, ip: String) {
    let rec = "";
}
