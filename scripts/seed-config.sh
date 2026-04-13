#!/bin/bash
# Seed initial configuration for Atlas ERP

set -e

DATABASE_URL="${DATABASE_URL:-postgres://atlas:atlas@localhost:5432/atlas}"

echo "🌱 Seeding Atlas configuration..."

# Run migrations in order
echo "📐 Running migrations..."
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/001_init.sql
echo "  ✓ Core schema (001_init.sql)"

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/002_seed_config.sql
echo "  ✓ Seed configuration (002_seed_config.sql)"

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f migrations/003_entity_tables.sql
echo "  ✓ Entity tables (003_entity_tables.sql)"

# Verify entities loaded
echo ""
echo "📊 Verifying entities..."
COUNT=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM _atlas.entities;" 2>/dev/null || echo "0")
echo "  Loaded $COUNT entity definitions"

# List entities
echo ""
echo "📋 Available entities:"
psql "$DATABASE_URL" -c "SELECT name, label, table_name FROM _atlas.entities ORDER BY name;" 2>/dev/null || echo "  (Could not list entities - database may not be ready)"

echo ""
echo "✅ Configuration seeding complete!"
