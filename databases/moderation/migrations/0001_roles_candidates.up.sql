-- eDd's privileges (docs/auth.md). Roles are cluster-wide: `edd` logs in as a member of this
-- role, created outside migrations. Parallel `#[sqlx::test]` databases race to create it.
DO $$
BEGIN
  CREATE ROLE mdb_edd NOLOGIN;
EXCEPTION
  WHEN duplicate_object OR unique_violation THEN NULL;
END
$$;

GRANT USAGE ON SCHEMA public TO mdb_edd;
-- For `Database::connect_checked`. Exists before this migration, so default privileges miss it.
GRANT SELECT ON _sqlx_migrations TO mdb_edd;
-- eDd reads all candidate data. Grant writes per table, in the migration that adds it.
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO mdb_edd;

CREATE TABLE candidates (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  email text NOT NULL,
  name text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Emails match case-insensitively: query with `lower(email) = lower($1)`.
CREATE UNIQUE INDEX candidates_email_key ON candidates (lower(email));
