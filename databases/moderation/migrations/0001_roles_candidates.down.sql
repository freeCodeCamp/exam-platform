DROP TABLE candidates;

ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE SELECT ON TABLES FROM mdb_edd;
REVOKE SELECT ON _sqlx_migrations FROM mdb_edd;
REVOKE USAGE ON SCHEMA public FROM mdb_edd;
-- Keeps mdb_edd: roles are cluster-wide, and other databases may still use it.
