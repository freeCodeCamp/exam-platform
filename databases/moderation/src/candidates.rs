use uuid::Uuid;

use crate::Database;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct NewCandidate {
    pub name: String,
    pub email: String,
}

impl Database {
    /// Case-insensitive.
    pub async fn find_candidate_by_email(
        &self,
        email: &str,
    ) -> Result<Option<Candidate>, sqlx::Error> {
        sqlx::query_as!(
            Candidate,
            "SELECT id, name, email FROM candidates WHERE lower(email) = lower($1)",
            email
        )
        .fetch_optional(&self.pool)
        .await
    }

    /// aAPI only. Inserts `candidate`, returning it with its new id.
    pub async fn insert_candidate(
        &self,
        candidate: NewCandidate,
    ) -> Result<Candidate, sqlx::Error> {
        sqlx::query_as!(
            Candidate,
            "INSERT INTO candidates (name, email) VALUES ($1, $2) RETURNING id, name, email",
            candidate.name,
            candidate.email
        )
        .fetch_one(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    fn new_candidate(email: &str) -> NewCandidate {
        NewCandidate {
            name: "Camper".to_string(),
            email: email.to_string(),
        }
    }

    #[sqlx::test]
    async fn finds_candidate_by_email_case_insensitively(pool: PgPool) -> sqlx::Result<()> {
        let db = Database::from_pool(pool);
        let candidate = db
            .insert_candidate(new_candidate("Camper@freecodecamp.org"))
            .await?;

        let found = db
            .find_candidate_by_email("camper@FREECODECAMP.org")
            .await?;

        assert_eq!(found.map(|c| c.id), Some(candidate.id));
        Ok(())
    }

    #[sqlx::test]
    async fn rejects_email_differing_only_in_case(pool: PgPool) -> sqlx::Result<()> {
        let db = Database::from_pool(pool);
        db.insert_candidate(new_candidate("camper@freecodecamp.org"))
            .await?;

        let duplicate = db
            .insert_candidate(new_candidate("CAMPER@freecodecamp.org"))
            .await;

        assert!(duplicate.is_err());
        Ok(())
    }

    #[sqlx::test]
    async fn edd_reads_but_cannot_write_candidates(pool: PgPool) -> sqlx::Result<()> {
        let db = Database::from_pool(pool.clone());
        db.insert_candidate(new_candidate("camper@freecodecamp.org"))
            .await?;
        let mut tx = pool.begin().await?;
        sqlx::query("SET LOCAL ROLE mdb_edd")
            .execute(&mut *tx)
            .await?;

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM candidates")
            .fetch_one(&mut *tx)
            .await?;
        let insert = sqlx::query("INSERT INTO candidates (name, email) VALUES ('a', 'a@b.c')")
            .execute(&mut *tx)
            .await;

        assert_eq!(count, 1);
        let Err(sqlx::Error::Database(err)) = insert else {
            panic!("insert as mdb_edd succeeded");
        };
        assert_eq!(err.code().as_deref(), Some("42501"));
        Ok(())
    }
}
