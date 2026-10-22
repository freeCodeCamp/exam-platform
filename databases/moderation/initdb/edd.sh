#!/bin/sh
# Creates eDd's login role. Postgres runs this on an empty volume only: on an existing one, create
# it by hand. Migration 0001 grants mdb_edd its privileges; it skips creating the role made here.
set -eu
psql -v ON_ERROR_STOP=1 -v password="$MDB_EDD_PASSWORD" \
  --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<'SQL'
CREATE ROLE mdb_edd NOLOGIN;
CREATE ROLE edd LOGIN PASSWORD :'password' IN ROLE mdb_edd;
SQL
