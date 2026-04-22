#!/bin/bash
# Runs all migration SQL files in alphabetical order on first database init.
# This script is mounted into docker-entrypoint-initdb.d by docker-compose.
set -euo pipefail

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Create internal schema for Atlas metadata
    CREATE SCHEMA IF NOT EXISTS _atlas;
EOSQL

# Apply migrations in order (mounted at /migrations inside the container)
for f in /migrations/*.sql; do
    [ -f "$f" ] || continue
    echo "Applying migration: $(basename "$f")"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$f"
done

echo "All migrations applied successfully."
