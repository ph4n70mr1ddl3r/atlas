//! Descriptive Flexfield Repository
//!
//! PostgreSQL storage for flexfields, contexts, segments, value sets,
//! value set entries, and flexfield data.

use atlas_shared::{
    DescriptiveFlexfield, FlexfieldContext, FlexfieldSegment,
    FlexfieldValueSet, FlexfieldValueSetEntry, FlexfieldData,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for descriptive flexfield storage
#[async_trait]
pub trait DescriptiveFlexfieldRepository: Send + Sync {
    // ── Flexfields ──────────────────────────────────────────────
    async fn create_flexfield(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        entity_name: &str,
        context_column: &str,
        default_context_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DescriptiveFlexfield>;

    async fn get_flexfield(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DescriptiveFlexfield>>;
    async fn get_flexfield_by_id(&self, id: Uuid) -> AtlasResult<Option<DescriptiveFlexfield>>;
    async fn get_flexfield_by_entity(&self, org_id: Uuid, entity_name: &str) -> AtlasResult<Option<DescriptiveFlexfield>>;
    async fn list_flexfields(&self, org_id: Uuid) -> AtlasResult<Vec<DescriptiveFlexfield>>;
    async fn update_flexfield_active(&self, id: Uuid, is_active: bool) -> AtlasResult<DescriptiveFlexfield>;
    async fn delete_flexfield(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Contexts ────────────────────────────────────────────────
    async fn create_context(
        &self,
        org_id: Uuid,
        flexfield_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        is_global: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldContext>;

    async fn get_context(&self, id: Uuid) -> AtlasResult<Option<FlexfieldContext>>;
    async fn list_contexts(&self, flexfield_id: Uuid) -> AtlasResult<Vec<FlexfieldContext>>;
    async fn get_context_by_code(&self, flexfield_id: Uuid, code: &str) -> AtlasResult<Option<FlexfieldContext>>;
    async fn update_context_enabled(&self, id: Uuid, is_enabled: bool) -> AtlasResult<FlexfieldContext>;
    async fn delete_context(&self, id: Uuid) -> AtlasResult<()>;

    // ── Segments ────────────────────────────────────────────────
    async fn create_segment(
        &self,
        org_id: Uuid,
        flexfield_id: Uuid,
        context_id: Uuid,
        segment_code: &str,
        name: &str,
        description: Option<&str>,
        display_order: i32,
        column_name: &str,
        data_type: &str,
        is_required: bool,
        is_read_only: bool,
        is_visible: bool,
        default_value: Option<&str>,
        value_set_id: Option<Uuid>,
        value_set_code: Option<&str>,
        help_text: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldSegment>;

    async fn get_segment(&self, id: Uuid) -> AtlasResult<Option<FlexfieldSegment>>;
    async fn list_segments_by_context(&self, context_id: Uuid) -> AtlasResult<Vec<FlexfieldSegment>>;
    async fn list_segments_by_flexfield(&self, flexfield_id: Uuid) -> AtlasResult<Vec<FlexfieldSegment>>;
    async fn get_segment_by_code(&self, context_id: Uuid, segment_code: &str) -> AtlasResult<Option<FlexfieldSegment>>;
    async fn update_segment(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        display_order: Option<i32>,
        is_required: Option<bool>,
        is_read_only: Option<bool>,
        is_visible: Option<bool>,
        default_value: Option<&str>,
        value_set_id: Option<Uuid>,
        value_set_code: Option<&str>,
    ) -> AtlasResult<FlexfieldSegment>;
    async fn delete_segment(&self, id: Uuid) -> AtlasResult<()>;

    // ── Value Sets ──────────────────────────────────────────────
    async fn create_value_set(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        validation_type: &str,
        data_type: &str,
        max_length: i32,
        min_length: i32,
        format_mask: Option<&str>,
        table_validation: Option<serde_json::Value>,
        independent_values: Option<serde_json::Value>,
        parent_value_set_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldValueSet>;

    async fn get_value_set(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FlexfieldValueSet>>;
    async fn get_value_set_by_id(&self, id: Uuid) -> AtlasResult<Option<FlexfieldValueSet>>;
    async fn list_value_sets(&self, org_id: Uuid) -> AtlasResult<Vec<FlexfieldValueSet>>;
    async fn delete_value_set(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ── Value Set Entries ───────────────────────────────────────
    async fn create_value_set_entry(
        &self,
        org_id: Uuid,
        value_set_id: Uuid,
        value: &str,
        meaning: Option<&str>,
        description: Option<&str>,
        parent_value: Option<&str>,
        is_enabled: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        sort_order: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldValueSetEntry>;

    async fn list_value_set_entries(&self, value_set_id: Uuid, parent_value: Option<&str>) -> AtlasResult<Vec<FlexfieldValueSetEntry>>;
    async fn delete_value_set_entry(&self, id: Uuid) -> AtlasResult<()>;
    async fn validate_value(&self, value_set_id: Uuid, value: &str, parent_value: Option<&str>) -> AtlasResult<bool>;

    // ── Flexfield Data ──────────────────────────────────────────
    async fn set_flexfield_data(
        &self,
        org_id: Uuid,
        flexfield_id: Uuid,
        entity_name: &str,
        entity_id: Uuid,
        context_code: &str,
        segment_values: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldData>;

    async fn get_flexfield_data(
        &self,
        entity_name: &str,
        entity_id: Uuid,
        context_code: Option<&str>,
    ) -> AtlasResult<Vec<FlexfieldData>>;

    async fn delete_flexfield_data(
        &self,
        entity_name: &str,
        entity_id: Uuid,
    ) -> AtlasResult<()>;

    // ── Dashboard ───────────────────────────────────────────────
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::FlexfieldDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresDescriptiveFlexfieldRepository {
    pool: PgPool,
}

impl PostgresDescriptiveFlexfieldRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_flexfield(row: &sqlx::postgres::PgRow) -> DescriptiveFlexfield {
    DescriptiveFlexfield {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        entity_name: row.get("entity_name"),
        context_column: row.get("context_column"),
        default_context_code: row.get("default_context_code"),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_context(row: &sqlx::postgres::PgRow) -> FlexfieldContext {
    FlexfieldContext {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        flexfield_id: row.get("flexfield_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        is_global: row.get("is_global"),
        is_enabled: row.get("is_enabled"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_segment(row: &sqlx::postgres::PgRow) -> FlexfieldSegment {
    FlexfieldSegment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        flexfield_id: row.get("flexfield_id"),
        context_id: row.get("context_id"),
        segment_code: row.get("segment_code"),
        name: row.get("name"),
        description: row.get("description"),
        display_order: row.get("display_order"),
        column_name: row.get("column_name"),
        data_type: row.get("data_type"),
        is_required: row.get("is_required"),
        is_read_only: row.get("is_read_only"),
        is_visible: row.get("is_visible"),
        default_value: row.get("default_value"),
        value_set_id: row.get("value_set_id"),
        value_set_code: row.get("value_set_code"),
        help_text: row.get("help_text"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_value_set(row: &sqlx::postgres::PgRow) -> FlexfieldValueSet {
    FlexfieldValueSet {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        validation_type: row.get("validation_type"),
        data_type: row.get("data_type"),
        max_length: row.get("max_length"),
        min_length: row.get("min_length"),
        format_mask: row.get("format_mask"),
        table_validation: row.get("table_validation"),
        independent_values: row.get("independent_values"),
        parent_value_set_code: row.get("parent_value_set_code"),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_value_set_entry(row: &sqlx::postgres::PgRow) -> FlexfieldValueSetEntry {
    FlexfieldValueSetEntry {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        value_set_id: row.get("value_set_id"),
        value: row.get("value"),
        meaning: row.get("meaning"),
        description: row.get("description"),
        parent_value: row.get("parent_value"),
        is_enabled: row.get("is_enabled"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        sort_order: row.get("sort_order"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_flexfield_data(row: &sqlx::postgres::PgRow) -> FlexfieldData {
    FlexfieldData {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        flexfield_id: row.get("flexfield_id"),
        entity_name: row.get("entity_name"),
        entity_id: row.get("entity_id"),
        context_code: row.get("context_code"),
        segment_values: row.get("segment_values"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl DescriptiveFlexfieldRepository for PostgresDescriptiveFlexfieldRepository {
    // ── Flexfields ──────────────────────────────────────────────

    async fn create_flexfield(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        entity_name: &str, context_column: &str,
        default_context_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DescriptiveFlexfield> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.dff_flexfields
                (organization_id, code, name, description, entity_name,
                 context_column, default_context_code, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(entity_name).bind(context_column).bind(default_context_code)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_flexfield(&row))
    }

    async fn get_flexfield(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DescriptiveFlexfield>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_flexfields WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_flexfield(&r)))
    }

    async fn get_flexfield_by_id(&self, id: Uuid) -> AtlasResult<Option<DescriptiveFlexfield>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_flexfields WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_flexfield(&r)))
    }

    async fn get_flexfield_by_entity(&self, org_id: Uuid, entity_name: &str) -> AtlasResult<Option<DescriptiveFlexfield>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_flexfields WHERE organization_id=$1 AND entity_name=$2 AND is_active=true"
        )
        .bind(org_id).bind(entity_name)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_flexfield(&r)))
    }

    async fn list_flexfields(&self, org_id: Uuid) -> AtlasResult<Vec<DescriptiveFlexfield>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.dff_flexfields WHERE organization_id=$1 ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_flexfield).collect())
    }

    async fn update_flexfield_active(&self, id: Uuid, is_active: bool) -> AtlasResult<DescriptiveFlexfield> {
        let row = sqlx::query(
            "UPDATE _atlas.dff_flexfields SET is_active=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(is_active)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_flexfield(&row))
    }

    async fn delete_flexfield(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.dff_flexfields WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Contexts ────────────────────────────────────────────────

    async fn create_context(
        &self,
        org_id: Uuid, flexfield_id: Uuid, code: &str, name: &str,
        description: Option<&str>, is_global: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldContext> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.dff_contexts
                (organization_id, flexfield_id, code, name, description, is_global, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *"#,
        )
        .bind(org_id).bind(flexfield_id).bind(code).bind(name)
        .bind(description).bind(is_global).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_context(&row))
    }

    async fn get_context(&self, id: Uuid) -> AtlasResult<Option<FlexfieldContext>> {
        let row = sqlx::query("SELECT * FROM _atlas.dff_contexts WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_context(&r)))
    }

    async fn list_contexts(&self, flexfield_id: Uuid) -> AtlasResult<Vec<FlexfieldContext>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.dff_contexts WHERE flexfield_id=$1 ORDER BY code"
        )
        .bind(flexfield_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_context).collect())
    }

    async fn get_context_by_code(&self, flexfield_id: Uuid, code: &str) -> AtlasResult<Option<FlexfieldContext>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_contexts WHERE flexfield_id=$1 AND code=$2"
        )
        .bind(flexfield_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_context(&r)))
    }

    async fn update_context_enabled(&self, id: Uuid, is_enabled: bool) -> AtlasResult<FlexfieldContext> {
        let row = sqlx::query(
            "UPDATE _atlas.dff_contexts SET is_enabled=$2, updated_at=now() WHERE id=$1 RETURNING *"
        )
        .bind(id).bind(is_enabled)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_context(&row))
    }

    async fn delete_context(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.dff_contexts WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Segments ────────────────────────────────────────────────

    async fn create_segment(
        &self,
        org_id: Uuid, flexfield_id: Uuid, context_id: Uuid,
        segment_code: &str, name: &str, description: Option<&str>,
        display_order: i32, column_name: &str, data_type: &str,
        is_required: bool, is_read_only: bool, is_visible: bool,
        default_value: Option<&str>,
        value_set_id: Option<Uuid>, value_set_code: Option<&str>,
        help_text: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldSegment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.dff_segments
                (organization_id, flexfield_id, context_id, segment_code, name, description,
                 display_order, column_name, data_type, is_required, is_read_only, is_visible,
                 default_value, value_set_id, value_set_code, help_text, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(flexfield_id).bind(context_id)
        .bind(segment_code).bind(name).bind(description)
        .bind(display_order).bind(column_name).bind(data_type)
        .bind(is_required).bind(is_read_only).bind(is_visible)
        .bind(default_value).bind(value_set_id).bind(value_set_code)
        .bind(help_text).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_segment(&row))
    }

    async fn get_segment(&self, id: Uuid) -> AtlasResult<Option<FlexfieldSegment>> {
        let row = sqlx::query("SELECT * FROM _atlas.dff_segments WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_segment(&r)))
    }

    async fn list_segments_by_context(&self, context_id: Uuid) -> AtlasResult<Vec<FlexfieldSegment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.dff_segments WHERE context_id=$1 ORDER BY display_order"
        )
        .bind(context_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_segment).collect())
    }

    async fn list_segments_by_flexfield(&self, flexfield_id: Uuid) -> AtlasResult<Vec<FlexfieldSegment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.dff_segments WHERE flexfield_id=$1 ORDER BY context_id, display_order"
        )
        .bind(flexfield_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_segment).collect())
    }

    async fn get_segment_by_code(&self, context_id: Uuid, segment_code: &str) -> AtlasResult<Option<FlexfieldSegment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_segments WHERE context_id=$1 AND segment_code=$2"
        )
        .bind(context_id).bind(segment_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_segment(&r)))
    }

    async fn update_segment(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        display_order: Option<i32>,
        is_required: Option<bool>,
        is_read_only: Option<bool>,
        is_visible: Option<bool>,
        default_value: Option<&str>,
        value_set_id: Option<Uuid>,
        value_set_code: Option<&str>,
    ) -> AtlasResult<FlexfieldSegment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.dff_segments SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                display_order = COALESCE($4, display_order),
                is_required = COALESCE($5, is_required),
                is_read_only = COALESCE($6, is_read_only),
                is_visible = COALESCE($7, is_visible),
                default_value = COALESCE($8, default_value),
                value_set_id = COALESCE($9, value_set_id),
                value_set_code = COALESCE($10, value_set_code),
                updated_at = now()
            WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(name).bind(description).bind(display_order)
        .bind(is_required).bind(is_read_only).bind(is_visible)
        .bind(default_value).bind(value_set_id).bind(value_set_code)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_segment(&row))
    }

    async fn delete_segment(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.dff_segments WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Value Sets ──────────────────────────────────────────────

    async fn create_value_set(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        validation_type: &str, data_type: &str,
        max_length: i32, min_length: i32,
        format_mask: Option<&str>,
        table_validation: Option<serde_json::Value>,
        independent_values: Option<serde_json::Value>,
        parent_value_set_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldValueSet> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.dff_value_sets
                (organization_id, code, name, description, validation_type, data_type,
                 max_length, min_length, format_mask, table_validation,
                 independent_values, parent_value_set_code, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(validation_type).bind(data_type)
        .bind(max_length).bind(min_length).bind(format_mask)
        .bind(table_validation).bind(independent_values)
        .bind(parent_value_set_code).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_value_set(&row))
    }

    async fn get_value_set(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FlexfieldValueSet>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_value_sets WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_value_set(&r)))
    }

    async fn get_value_set_by_id(&self, id: Uuid) -> AtlasResult<Option<FlexfieldValueSet>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.dff_value_sets WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_value_set(&r)))
    }

    async fn list_value_sets(&self, org_id: Uuid) -> AtlasResult<Vec<FlexfieldValueSet>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.dff_value_sets WHERE organization_id=$1 ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_value_set).collect())
    }

    async fn delete_value_set(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.dff_value_sets WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Value Set Entries ───────────────────────────────────────

    async fn create_value_set_entry(
        &self,
        org_id: Uuid, value_set_id: Uuid, value: &str,
        meaning: Option<&str>, description: Option<&str>,
        parent_value: Option<&str>, is_enabled: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        sort_order: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldValueSetEntry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.dff_value_set_entries
                (organization_id, value_set_id, value, meaning, description,
                 parent_value, is_enabled, effective_from, effective_to,
                 sort_order, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(value_set_id).bind(value)
        .bind(meaning).bind(description)
        .bind(parent_value).bind(is_enabled)
        .bind(effective_from).bind(effective_to)
        .bind(sort_order).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_value_set_entry(&row))
    }

    async fn list_value_set_entries(&self, value_set_id: Uuid, parent_value: Option<&str>) -> AtlasResult<Vec<FlexfieldValueSetEntry>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.dff_value_set_entries
            WHERE value_set_id=$1
              AND ($2::text IS NULL OR parent_value=$2)
              AND is_enabled = true
            ORDER BY sort_order, value"#,
        )
        .bind(value_set_id).bind(parent_value)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_value_set_entry).collect())
    }

    async fn delete_value_set_entry(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.dff_value_set_entries WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn validate_value(&self, value_set_id: Uuid, value: &str, parent_value: Option<&str>) -> AtlasResult<bool> {
        let row = sqlx::query(
            r#"SELECT COUNT(*) as cnt FROM _atlas.dff_value_set_entries
            WHERE value_set_id=$1
              AND value=$2
              AND is_enabled = true
              AND ($3::text IS NULL OR parent_value=$3)
              AND (effective_from IS NULL OR effective_from <= CURRENT_DATE)
              AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)"#,
        )
        .bind(value_set_id).bind(value).bind(parent_value)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let count: i64 = row.get("cnt");
        Ok(count > 0)
    }

    // ── Flexfield Data ──────────────────────────────────────────

    async fn set_flexfield_data(
        &self,
        org_id: Uuid, flexfield_id: Uuid, entity_name: &str,
        entity_id: Uuid, context_code: &str,
        segment_values: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FlexfieldData> {
        // Upsert: if data exists for this entity+context, update it
        let existing = sqlx::query(
            r#"SELECT id FROM _atlas.dff_data
            WHERE entity_name=$1 AND entity_id=$2 AND context_code=$3"#,
        )
        .bind(entity_name).bind(entity_id).bind(context_code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            let row = sqlx::query(
                r#"UPDATE _atlas.dff_data
                SET segment_values = $1, updated_at = now()
                WHERE entity_name = $2 AND entity_id = $3 AND context_code = $4
                RETURNING *"#,
            )
            .bind(&segment_values).bind(entity_name).bind(entity_id).bind(context_code)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
            Ok(row_to_flexfield_data(&row))
        } else {
            let row = sqlx::query(
                r#"INSERT INTO _atlas.dff_data
                    (organization_id, flexfield_id, entity_name, entity_id,
                     context_code, segment_values, created_by)
                VALUES ($1,$2,$3,$4,$5,$6,$7)
                RETURNING *"#,
            )
            .bind(org_id).bind(flexfield_id).bind(entity_name)
            .bind(entity_id).bind(context_code).bind(&segment_values)
            .bind(created_by)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
            Ok(row_to_flexfield_data(&row))
        }
    }

    async fn get_flexfield_data(
        &self,
        entity_name: &str,
        entity_id: Uuid,
        context_code: Option<&str>,
    ) -> AtlasResult<Vec<FlexfieldData>> {
        let rows = if context_code.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.dff_data WHERE entity_name=$1 AND entity_id=$2 AND context_code=$3"
            )
            .bind(entity_name).bind(entity_id).bind(context_code)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.dff_data WHERE entity_name=$1 AND entity_id=$2"
            )
            .bind(entity_name).bind(entity_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(row_to_flexfield_data).collect())
    }

    async fn delete_flexfield_data(
        &self,
        entity_name: &str,
        entity_id: Uuid,
    ) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.dff_data WHERE entity_name=$1 AND entity_id=$2"
        )
        .bind(entity_name).bind(entity_id)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Dashboard ───────────────────────────────────────────────

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::FlexfieldDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active) as active
            FROM _atlas.dff_flexfields WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let active: i64 = row.try_get("active").unwrap_or(0);

        let ctx_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.dff_contexts c JOIN _atlas.dff_flexfields f ON c.flexfield_id = f.id WHERE f.organization_id = $1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let seg_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.dff_segments s JOIN _atlas.dff_flexfields f ON s.flexfield_id = f.id WHERE f.organization_id = $1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let vs_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.dff_value_sets WHERE organization_id = $1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let entity_rows = sqlx::query(
            r#"SELECT entity_name, COUNT(*) as cnt FROM _atlas.dff_flexfields
            WHERE organization_id = $1 GROUP BY entity_name ORDER BY entity_name"#
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_entity = serde_json::Map::new();
        for r in &entity_rows {
            let ename: String = r.get("entity_name");
            let cnt: i64 = r.get("cnt");
            by_entity.insert(ename, serde_json::json!(cnt));
        }

        Ok(atlas_shared::FlexfieldDashboardSummary {
            total_flexfields: total as i32,
            active_flexfields: active as i32,
            total_contexts: ctx_count as i32,
            total_segments: seg_count as i32,
            total_value_sets: vs_count as i32,
            flexfields_by_entity: serde_json::Value::Object(by_entity),
        })
    }
}
