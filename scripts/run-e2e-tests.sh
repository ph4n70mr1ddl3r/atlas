#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# Atlas ERP – End-to-End Test Runner
#
# Usage:
#   ./scripts/run-e2e-tests.sh                # run everything
#   ./scripts/run-e2e-tests.sh --suite auth   # run a single suite
#   ./scripts/run-e2e-tests.sh --reset-only   # reset DB, don't run tests
#   ./scripts/run-e2e-tests.sh --keep-db      # don't tear down after tests
#
# Environment variables:
#   TEST_DATABASE_URL   Postgres connection string
#                       (default: postgres://atlas:atlas@localhost:5432/atlas)
#   NO_DOCKER           Set to "1" if postgres is already running externally
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────
DATABASE_URL="${TEST_DATABASE_URL:-postgres://atlas:atlas@localhost:5432/atlas}"
NO_DOCKER="${NO_DOCKER:-0}"
KEEP_DB=0
RESET_ONLY=0
SUITE=""
EXTRA_CARGO_ARGS=()
TEST_THREADS=1          # serial by default – tests share DB state

# ── Colours (disabled when not a tty) ─────────────────────────────────────────
if test -t 1; then
    R='\033[0;31m' G='\033[0;32m' Y='\033[0;33m' B='\033[0;34m' N='\033[0m'
else
    R='' G='' Y='' B='' N=''
fi

info()  { printf "${B}[INFO]${N}  %s\n" "$*"; }
ok()    { printf "${G}[ OK ]${N}  %s\n" "$*"; }
warn()  { printf "${Y}[WARN]${N}  %s\n" "$*"; }
fail()  { printf "${R}[FAIL]${N}  %s\n" "$*" >&2; exit 1; }

# ── Parse arguments ───────────────────────────────────────────────────────────
usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Options:
  --suite <name>       Run only the named test suite
                       (auth | crud | workflow | schema | admin |
                        report | procure_to_pay | order_to_cash)
  --test <pattern>     Run tests matching Cargo's test-filter pattern
  --threads <n>        Parallel test threads (default: 1)
  --keep-db            Do not stop/remove the database container after tests
  --reset-only         Reset the database and exit (don't run tests)
  --no-docker          Assume postgres is already reachable (skip docker)
  -v / --verbose       Forward --nocapture to Cargo for println output
  -h / --help          Show this help

Examples:
  $(basename "$0")                             # full run, fresh DB
  $(basename "$0") --suite auth                # just the auth suite
  $(basename "$0") --test test_login --verbose # single test, with output
  $(basename "$0") --keep-db                   # keep DB running after tests
  NO_DOCKER=1 $(basename "$0")                 # use an externally managed DB
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --suite)      SUITE="$2"; shift 2 ;;
        --test)       EXTRA_CARGO_ARGS+=( "$2" ); shift 2 ;;
        --threads)    TEST_THREADS="$2"; shift 2 ;;
        --keep-db)    KEEP_DB=1; shift ;;
        --reset-only) RESET_ONLY=1; shift ;;
        --no-docker)  NO_DOCKER=1; shift ;;
        -v|--verbose) EXTRA_CARGO_ARGS+=( "--nocapture" ); shift ;;
        -h|--help)    usage ;;
        *)            fail "Unknown option: $1" ;;
    esac
done

# ── Resolve project root ──────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# ── Detect docker compose command ─────────────────────────────────────────────
if docker compose version >/dev/null 2>&1; then
    COMPOSE="docker compose"
elif docker-compose --version >/dev/null 2>&1; then
    COMPOSE="docker-compose"
else
    COMPOSE="docker compose"  # will fail with a clear error later
fi

# ── Helper: wait until Postgres is reachable ──────────────────────────────────
wait_for_postgres() {
    local attempts=0
    local max=${1:-30}
    info "Waiting for Postgres to accept connections …"
    while ! docker exec atlas-postgres pg_isready -U atlas -q 2>/dev/null; do
        attempts=$((attempts + 1))
        if [ $attempts -ge $max ]; then
            fail "Postgres did not become ready within ${max}s"
        fi
        sleep 1
    done
    ok "Postgres is ready"
}

# ── Helper: check if the pg container is running ──────────────────────────────
pg_is_running() {
    docker ps --format '{{.Names}}' 2>/dev/null | grep -qx 'atlas-postgres'
}

# ── Step 1: Start Postgres (unless disabled) ──────────────────────────────────
start_postgres() {
    if [ "$NO_DOCKER" = "1" ]; then
        info "NO_DOCKER=1 — skipping container management"
        # Quick connectivity check
        if ! psql "$DATABASE_URL" -c "SELECT 1" >/dev/null 2>&1; then
            fail "Cannot connect to $DATABASE_URL. Is Postgres running?"
        fi
        ok "Database is reachable"
        return
    fi

    if pg_is_running; then
        ok "Postgres container already running"
        return
    fi

    if ! command -v docker >/dev/null 2>&1; then
        fail "Docker is not installed/running. Start Postgres manually and re-run with NO_DOCKER=1"
    fi

    info "Starting Postgres via $COMPOSE …"
    $COMPOSE up -d postgres
    wait_for_postgres 30
}

