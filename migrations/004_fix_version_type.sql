-- Atlas ERP - Fix version column type to match Rust i64 (BIGINT)
-- The config_versions.version column was INT but Rust expects i64/BIGINT

ALTER TABLE _atlas.config_versions
    ALTER COLUMN version TYPE BIGINT;
