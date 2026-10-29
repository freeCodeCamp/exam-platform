-- Roles and schemas, run once as a superuser (or a hosted provider's admin role). Docker runs it on
-- an empty volume; `#[sqlx::test]`s run it per test database. `edb_migrator` then owns every object:
-- migrations create them, and grant the app roles access table by table.
--
-- Roles are cluster-wide: parallel test databases race to create them.
DO $$
BEGIN
  CREATE ROLE edb_migrator LOGIN;
EXCEPTION
  WHEN duplicate_object OR unique_violation THEN NULL;
END
$$;

DO $$
BEGIN
  CREATE ROLE edd LOGIN;
EXCEPTION
  WHEN duplicate_object OR unique_violation THEN NULL;
END
$$;

DO $$
BEGIN
  CREATE ROLE aapi LOGIN;
EXCEPTION
  WHEN duplicate_object OR unique_violation THEN NULL;
END
$$;

-- CONNECT only: PUBLIC's default TEMP would let the app roles create temporary tables.
DO $$
BEGIN
  EXECUTE format('REVOKE ALL ON DATABASE %I FROM PUBLIC', current_database());
  EXECUTE format('GRANT CONNECT ON DATABASE %I TO edb_migrator, edd, aapi', current_database());
END
$$;

-- `edb`: migration history, private to the migrator. `edd` and `aapi`: each app's tables.
CREATE SCHEMA IF NOT EXISTS edb AUTHORIZATION edb_migrator;
CREATE SCHEMA IF NOT EXISTS edd AUTHORIZATION edb_migrator;
CREATE SCHEMA IF NOT EXISTS aapi AUTHORIZATION edb_migrator;
