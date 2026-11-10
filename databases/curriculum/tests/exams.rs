mod common;

use cdb::{
    Database,
    edd::exams::create,
    migrations::{self, DEV, Error},
};
use common::TestDb;
use sqlx::ConnectOptions;

impl TestDb {
    pub async fn migrated() -> Result<Self, Error> {
        let db = Self::new().await?.unwrap();
        migrations::migrate(db.options.to_url_lossy().as_str()).await?;
        Ok(db)
    }

    pub async fn pool(&self) -> sqlx::Result<sqlx::MySqlPool> {
        sqlx::mysql::MySqlPoolOptions::new()
            .connect_with(
                self.options
                    .clone()
                    .database(&format!("{}/{DEV}", self.name)),
            )
            .await
    }
}

#[tokio::test]
async fn creates_exam_in_migrated_database() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestDb::migrated().await?;
    let pool = fixture.pool().await?;
    let db = Database::from_pool(pool.clone());

    create(
        &db,
        "exam-id",
        "sample-exam",
        "Sample exam",
        "Read carefully",
    )
    .await?;

    let row: (String, String, String, String, String) =
        sqlx::query_as("SELECT id, slug, title, instructions, status FROM exams WHERE id = ?")
            .bind("exam-id")
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        row,
        (
            "exam-id".into(),
            "sample-exam".into(),
            "Sample exam".into(),
            "Read carefully".into(),
            "disabled".into(),
        )
    );
    pool.close().await;
    fixture.destroy().await?;
    Ok(())
}
