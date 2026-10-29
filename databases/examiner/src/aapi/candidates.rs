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
            "SELECT id, name, email FROM aapi.candidates WHERE lower(email) = lower($1)",
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
            "INSERT INTO aapi.candidates (name, email) VALUES ($1, $2) RETURNING id, name, email",
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
    use crate::testing::*;

    fn new_candidate(email: &str) -> NewCandidate {
        NewCandidate {
            name: "Camper".to_string(),
            email: email.to_string(),
        }
    }

    #[sqlx::test(migrations = false)]
    async fn finds_candidate_by_email_case_insensitively(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        let candidate = db
            .insert_candidate(new_candidate("Camper@freecodecamp.org"))
            .await?;

        let found = db
            .find_candidate_by_email("camper@FREECODECAMP.org")
            .await?;

        assert_eq!(found.map(|c| c.id), Some(candidate.id));
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn rejects_email_differing_only_in_case(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        db.insert_candidate(new_candidate("camper@freecodecamp.org"))
            .await?;

        let duplicate = db
            .insert_candidate(new_candidate("CAMPER@freecodecamp.org"))
            .await;

        assert!(duplicate.is_err());
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn edd_reads_but_cannot_write_candidates(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let aapi = database_as(&pool, Role::Aapi).await?;
        let candidate = aapi
            .insert_candidate(new_candidate("camper@freecodecamp.org"))
            .await?;
        let edd = database_as(&pool, Role::Edd).await?;

        let found = edd.find_candidate_by_email(&candidate.email).await?;
        let insert = edd
            .insert_candidate(new_candidate("other@freecodecamp.org"))
            .await;

        assert_eq!(found.map(|c| c.id), Some(candidate.id));
        assert_denied(insert);
        Ok(())
    }
}
