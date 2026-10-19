//! Persistence for eDd. Stubs until the database is chosen.
use std::time::SystemTime;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
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

#[derive(Debug, Clone)]
pub struct Session {
    /// Value of the `sid` cookie.
    pub session_id: String,
    pub user_id: String,
    pub expires_at: SystemTime,
}

#[derive(Debug, Clone, Default)]
pub struct Database;

impl Database {
    pub async fn find_user_by_email(&self, _email: &str) -> Result<Option<User>, Error> {
        todo!("find user by email")
    }

    /// Inserts `user`, returning it with its new id.
    pub async fn insert_user(&self, _user: NewUser) -> Result<User, Error> {
        todo!("insert user")
    }

    /// Sets the user's Google id and picture, which can change between logins.
    pub async fn update_user_google_profile(
        &self,
        _user_id: &str,
        _google_id: &str,
        _picture: Option<&str>,
    ) -> Result<(), Error> {
        todo!("update user google profile")
    }

    pub async fn insert_session(&self, _session: &Session) -> Result<(), Error> {
        todo!("insert session")
    }

    /// Only unexpired sessions.
    pub async fn find_session(&self, _session_id: &str) -> Result<Option<Session>, Error> {
        todo!("find session")
    }

    pub async fn delete_user_sessions(&self, _user_id: &str) -> Result<(), Error> {
        todo!("delete user sessions")
    }
}
