#!/bin/sh
# Sets the login roles' passwords. Postgres runs initdb/ on an empty volume only: on an existing
# one, run 10-bootstrap.sql and these `ALTER ROLE`s by hand.
set -eu
psql -v ON_ERROR_STOP=1 -v migrator="$EDB_MIGRATOR_PASSWORD" -v edd="$EDB_EDD_PASSWORD" \
  -v aapi="$EDB_AAPI_PASSWORD" --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<'SQL'
ALTER ROLE edb_migrator PASSWORD :'migrator';
ALTER ROLE edd PASSWORD :'edd';
ALTER ROLE aapi PASSWORD :'aapi';
SQL
