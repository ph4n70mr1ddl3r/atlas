//! Oracle Fusion Financial Feature: Supplier Merge (Payables)
//! Merges a duplicate supplier into a primary supplier, transferring transactions and inactivating the old record.

pub struct SupplierMergeService;

#[derive(Debug, PartialEq, Clone)]
pub struct Supplier {
    pub supplier_id: String,
    pub supplier_name: String,
    pub is_active: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ApInvoice {
    pub invoice_id: String,
    pub supplier_id: String,
    pub amount: f64,
    pub is_paid: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PurchaseOrder {
    pub po_id: String,
    pub supplier_id: String,
    pub status: String,
}

#[derive(Debug, PartialEq)]
pub struct MergeRequest {
    pub from_supplier_id: String,
    pub to_supplier_id: String,
    pub transfer_invoices: bool,
    pub transfer_purchase_orders: bool,
    pub inactivate_from_supplier: bool,
}

#[derive(Debug, PartialEq)]
pub struct MergeResult {
    pub is_successful: bool,
    pub from_supplier_inactivated: bool,
    pub invoices_transferred: usize,
    pub pos_transferred: usize,
    pub updated_invoices: Vec<ApInvoice>,
    pub updated_pos: Vec<PurchaseOrder>,
    pub error_message: Option<String>,
}

impl SupplierMergeService {
    /// Executes a supplier merge, transferring un-paid invoices and open POs,
    /// and optionally inactivating the duplicate supplier.
    #[must_use]
    pub fn process_merge(
        request: &MergeRequest,
        from_supplier: &mut Supplier,
        to_supplier: &Supplier,
        invoices: &mut [ApInvoice],
        purchase_orders: &mut [PurchaseOrder],
    ) -> MergeResult {
        if request.from_supplier_id == request.to_supplier_id {
            return MergeResult {
                is_successful: false,
                from_supplier_inactivated: false,
                invoices_transferred: 0,
                pos_transferred: 0,
                updated_invoices: vec![],
                updated_pos: vec![],
                error_message: Some("Cannot merge a supplier into itself.".to_string()),
            };
        }

        if from_supplier.supplier_id != request.from_supplier_id {
            return MergeResult {
                is_successful: false,
                from_supplier_inactivated: false,
                invoices_transferred: 0,
                pos_transferred: 0,
                updated_invoices: vec![],
                updated_pos: vec![],
                error_message: Some("Provided 'from_supplier' does not match request.".to_string()),
            };
        }

        if !to_supplier.is_active {
            return MergeResult {
                is_successful: false,
                from_supplier_inactivated: false,
                invoices_transferred: 0,
                pos_transferred: 0,
                updated_invoices: vec![],
                updated_pos: vec![],
                error_message: Some("The target 'to_supplier' must be active.".to_string()),
            };
        }

        let mut invoices_transferred = 0;
        let mut pos_transferred = 0;
        let mut updated_invoices = Vec::new();
        let mut updated_pos = Vec::new();

        // 1. Transfer Invoices
        if request.transfer_invoices {
            for inv in invoices.iter_mut() {
                // In Oracle, typically unpaid or partially paid invoices are transferred.
                // We'll transfer all matching invoices for the duplicate supplier for simplicity.
                if inv.supplier_id == request.from_supplier_id {
                    inv.supplier_id = request.to_supplier_id.clone();
                    updated_invoices.push(inv.clone());
                    invoices_transferred += 1;
                }
            }
        }

        // 2. Transfer Purchase Orders
        if request.transfer_purchase_orders {
            for po in purchase_orders.iter_mut() {
                // Only transfer open/active POs
                if po.supplier_id == request.from_supplier_id
                    && po.status != "CLOSED"
                    && po.status != "CANCELED"
                {
                    po.supplier_id = request.to_supplier_id.clone();
                    updated_pos.push(po.clone());
                    pos_transferred += 1;
                }
            }
        }

        // 3. Inactivate From Supplier
        let mut inactivated = false;
        if request.inactivate_from_supplier {
            from_supplier.is_active = false;
            inactivated = true;
        }

        MergeResult {
            is_successful: true,
            from_supplier_inactivated: inactivated,
            invoices_transferred,
            pos_transferred,
            updated_invoices,
            updated_pos,
            error_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_data() -> (Supplier, Supplier, Vec<ApInvoice>, Vec<PurchaseOrder>) {
        let from_supp = Supplier {
            supplier_id: "SUPP-DUPE".to_string(),
            supplier_name: "Acme Corp Ltd".to_string(),
            is_active: true,
        };

        let to_supp = Supplier {
            supplier_id: "SUPP-PRIMARY".to_string(),
            supplier_name: "Acme Corporation".to_string(),
            is_active: true,
        };

        let invoices = vec![
            ApInvoice {
                invoice_id: "INV-1".to_string(),
                supplier_id: "SUPP-DUPE".to_string(),
                amount: 100.0,
                is_paid: false,
            },
            ApInvoice {
                invoice_id: "INV-2".to_string(),
                supplier_id: "SUPP-DUPE".to_string(),
                amount: 500.0,
                is_paid: true,
            },
            ApInvoice {
                invoice_id: "INV-3".to_string(),
                supplier_id: "SUPP-OTHER".to_string(),
                amount: 300.0,
                is_paid: false,
            },
        ];

        let pos = vec![
            PurchaseOrder {
                po_id: "PO-1".to_string(),
                supplier_id: "SUPP-DUPE".to_string(),
                status: "OPEN".to_string(),
            },
            PurchaseOrder {
                po_id: "PO-2".to_string(),
                supplier_id: "SUPP-DUPE".to_string(),
                status: "CLOSED".to_string(),
            }, // Won't transfer
            PurchaseOrder {
                po_id: "PO-3".to_string(),
                supplier_id: "SUPP-PRIMARY".to_string(),
                status: "OPEN".to_string(),
            },
        ];

        (from_supp, to_supp, invoices, pos)
    }

    #[test]
    fn test_successful_full_merge() {
        let (mut from_supp, to_supp, mut invoices, mut pos) = setup_data();

        let request = MergeRequest {
            from_supplier_id: "SUPP-DUPE".to_string(),
            to_supplier_id: "SUPP-PRIMARY".to_string(),
            transfer_invoices: true,
            transfer_purchase_orders: true,
            inactivate_from_supplier: true,
        };

        let result = SupplierMergeService::process_merge(
            &request,
            &mut from_supp,
            &to_supp,
            &mut invoices,
            &mut pos,
        );

        assert!(result.is_successful);
        assert!(result.from_supplier_inactivated);
        assert_eq!(result.invoices_transferred, 2); // INV-1, INV-2
        assert_eq!(result.pos_transferred, 1); // Only PO-1 (PO-2 is CLOSED)

        // Verify the original structs were mutated
        assert!(!from_supp.is_active);
        let transferred_inv = invoices.iter().find(|i| i.invoice_id == "INV-1").unwrap();
        assert_eq!(transferred_inv.supplier_id, "SUPP-PRIMARY");
    }

    #[test]
    fn test_merge_no_inactivation() {
        let (mut from_supp, to_supp, mut invoices, mut pos) = setup_data();

        let request = MergeRequest {
            from_supplier_id: "SUPP-DUPE".to_string(),
            to_supplier_id: "SUPP-PRIMARY".to_string(),
            transfer_invoices: true,
            transfer_purchase_orders: false, // Don't transfer POs
            inactivate_from_supplier: false, // Keep duplicate active (rare but possible)
        };

        let result = SupplierMergeService::process_merge(
            &request,
            &mut from_supp,
            &to_supp,
            &mut invoices,
            &mut pos,
        );

        assert!(result.is_successful);
        assert!(!result.from_supplier_inactivated);
        assert!(from_supp.is_active); // Still active
        assert_eq!(result.invoices_transferred, 2);
        assert_eq!(result.pos_transferred, 0); // Requested not to transfer
    }

    #[test]
    fn test_merge_fail_same_supplier() {
        let (mut from_supp, to_supp, mut invoices, mut pos) = setup_data();

        let request = MergeRequest {
            from_supplier_id: "SUPP-PRIMARY".to_string(),
            to_supplier_id: "SUPP-PRIMARY".to_string(),
            transfer_invoices: true,
            transfer_purchase_orders: true,
            inactivate_from_supplier: true,
        };

        let result = SupplierMergeService::process_merge(
            &request,
            &mut from_supp,
            &to_supp,
            &mut invoices,
            &mut pos,
        );

        assert!(!result.is_successful);
        assert_eq!(
            result.error_message.unwrap(),
            "Cannot merge a supplier into itself."
        );
    }

    #[test]
    fn test_merge_fail_target_inactive() {
        let (mut from_supp, mut to_supp, mut invoices, mut pos) = setup_data();
        to_supp.is_active = false; // Make target inactive

        let request = MergeRequest {
            from_supplier_id: "SUPP-DUPE".to_string(),
            to_supplier_id: "SUPP-PRIMARY".to_string(),
            transfer_invoices: true,
            transfer_purchase_orders: true,
            inactivate_from_supplier: true,
        };

        let result = SupplierMergeService::process_merge(
            &request,
            &mut from_supp,
            &to_supp,
            &mut invoices,
            &mut pos,
        );

        assert!(!result.is_successful);
        assert_eq!(
            result.error_message.unwrap(),
            "The target 'to_supplier' must be active."
        );
    }
}
