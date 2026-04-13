#!/bin/bash
# Seed initial configuration for Atlas ERP

set -e

DATABASE_URL="${DATABASE_URL:-postgres://atlas:atlas@localhost:5432/atlas}"

echo "Seeding Atlas configuration..."

# Run migrations
echo "Running migrations..."
psql "$DATABASE_URL" -f migrations/001_init.sql
psql "$DATABASE_URL" -f migrations/002_seed_config.sql
psql "$DATABASE_URL" -f migrations/003_entity_tables.sql

# Verify entities loaded
echo "Verifying entities..."
COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM _atlas.entities;")
echo "Loaded $COUNT entity definitions"

# List entities
echo "Available entities:"
psql "$DATABASE_URL" -c "SELECT name, label FROM _atlas.entities ORDER BY name;"

echo "Configuration seeding complete!"
