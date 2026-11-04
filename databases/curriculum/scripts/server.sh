#!/usr/bin/env bash
# cDb entrypoint.
set -euo pipefail
umask 077
mkdir -p /var/lib/dolt/{databases,config,home}
# No usage metrics, no upstream version checks.
dolt config --global --set metrics.disabled true >/dev/null
dolt config --global --set versioncheck.disabled true >/dev/null
# CREATE DATABASE starts on `dev`.
dolt config --global --set init.defaultbranch dev >/dev/null
[[ "${DOLT_ROOT_PASSWORD:-}" =~ ^[a-f0-9]{64}$ ]] || {
  echo 'CDB_ROOT_PASSWORD: expected 64 hex characters (openssl rand -hex 32)' >&2
  exit 1
}
exec dolt sql-server --config /etc/curriculum/server.yaml
