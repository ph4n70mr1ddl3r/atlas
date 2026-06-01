use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub status: String, // draft, submitted, selection_complete, formatted, confirmed, cancelled
    pub payment_method: String,
    pub total_payment_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedDocument {
    pub id: Uuid,
    pub ppr_id: Uuid,
    pub invoice_id: Uuid,
    pub amount_to_pay: f64,
    pub status: String, // selected, excluded, paid, error
}

pub struct PaymentProcessRequestService {
    requests: Arc<RwLock<Vec<PaymentProcessRequest>>>,
    documents: Arc<RwLock<Vec<SelectedDocument>>>,
}

impl Default for PaymentProcessRequestService {
    fn default() -> Self {
        Self {
            requests: Arc::new(RwLock::new(Vec::new())),
            documents: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl PaymentProcessRequestService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_request(
        &self,
        organization_id: Uuid,
        number: String,
        method: String,
    ) -> Result<PaymentProcessRequest, String> {
        let mut requests = self.requests.write().unwrap();
        if requests.iter().any(|r| r.organization_id == organization_id && r.request_number == number) {
            return Err("Request with this number already exists".to_string());
        }

        let ppr = PaymentProcessRequest {
            id: Uuid::new_v4(),
            organization_id,
            request_number: number,
            status: "draft".to_string(),
            payment_method: method,
            total_payment_amount: 0.0,
        };

        requests.push(ppr.clone());
        Ok(ppr)
    }

    pub fn add_document(
        &self,
        ppr_id: Uuid,
        invoice_id: Uuid,
        amount: f64,
    ) -> Result<SelectedDocument, String> {
        let mut requests = self.requests.write().unwrap();
        let ppr = requests.iter_mut().find(|r| r.id == ppr_id)
            .ok_or_else(|| "Request not found".to_string())?;

        if ppr.status != "draft" {
            return Err("Documents can only be added to draft requests".to_string());
        }

        let doc = SelectedDocument {
            id: Uuid::new_v4(),
            ppr_id,
            invoice_id,
            amount_to_pay: amount,
            status: "selected".to_string(),
        };

        ppr.total_payment_amount += amount;

        let mut documents = self.documents.write().unwrap();
        documents.push(doc.clone());
        Ok(doc)
    }

    pub fn confirm_payment(&self, id: Uuid) -> Result<(), String> {
        let mut requests = self.requests.write().unwrap();
        let ppr = requests.iter_mut().find(|r| r.id == id)
            .ok_or_else(|| "Request not found".to_string())?;
        
        ppr.status = "confirmed".to_string();

        let mut documents = self.documents.write().unwrap();
        for doc in documents.iter_mut().filter(|d| d.ppr_id == id) {
            doc.status = "paid".to_string();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_confirm_ppr() {
        let service = PaymentProcessRequestService::new();
        let org_id = Uuid::new_v4();
        
        let ppr = service.create_request(org_id, "PPR-2026-01".to_string(), "wire".to_string()).unwrap();
        assert_eq!(ppr.status, "draft");

        service.add_document(ppr.id, Uuid::new_v4(), 5000.0).unwrap();
        service.add_document(ppr.id, Uuid::new_v4(), 2500.0).unwrap();

        {
            let requests = service.requests.read().unwrap();
            let updated = requests.iter().find(|r| r.id == ppr.id).unwrap();
            assert_eq!(updated.total_payment_amount, 7500.0);
        }

        service.confirm_payment(ppr.id).unwrap();

        let requests = service.requests.read().unwrap();
        let confirmed = requests.iter().find(|r| r.id == ppr.id).unwrap();
        assert_eq!(confirmed.status, "confirmed");

        let docs = service.documents.read().unwrap();
        assert!(docs.iter().all(|d| d.status == "paid"));
    }

    #[test]
    fn test_duplicate_ppr_number() {
        let service = PaymentProcessRequestService::new();
        let org_id = Uuid::new_v4();
        service.create_request(org_id, "REQ-1".to_string(), "ach".to_string()).unwrap();
        let res = service.create_request(org_id, "REQ-1".to_string(), "ach".to_string());
        assert!(res.is_err());
    }
}
