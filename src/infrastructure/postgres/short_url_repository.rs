use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};

use crate::ports::short_url_repository::{
    CreateShortUrlRecord, RepositoryError, ShortUrlRecord, ShortUrlRepository,
};

#[derive(Clone)]
pub struct PostgresShortUrlRepository {
    pool: PgPool,
}

impl PostgresShortUrlRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_row(row: PgRow) -> ShortUrlRecord {
    ShortUrlRecord {
        code: row.get("code"),
        long_url: row.get("long_url"),
        access_count: row.get("access_count"),
    }
}

#[async_trait]
impl ShortUrlRepository for PostgresShortUrlRepository {
    async fn create(
        &self,
        record: CreateShortUrlRecord,
    ) -> Result<ShortUrlRecord, RepositoryError> {
        let result = sqlx::query(
            r#"
            INSERT INTO short_urls (code, long_url)
            VALUES ($1, $2)
            RETURNING code, long_url, access_count
            "#,
        )
        .bind(record.code)
        .bind(record.long_url)
        .map(map_row)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(record) => Ok(record),
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
                Err(RepositoryError::DuplicateCode)
            }
            Err(error) => Err(RepositoryError::Other(error.into())),
        }
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<ShortUrlRecord>, RepositoryError> {
        sqlx::query(
            r#"
            SELECT code, long_url, access_count
            FROM short_urls
            WHERE code = $1
            "#,
        )
        .bind(code)
        .map(map_row)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| RepositoryError::Other(error.into()))
    }

    async fn increment_access_count(&self, code: &str, amount: u64) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            UPDATE short_urls
            SET access_count = access_count + $1,
                updated_at = now()
            WHERE code = $2
            "#,
        )
        .bind(amount as i64)
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(|error| RepositoryError::Other(error.into()))?;

        Ok(())
    }
}
