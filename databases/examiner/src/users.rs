use uuid::Uuid;

use crate::Database;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    /// Google account `sub`. `None` for users created through dev login.
    pub google_id: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub google_id: Option<String>,
    pub picture: Option<String>,
}

impl Database {
    /// Case-insensitive.
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, name, email, google_id, picture FROM users WHERE lower(email) = lower($1)",
            email
        )
        .fetch_optional(&self.pool)
        .await
    }

    /// Inserts `user`, returning it with its new id.
    pub async fn insert_user(&self, user: NewUser) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users (name, email, google_id, picture) VALUES ($1, $2, $3, $4)
            RETURNING id, name, email, google_id, picture",
            user.name,
            user.email,
            user.google_id,
            user.picture
        )
        .fetch_one(&self.pool)
        .await
    }

    /// Sets the user's Google id and picture, which can change between logins.
    pub async fn update_user_google_profile(
        &self,
        user_id: Uuid,
        google_id: &str,
        picture: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET google_id = $2, picture = $3, updated_at = now() WHERE id = $1",
            user_id,
            google_id,
            picture
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    fn new_user(email: &str) -> NewUser {
        NewUser {
            name: "Camper".to_string(),
            email: email.to_string(),
            google_id: None,
            picture: None,
        }
    }

    #[sqlx::test]
    async fn finds_user_by_email_case_insensitively(pool: PgPool) -> sqlx::Result<()> {
        let db = Database::from_pool(pool);
        let user = db.insert_user(new_user("Camper@freecodecamp.org")).await?;

        let found = db.find_user_by_email("camper@FREECODECAMP.org").await?;

        assert_eq!(found.map(|u| u.id), Some(user.id));
        Ok(())
    }

    #[sqlx::test]
    async fn rejects_email_differing_only_in_case(pool: PgPool) -> sqlx::Result<()> {
        let db = Database::from_pool(pool);
        db.insert_user(new_user("camper@freecodecamp.org")).await?;

        let duplicate = db.insert_user(new_user("CAMPER@freecodecamp.org")).await;

        assert!(duplicate.is_err());
        Ok(())
    }

    #[sqlx::test]
    async fn updates_google_profile(pool: PgPool) -> sqlx::Result<()> {
        let db = Database::from_pool(pool);
        let user = db.insert_user(new_user("camper@freecodecamp.org")).await?;

        db.update_user_google_profile(user.id, "sub-1", Some("https://example.com/p.png"))
            .await?;

        let found = db
            .find_user_by_email(&user.email)
            .await?
            .expect("user exists");
        assert_eq!(found.google_id.as_deref(), Some("sub-1"));
        assert_eq!(found.picture.as_deref(), Some("https://example.com/p.png"));
        Ok(())
    }
}
