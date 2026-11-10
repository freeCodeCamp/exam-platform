use time::OffsetDateTime;
use uuid::Uuid;

use crate::{Database, session_id_hash};

#[derive(Debug, Clone)]
pub struct CandidateSession {
    pub session_id: String,
    pub candidate_id: Uuid,
    pub expires_at: OffsetDateTime,
}

impl Database {
    pub async fn insert_candidate_session(
        &self,
        session: &CandidateSession,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "WITH expired AS (DELETE FROM aapi.sessions WHERE expires_at <= now())
            INSERT INTO aapi.sessions (id_hash, candidate_id, expires_at) VALUES ($1, $2, $3)",
            session_id_hash(&session.session_id),
            session.candidate_id,
            session.expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_candidate_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CandidateSession>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT candidate_id, expires_at FROM aapi.sessions WHERE id_hash = $1 AND expires_at > now()",
            session_id_hash(session_id)
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| CandidateSession {
            session_id: session_id.to_owned(),
            candidate_id: row.candidate_id,
            expires_at: row.expires_at,
        }))
    }

    pub async fn delete_candidate_sessions(&self, candidate_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM aapi.sessions WHERE candidate_id = $1",
            candidate_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use time::Duration;

    use super::*;
    use crate::{NewCandidate, testing::*};

    async fn candidate_id(db: &Database) -> sqlx::Result<Uuid> {
        let candidate = db
            .insert_candidate(NewCandidate {
                name: "Camper".to_string(),
                email: "camper@example.com".to_string(),
            })
            .await?;
        Ok(candidate.id)
    }

    fn session(id: &str, candidate_id: Uuid, expires_in: Duration) -> CandidateSession {
        CandidateSession {
            session_id: id.to_string(),
            candidate_id,
            expires_at: OffsetDateTime::now_utc() + expires_in,
        }
    }

    #[sqlx::test(migrations = false)]
    async fn stores_only_the_hash(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        let candidate_id = candidate_id(&db).await?;
        db.insert_candidate_session(&session("sid", candidate_id, Duration::hours(1)))
            .await?;

        let stored: Vec<u8> = sqlx::query_scalar("SELECT id_hash FROM aapi.sessions")
            .fetch_one(&pool)
            .await?;

        assert_eq!(stored, session_id_hash("sid"));
        assert_eq!(
            db.find_candidate_session("sid")
                .await?
                .map(|s| s.candidate_id),
            Some(candidate_id)
        );
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn skips_expired_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        let candidate_id = candidate_id(&db).await?;
        db.insert_candidate_session(&session("old", candidate_id, Duration::seconds(-1)))
            .await?;

        assert!(db.find_candidate_session("old").await?.is_none());
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn insert_prunes_expired_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        let candidate_id = candidate_id(&db).await?;
        db.insert_candidate_session(&session("old", candidate_id, Duration::seconds(-1)))
            .await?;
        db.insert_candidate_session(&session("new", candidate_id, Duration::hours(1)))
            .await?;

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM aapi.sessions")
            .fetch_one(&pool)
            .await?;

        assert_eq!(count, 1);
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn deletes_all_candidate_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        let candidate_id = candidate_id(&db).await?;
        for id in ["a", "b"] {
            db.insert_candidate_session(&session(id, candidate_id, Duration::hours(1)))
                .await?;
        }

        db.delete_candidate_sessions(candidate_id).await?;

        assert!(db.find_candidate_session("a").await?.is_none());
        assert!(db.find_candidate_session("b").await?.is_none());
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn deleting_candidate_cascades_to_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Aapi).await?;
        let candidate_id = candidate_id(&db).await?;
        db.insert_candidate_session(&session("sid", candidate_id, Duration::hours(1)))
            .await?;

        sqlx::query("DELETE FROM aapi.candidates WHERE id = $1")
            .bind(candidate_id)
            .execute(&pool)
            .await?;

        assert!(db.find_candidate_session("sid").await?.is_none());
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn edd_cannot_read_candidate_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let mut tx = pool.begin().await?;
        sqlx::query(set_local_role(Role::Edd))
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("SELECT count(*) FROM aapi.sessions")
            .execute(&mut *tx)
            .await;

        assert_denied(result);
        Ok(())
    }
}
