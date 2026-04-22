//! Purchase Requisition Repository
//!
//! PostgreSQL storage for purchase requisitions, lines, distributions,
//! approvals, and AutoCreate links.

use atlas_shared::{
    PurchaseRequisition, RequisitionLine, RequisitionDistribution,
    RequisitionApproval, AutocreateLink, RequisitionDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for purchase requisition storage
#[async_trait]
pub trait PurchaseRequisitionRepository: Send + Sync {
    // ── Requisitions ─────────────────────────────────────────────
    async fn create_requisition(
        &self,
        org_id: Uuid, requisition_number: &str, description: Option<&str>,
        urgency_code: &str, requester_id: Option<Uuid>, requester_name: Option<&str>,
        department: Option<&str>, justification: Option<&str>,
        budget_code: Option<&str>, amount_limit: Option<&str>,
        total_amount: &str, currency_code: &str,
        charge_account_code: Option<&str>, delivery_address: Option<&str>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<PurchaseRequisition>;
    async fn get_requisition_by_id(&self, id: Uuid) -> AtlasResult<Option<PurchaseRequisition>>;
    async fn list_requisitions(&self, org_id: Uuid, status: Option<&str>, requester_id: Option<Uuid>) -> AtlasResult<Vec<PurchaseRequisition>>;
    async fn update_requisition(
        &self,
        id: Uuid, description: Option<&str>, urgency_code: &str,
        department: Option<&str>, justification: Option<&str>,
        budget_code: Option<&str>, total_amount: &str,
        charge_account_code: Option<&str>, delivery_address: Option<&str>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, updated_by: Option<Uuid>,
    ) -> AtlasResult<PurchaseRequisition>;
    async fn update_requisition_status(
        &self,
        id: Uuid, status: &str, approved_by: Option<Uuid>, approver_name: Option<&str>,
    ) -> AtlasResult<PurchaseRequisition>;
    async fn update_requisition_total(&self, id: Uuid, total_amount: &str) -> AtlasResult<()>;
    async fn delete_requisition(&self, id: Uuid) -> AtlasResult<()>;

    // ── Lines ────────────────────────────────────────────────────
    async fn create_line(
        &self,
        org_id: Uuid, requisition_id: Uuid, line_number: i32,
        item_code: Option<&str>, item_description: &str,
        category: Option<&str>, quantity: &str, unit_of_measure: &str,
        unit_price: &str, line_amount: &str, currency_code: &str,
        charge_account_code: Option<&str>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        source_type: &str, source_reference: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RequisitionLine>;
    async fn get_line_by_id(&self, line_id: Uuid) -> AtlasResult<Option<RequisitionLine>>;
    async fn update_line_status(&self, line_id: Uuid, status: &str) -> AtlasResult<()>;
    async fn update_line_statuses(&self, requisition_id: Uuid, status: &str) -> AtlasResult<()>;
    async fn delete_line(&self, line_id: Uuid) -> AtlasResult<()>;

    // ── Distributions ───────────────────────────────────────────
    async fn create_distribution(
        &self,
        org_id: Uuid, requisition_id: Uuid, line_id: Uuid,
        distribution_number: i32, charge_account_code: &str,
        allocation_percentage: &str, amount: &str,
        project_code: Option<&str>, cost_center: Option<&str>,
    ) -> AtlasResult<RequisitionDistribution>;
    async fn list_distributions_by_line(&self, line_id: Uuid) -> AtlasResult<Vec<RequisitionDistribution>>;

    // ── Approvals ────────────────────────────────────────────────
    async fn create_approval(
        &self,
        org_id: Uuid, requisition_id: Uuid, approver_id: Uuid,
        approver_name: Option<&str>, action: &str, comments: Option<&str>,
    ) -> AtlasResult<RequisitionApproval>;
    async fn list_approvals(&self, requisition_id: Uuid) -> AtlasResult<Vec<RequisitionApproval>>;

    // ── AutoCreate Links ─────────────────────────────────────────
    async fn create_autocreate_link(
        &self,
        org_id: Uuid, requisition_id: Uuid, requisition_line_id: Uuid,
        purchase_order_number: &str,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        quantity_ordered: &str, status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutocreateLink>;
    async fn list_autocreate_links(&self, requisition_id: Uuid) -> AtlasResult<Vec<AutocreateLink>>;
    async fn update_autocreate_link_status(&self, link_id: Uuid, status: &str) -> AtlasResult<()>;

    // ── Dashboard ────────────────────────────────────────────────
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<RequisitionDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresPurchaseRequisitionRepository {
    pool: PgPool,
}

impl PostgresPurchaseRequisitionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_requisition(&self, row: &sqlx::postgres::PgRow) -> PurchaseRequisition {
        PurchaseRequisition {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            requisition_number: row.get("requisition_number"),
            description: row.get("description"),
            urgency_code: row.get("urgency_code"),
            status: row.get("status"),
            requester_id: row.get("requester_id"),
            requester_name: row.get("requester_name"),
            department: row.get("department"),
            justification: row.get("justification"),
            budget_code: row.get("budget_code"),
            amount_limit: row.get("amount_limit"),
            total_amount: row.get("total_amount"),
            currency_code: row.get("currency_code"),
            charge_account_code: row.get("charge_account_code"),
            delivery_address: row.get("delivery_address"),
            requested_delivery_date: row.get("requested_delivery_date"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            submitted_at: row.get("submitted_at"),
            closed_at: row.get("closed_at"),
            notes: row.get("notes"),
            lines: vec![],
            approvals: vec![],
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> RequisitionLine {
        RequisitionLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            requisition_id: row.get("requisition_id"),
            line_number: row.get("line_number"),
            item_code: row.get("item_code"),
            item_description: row.get("item_description"),
            category: row.get("category"),
            quantity: row.get("quantity"),
            unit_of_measure: row.get("unit_of_measure"),
            unit_price: row.get("unit_price"),
            line_amount: row.get("line_amount"),
            currency_code: row.get("currency_code"),
            charge_account_code: row.get("charge_account_code"),
            requested_delivery_date: row.get("requested_delivery_date"),
            supplier_id: row.get("supplier_id"),
            supplier_name: row.get("supplier_name"),
            status: row.get("status"),
            source_type: row.get("source_type"),
            source_reference: row.get("source_reference"),
            notes: row.get("notes"),
            distributions: vec![],
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_distribution(&self, row: &sqlx::postgres::PgRow) -> RequisitionDistribution {
        RequisitionDistribution {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            requisition_id: row.get("requisition_id"),
            line_id: row.get("line_id"),
            distribution_number: row.get("distribution_number"),
            charge_account_code: row.get("charge_account_code"),
            allocation_percentage: row.get("allocation_percentage"),
            amount: row.get("amount"),
            project_code: row.get("project_code"),
            cost_center: row.get("cost_center"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_approval(&self, row: &sqlx::postgres::PgRow) -> RequisitionApproval {
        RequisitionApproval {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            requisition_id: row.get("requisition_id"),
            approver_id: row.get("approver_id"),
            approver_name: row.get("approver_name"),
            action: row.get("action"),
            comments: row.get("comments"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_autocreate_link(&self, row: &sqlx::postgres::PgRow) -> AutocreateLink {
        AutocreateLink {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            requisition_id: row.get("requisition_id"),
            requisition_line_id: row.get("requisition_line_id"),
            purchase_order_id: row.get("purchase_order_id"),
            purchase_order_number: row.get("purchase_order_number"),
            purchase_order_line_id: row.get("purchase_order_line_id"),
            purchase_order_line_number: row.get("purchase_order_line_number"),
            quantity_ordered: row.get("quantity_ordered"),
            status: row.get("status"),
            autocreate_date: row.get("autocreate_date"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        }
    }

    async fn load_lines(&self, requisition_id: Uuid) -> AtlasResult<Vec<RequisitionLine>> {
        let line_rows = sqlx::query(
            "SELECT * FROM _atlas.requisition_lines WHERE requisition_id = $1 ORDER BY line_number"
        ).bind(requisition_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut lines = Vec::new();
        for line_row in &line_rows {
            let mut line = self.row_to_line(line_row);
            let line_id: Uuid = line_row.get("id");

            let dist_rows = sqlx::query(
                "SELECT * FROM _atlas.requisition_distributions WHERE line_id = $1 ORDER BY distribution_number"
            ).bind(line_id).fetch_all(&self.pool).await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

            line.distributions = dist_rows.iter().map(|d| self.row_to_distribution(d)).collect();
            lines.push(line);
        }
        Ok(lines)
    }

    async fn load_approvals(&self, requisition_id: Uuid) -> AtlasResult<Vec<RequisitionApproval>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.requisition_approvals WHERE requisition_id = $1 ORDER BY created_at"
        ).bind(requisition_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_approval(r)).collect())
    }
}

#[async_trait]
impl PurchaseRequisitionRepository for PostgresPurchaseRequisitionRepository {
    // ── Requisitions ─────────────────────────────────────────────

    async fn create_requisition(
        &self,
        org_id: Uuid, requisition_number: &str, description: Option<&str>,
        urgency_code: &str, requester_id: Option<Uuid>, requester_name: Option<&str>,
        department: Option<&str>, justification: Option<&str>,
        budget_code: Option<&str>, amount_limit: Option<&str>,
        total_amount: &str, currency_code: &str,
        charge_account_code: Option<&str>, delivery_address: Option<&str>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<PurchaseRequisition> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.purchase_requisitions
                (organization_id, requisition_number, description, urgency_code,
                 status, requester_id, requester_name, department, justification,
                 budget_code, amount_limit, total_amount, currency_code,
                 charge_account_code, delivery_address, requested_delivery_date,
                 notes, created_by)
            VALUES ($1,$2,$3,$4,'draft',$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            RETURNING *"#,
        )
        .bind(org_id).bind(requisition_number).bind(description).bind(urgency_code)
        .bind(requester_id).bind(requester_name).bind(department).bind(justification)
        .bind(budget_code).bind(amount_limit).bind(total_amount).bind(currency_code)
        .bind(charge_account_code).bind(delivery_address).bind(requested_delivery_date)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let id: Uuid = row.get("id");
        let mut req = self.row_to_requisition(&row);
        req.lines = self.load_lines(id).await?;
        req.approvals = self.load_approvals(id).await?;
        Ok(req)
    }

    async fn get_requisition_by_id(&self, id: Uuid) -> AtlasResult<Option<PurchaseRequisition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.purchase_requisitions WHERE id = $1"
        ).bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let mut req = self.row_to_requisition(&r);
                req.lines = self.load_lines(id).await?;
                req.approvals = self.load_approvals(id).await?;
                Ok(Some(req))
            }
            None => Ok(None),
        }
    }

    async fn list_requisitions(&self, org_id: Uuid, status: Option<&str>, requester_id: Option<Uuid>) -> AtlasResult<Vec<PurchaseRequisition>> {
        let rows = if status.is_some() && requester_id.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.purchase_requisitions WHERE organization_id = $1 AND status = $2 AND requester_id = $3 ORDER BY created_at DESC"
            ).bind(org_id).bind(status).bind(requester_id)
            .fetch_all(&self.pool).await
        } else if status.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.purchase_requisitions WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(status)
            .fetch_all(&self.pool).await
        } else if requester_id.is_some() {
            sqlx::query(
                "SELECT * FROM _atlas.purchase_requisitions WHERE organization_id = $1 AND requester_id = $2 ORDER BY created_at DESC"
            ).bind(org_id).bind(requester_id)
            .fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.purchase_requisitions WHERE organization_id = $1 ORDER BY created_at DESC"
            ).bind(org_id)
            .fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut reqs = Vec::new();
        for row in &rows {
            let id: Uuid = row.get("id");
            let mut req = self.row_to_requisition(row);
            req.lines = self.load_lines(id).await?;
            req.approvals = self.load_approvals(id).await?;
            reqs.push(req);
        }
        Ok(reqs)
    }

    async fn update_requisition(
        &self,
        id: Uuid, description: Option<&str>, urgency_code: &str,
        department: Option<&str>, justification: Option<&str>,
        budget_code: Option<&str>, total_amount: &str,
        charge_account_code: Option<&str>, delivery_address: Option<&str>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, updated_by: Option<Uuid>,
    ) -> AtlasResult<PurchaseRequisition> {
        let row = sqlx::query(
            r#"UPDATE _atlas.purchase_requisitions SET
                description = COALESCE($2, description),
                urgency_code = $3,
                department = COALESCE($4, department),
                justification = COALESCE($5, justification),
                budget_code = COALESCE($6, budget_code),
                total_amount = $7,
                charge_account_code = COALESCE($8, charge_account_code),
                delivery_address = COALESCE($9, delivery_address),
                requested_delivery_date = COALESCE($10, requested_delivery_date),
                notes = COALESCE($11, notes),
                updated_by = $12,
                updated_at = now()
            WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(description).bind(urgency_code).bind(department)
        .bind(justification).bind(budget_code).bind(total_amount)
        .bind(charge_account_code).bind(delivery_address).bind(requested_delivery_date)
        .bind(notes).bind(updated_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut req = self.row_to_requisition(&row);
        req.lines = self.load_lines(id).await?;
        req.approvals = self.load_approvals(id).await?;
        Ok(req)
    }

    async fn update_requisition_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>, _approver_name: Option<&str>,
    ) -> AtlasResult<PurchaseRequisition> {
        let row = if status == "approved" {
            sqlx::query(
                r#"UPDATE _atlas.purchase_requisitions SET status = $2, approved_by = $3, approved_at = now(), updated_at = now()
                WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status).bind(approved_by)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else if status == "submitted" {
            sqlx::query(
                r#"UPDATE _atlas.purchase_requisitions SET status = $2, submitted_at = now(), updated_at = now()
                WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else if status == "closed" {
            sqlx::query(
                r#"UPDATE _atlas.purchase_requisitions SET status = $2, closed_at = now(), updated_at = now()
                WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r#"UPDATE _atlas.purchase_requisitions SET status = $2, updated_at = now()
                WHERE id = $1 RETURNING *"#,
            ).bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };

        let mut req = self.row_to_requisition(&row);
        req.lines = self.load_lines(id).await?;
        req.approvals = self.load_approvals(id).await?;
        Ok(req)
    }

    async fn update_requisition_total(&self, id: Uuid, total_amount: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.purchase_requisitions SET total_amount = $2, updated_at = now() WHERE id = $1"
        ).bind(id).bind(total_amount)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_requisition(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.purchase_requisitions WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Lines ────────────────────────────────────────────────────

    async fn create_line(
        &self,
        org_id: Uuid, requisition_id: Uuid, line_number: i32,
        item_code: Option<&str>, item_description: &str,
        category: Option<&str>, quantity: &str, unit_of_measure: &str,
        unit_price: &str, line_amount: &str, currency_code: &str,
        charge_account_code: Option<&str>,
        requested_delivery_date: Option<chrono::NaiveDate>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        source_type: &str, source_reference: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<RequisitionLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.requisition_lines
                (organization_id, requisition_id, line_number,
                 item_code, item_description, category, quantity, unit_of_measure,
                 unit_price, line_amount, currency_code, charge_account_code,
                 requested_delivery_date, supplier_id, supplier_name,
                 status, source_type, source_reference, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,'draft',$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(org_id).bind(requisition_id).bind(line_number)
        .bind(item_code).bind(item_description).bind(category)
        .bind(quantity).bind(unit_of_measure).bind(unit_price).bind(line_amount)
        .bind(currency_code).bind(charge_account_code).bind(requested_delivery_date)
        .bind(supplier_id).bind(supplier_name).bind(source_type).bind(source_reference)
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut line = self.row_to_line(&row);
        line.distributions = vec![];
        Ok(line)
    }

    async fn get_line_by_id(&self, line_id: Uuid) -> AtlasResult<Option<RequisitionLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.requisition_lines WHERE id = $1"
        ).bind(line_id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let mut line = self.row_to_line(&r);
                line.distributions = self.load_line_distributions(line_id).await?;
                Ok(Some(line))
            }
            None => Ok(None),
        }
    }

    async fn update_line_status(&self, line_id: Uuid, status: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.requisition_lines SET status = $2, updated_at = now() WHERE id = $1"
        ).bind(line_id).bind(status)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_line_statuses(&self, requisition_id: Uuid, status: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.requisition_lines SET status = $2, updated_at = now() WHERE requisition_id = $1"
        ).bind(requisition_id).bind(status)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_line(&self, line_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.requisition_distributions WHERE line_id = $1")
            .bind(line_id).execute(&self.pool).await.ok();
        sqlx::query("DELETE FROM _atlas.requisition_lines WHERE id = $1")
            .bind(line_id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Distributions ────────────────────────────────────────────

    async fn create_distribution(
        &self,
        org_id: Uuid, requisition_id: Uuid, line_id: Uuid,
        distribution_number: i32, charge_account_code: &str,
        allocation_percentage: &str, amount: &str,
        project_code: Option<&str>, cost_center: Option<&str>,
    ) -> AtlasResult<RequisitionDistribution> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.requisition_distributions
                (organization_id, requisition_id, line_id, distribution_number,
                 charge_account_code, allocation_percentage, amount,
                 project_code, cost_center)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *"#,
        )
        .bind(org_id).bind(requisition_id).bind(line_id).bind(distribution_number)
        .bind(charge_account_code).bind(allocation_percentage).bind(amount)
        .bind(project_code).bind(cost_center)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_distribution(&row))
    }

    async fn list_distributions_by_line(&self, line_id: Uuid) -> AtlasResult<Vec<RequisitionDistribution>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.requisition_distributions WHERE line_id = $1 ORDER BY distribution_number"
        ).bind(line_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_distribution(r)).collect())
    }

    // ── Approvals ────────────────────────────────────────────────

    async fn create_approval(
        &self,
        org_id: Uuid, requisition_id: Uuid, approver_id: Uuid,
        approver_name: Option<&str>, action: &str, comments: Option<&str>,
    ) -> AtlasResult<RequisitionApproval> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.requisition_approvals
                (organization_id, requisition_id, approver_id, approver_name, action, comments)
            VALUES ($1,$2,$3,$4,$5,$6)
            RETURNING *"#,
        )
        .bind(org_id).bind(requisition_id).bind(approver_id).bind(approver_name)
        .bind(action).bind(comments)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_approval(&row))
    }

    async fn list_approvals(&self, requisition_id: Uuid) -> AtlasResult<Vec<RequisitionApproval>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.requisition_approvals WHERE requisition_id = $1 ORDER BY created_at"
        ).bind(requisition_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_approval(r)).collect())
    }

    // ── AutoCreate Links ─────────────────────────────────────────

    async fn create_autocreate_link(
        &self,
        org_id: Uuid, requisition_id: Uuid, requisition_line_id: Uuid,
        purchase_order_number: &str,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        quantity_ordered: &str, status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutocreateLink> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.autocreate_links
                (organization_id, requisition_id, requisition_line_id,
                 purchase_order_number, supplier_id, supplier_name,
                 quantity_ordered, status, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *"#,
        )
        .bind(org_id).bind(requisition_id).bind(requisition_line_id)
        .bind(purchase_order_number).bind(supplier_id).bind(supplier_name)
        .bind(quantity_ordered).bind(status).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_autocreate_link(&row))
    }

    async fn list_autocreate_links(&self, requisition_id: Uuid) -> AtlasResult<Vec<AutocreateLink>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.autocreate_links WHERE requisition_id = $1 ORDER BY created_at"
        ).bind(requisition_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_autocreate_link(r)).collect())
    }

    async fn update_autocreate_link_status(&self, link_id: Uuid, status: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.autocreate_links SET status = $2 WHERE id = $1"
        ).bind(link_id).bind(status)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ── Dashboard ─────────────────────────────────────────────────

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<RequisitionDashboardSummary> {
        let req_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft,
                COUNT(*) FILTER (WHERE status = 'submitted') as submitted,
                COUNT(*) FILTER (WHERE status = 'approved') as approved,
                COUNT(*) FILTER (WHERE status = 'rejected') as rejected,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled,
                COALESCE(SUM(total_amount::numeric), 0) as total_amount
            FROM _atlas.purchase_requisitions WHERE organization_id = $1"#
        ).bind(org_id).fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let auto_row = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'ordered') as ordered
            FROM _atlas.autocreate_links WHERE organization_id = $1"#
        ).bind(org_id).fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let prio_rows = sqlx::query(
            "SELECT urgency_code, COUNT(*) as cnt FROM _atlas.purchase_requisitions WHERE organization_id = $1 GROUP BY urgency_code"
        ).bind(org_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_priority = serde_json::Map::new();
        for r in &prio_rows {
            let code: String = r.get("urgency_code");
            let cnt: i64 = r.get("cnt");
            by_priority.insert(code, serde_json::json!(cnt));
        }

        let total_reqs: i64 = req_row.try_get("total").unwrap_or(0);
        let draft_reqs: i64 = req_row.try_get("draft").unwrap_or(0);
        let submitted_reqs: i64 = req_row.try_get("submitted").unwrap_or(0);
        let approved_reqs: i64 = req_row.try_get("approved").unwrap_or(0);
        let rejected_reqs: i64 = req_row.try_get("rejected").unwrap_or(0);
        let cancelled_reqs: i64 = req_row.try_get("cancelled").unwrap_or(0);
        let total_amount: serde_json::Value = req_row.try_get("total_amount").unwrap_or(serde_json::json!("0"));
        let auto_pending: i64 = auto_row.try_get("pending").unwrap_or(0);
        let auto_ordered: i64 = auto_row.try_get("ordered").unwrap_or(0);

        Ok(RequisitionDashboardSummary {
            total_requisitions: total_reqs as i32,
            draft_requisitions: draft_reqs as i32,
            submitted_requisitions: submitted_reqs as i32,
            approved_requisitions: approved_reqs as i32,
            rejected_requisitions: rejected_reqs as i32,
            cancelled_requisitions: cancelled_reqs as i32,
            total_amount: total_amount.to_string(),
            autocreate_pending: auto_pending as i32,
            autocreate_ordered: auto_ordered as i32,
            by_priority: serde_json::Value::Object(by_priority),
        })
    }
}

impl PostgresPurchaseRequisitionRepository {
    async fn load_line_distributions(&self, line_id: Uuid) -> AtlasResult<Vec<RequisitionDistribution>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.requisition_distributions WHERE line_id = $1 ORDER BY distribution_number"
        ).bind(line_id).fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_distribution(r)).collect())
    }
}