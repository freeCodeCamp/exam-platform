use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::Database;

#[derive(Debug, Clone)]
pub struct Session {
    /// Value of the `sid` cookie. Only its hash is stored.
    pub session_id: String,
    pub user_id: Uuid,
    pub expires_at: OffsetDateTime,
}

fn hash(session_id: &str) -> Vec<u8> {
    Sha256::digest(session_id.as_bytes()).to_vec()
}

impl Database {
    /// Also deletes expired sessions: Postgres has no TTL index, and reads skip them anyway.
    pub async fn insert_session(&self, session: &Session) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "WITH expired AS (DELETE FROM edd.sessions WHERE expires_at <= now())
            INSERT INTO edd.sessions (id_hash, user_id, expires_at) VALUES ($1, $2, $3)",
            hash(&session.session_id),
            session.user_id,
            session.expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Only unexpired sessions.
    pub async fn find_session(&self, session_id: &str) -> Result<Option<Session>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT user_id, expires_at FROM edd.sessions WHERE id_hash = $1 AND expires_at > now()",
            hash(session_id)
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Session {
            session_id: session_id.to_owned(),
            user_id: row.user_id,
            expires_at: row.expires_at,
        }))
    }

    pub async fn delete_user_sessions(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM edd.sessions WHERE user_id = $1", user_id)
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
    use crate::{NewUser, testing::*};

    async fn user_id(db: &Database) -> sqlx::Result<Uuid> {
        let user = db
            .insert_user(NewUser {
                name: "Camper".to_string(),
                email: "camper@freecodecamp.org".to_string(),
                google_id: None,
                picture: None,
            })
            .await?;
        Ok(user.id)
    }

    fn session(id: &str, user_id: Uuid, expires_in: Duration) -> Session {
        Session {
            session_id: id.to_string(),
            user_id,
            expires_at: OffsetDateTime::now_utc() + expires_in,
        }
    }

    #[sqlx::test(migrations = false)]
    async fn stores_only_the_hash(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user_id = user_id(&db).await?;
        db.insert_session(&session("sid", user_id, Duration::hours(1)))
            .await?;

        let stored: Vec<u8> = sqlx::query_scalar("SELECT id_hash FROM edd.sessions")
            .fetch_one(&pool)
            .await?;

        assert_eq!(stored, hash("sid"));
        assert_eq!(
            db.find_session("sid").await?.map(|s| s.user_id),
            Some(user_id)
        );
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn skips_expired_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user_id = user_id(&db).await?;
        db.insert_session(&session("old", user_id, Duration::seconds(-1)))
            .await?;

        assert!(db.find_session("old").await?.is_none());
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn insert_prunes_expired_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user_id = user_id(&db).await?;
        db.insert_session(&session("old", user_id, Duration::seconds(-1)))
            .await?;
        db.insert_session(&session("new", user_id, Duration::hours(1)))
            .await?;

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM edd.sessions")
            .fetch_one(&pool)
            .await?;

        assert_eq!(count, 1);
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn deletes_all_user_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user_id = user_id(&db).await?;
        for id in ["a", "b"] {
            db.insert_session(&session(id, user_id, Duration::hours(1)))
                .await?;
        }

        db.delete_user_sessions(user_id).await?;

        assert!(db.find_session("a").await?.is_none());
        assert!(db.find_session("b").await?.is_none());
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn deleting_user_cascades_to_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user_id = user_id(&db).await?;
        db.insert_session(&session("sid", user_id, Duration::hours(1)))
            .await?;

        sqlx::query("DELETE FROM edd.users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await?;

        assert!(db.find_session("sid").await?.is_none());
        Ok(())
    }

    /// Users and session hashes stay private to eDd.
    #[sqlx::test(migrations = false)]
    async fn aapi_cannot_read_users_or_sessions(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        for sql in [
            "SELECT count(*) FROM edd.users",
            "SELECT count(*) FROM edd.sessions",
        ] {
            let mut tx = pool.begin().await?;
            sqlx::query(set_local_role(Role::Aapi))
                .execute(&mut *tx)
                .await?;

            let result = sqlx::query(sql).execute(&mut *tx).await;

            assert_denied(result);
        }
        Ok(())
    }
}