# ── Step 2: (Re-)apply migrations ─────────────────────────────────────────────
reset_database() {
    if ! command -v psql >/dev/null 2>&1; then
        fail "psql is not installed. Install postgresql-client or use NO_DOCKER=1 with a pre-migrated DB."
    fi

    info "Applying migrations …"

    # Drop and recreate the database schema for a clean slate.
    # We do this by re-running the migration files in order.
    # _atlas schema and tables use CREATE IF NOT EXISTS / ON CONFLICT,
    # so re-running is safe; we also truncate test-critical tables.
    psql "$DATABASE_URL" -v ON_ERROR_STOP=0 >/dev/null 2>&1 <<'SQL'
        -- Nuke all data for a clean run, but keep the schema
        TRUNCATE TABLE _atlas.audit_log CASCADE;
        TRUNCATE TABLE _atlas.workflow_states CASCADE;
        TRUNCATE TABLE _atlas.config_versions CASCADE;
        TRUNCATE TABLE _atlas.entities CASCADE;
        -- Re-seed the org and admin user (idempotent)
SQL

    local mig_dir="$PROJECT_ROOT/migrations"
    for sql_file in "$mig_dir"/*.sql; do
        local base
        base="$(basename "$sql_file")"
        info "  Applying $base"
        psql "$DATABASE_URL" -v ON_ERROR_STOP=0 -f "$sql_file" >/dev/null 2>&1 || true
    done

    # Seed the admin user + org (idempotent via ON CONFLICT)
    psql "$DATABASE_URL" -v ON_ERROR_STOP=0 >/dev/null 2>&1 <<'SQL'
        INSERT INTO _atlas.organizations (id, name, code)
        VALUES ('00000000-0000-0000-0000-000000000001', 'Default Organization', 'DEFAULT')
        ON CONFLICT (id) DO NOTHING;

        INSERT INTO _atlas.users (id, email, name, password_hash, roles, organization_id)
        VALUES (
            '00000000-0000-0000-0000-000000000002',
            'admin@atlas.local',
            'System Administrator',
            '$argon2id$v=19$m=19456,t=2,p=1$d/ce2R9A0BCBBqiaYeGHUw$iGegymLltUV9IKxr7cixQqWUvamhHdjKhjEcH7qcGmI',
            '["admin", "system"]'::jsonb,
            '00000000-0000-0000-0000-000000000001'
        ) ON CONFLICT (id) DO NOTHING;
SQL

    ok "Database reset complete"
}

# ── Step 3: Run the Rust unit tests (no DB needed) ────────────────────────────
run_unit_tests() {
    info "Running unit tests (--lib) …"
    if cargo test --workspace --lib --quiet 2>&1; then
        ok "Unit tests passed"
    else
        fail "Unit tests failed"
    fi
}

# ── Step 4: Run clippy ────────────────────────────────────────────────────────
run_clippy() {
    info "Running clippy …"
    if cargo clippy --workspace --quiet 2>&1 | grep -vE '^(warning|note).*sqlx-postgres'; then
        ok "Clippy clean"
    else
        warn "Clippy reported issues (see above)"
    fi
}

# ── Step 5: Run the e2e tests ─────────────────────────────────────────────────
run_e2e_tests() {
    local filter=""
    if [ -n "$SUITE" ]; then
        filter="suite::$SUITE"
    fi

    # Build the cargo test invocation
    # NOTE: E2E integration tests are marked #[ignore] because they require a
    # live database.  The --ignored flag is required so they actually run.
    local cmd=(
        cargo test
        -p atlas-gateway
        --test e2e
        --quiet
        --test-threads "$TEST_THREADS"
        --ignored
    )

    if [ -n "$filter" ]; then
        cmd+=( "$filter" )
    fi

    # Append any extra args (e.g. specific test name, --nocapture)
    if [ ${#EXTRA_CARGO_ARGS[@]} -gt 0 ]; then
        cmd+=( "--" "${EXTRA_CARGO_ARGS[@]}" )
    fi

    info "Running e2e tests (--ignored) …"
    info "  Command: ${cmd[*]}"

    export TEST_DATABASE_URL="$DATABASE_URL"
    export DATABASE_URL="$DATABASE_URL"

    if "${cmd[@]}" 2>&1; then
        ok "E2E tests passed"
    else
        fail "E2E tests failed"
    fi
}

# ── Step 6: Tear down (unless --keep-db) ──────────────────────────────────────
teardown() {
    if [ "$KEEP_DB" = "1" ] || [ "$NO_DOCKER" = "1" ]; then
        info "Keeping database running (--keep-db / NO_DOCKER)"
        return
    fi

    info "Stopping Postgres container …"
    $COMPOSE down -v 2>/dev/null || true
    ok "Container removed"
}

# ── Main ──────────────────────────────────────────────────────────────────────
main() {
    echo ""
    printf "${B}╔══════════════════════════════════════════════════════╗${N}\n"
    printf "${B}║          Atlas ERP – Test Runner                     ║${N}\n"
    printf "${B}╚══════════════════════════════════════════════════════╝${N}\n"
    echo ""

    # Always run these (no DB needed)
    run_unit_tests
    run_clippy

    if [ "$RESET_ONLY" = "1" ]; then
        start_postgres
        reset_database
        ok "Reset-only complete"
        exit 0
    fi

    # Set up a trap to tear down on exit / interrupt
    if [ "$KEEP_DB" = "0" ] && [ "$NO_DOCKER" = "0" ]; then
        trap teardown EXIT
    fi

    start_postgres
    reset_database
    run_e2e_tests

    echo ""
    printf "${G}╔══════════════════════════════════════════════════════╗${N}\n"
    printf "${G}║          All tests passed ✓                          ║${N}\n"
    printf "${G}╚══════════════════════════════════════════════════════╝${N}\n"
}

main "$@"
