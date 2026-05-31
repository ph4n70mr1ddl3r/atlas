use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubtfulAccountPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_code: String,
    pub policy_name: String,
    pub calculation_method: String, // aging_based, percentage_based, specific_identification
    pub flat_percentage: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgingBucket {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub bucket_name: String,
    pub from_days: i32,
    pub to_days: Option<i32>,
    pub provision_percentage: f64,
}

pub struct DoubtfulAccountService {
    policies: Arc<RwLock<Vec<DoubtfulAccountPolicy>>>,
    buckets: Arc<RwLock<Vec<AgingBucket>>>,
}

impl DoubtfulAccountService {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(Vec::new())),
            buckets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_policy(
        &self,
        organization_id: Uuid,
        code: String,
        name: String,
        method: String,
        flat_pct: f64,
    ) -> Result<DoubtfulAccountPolicy, String> {
        let mut policies = self.policies.write().unwrap();
        if policies.iter().any(|p| p.organization_id == organization_id && p.policy_code == code) {
            return Err("Policy with this code already exists".to_string());
        }

        let policy = DoubtfulAccountPolicy {
            id: Uuid::new_v4(),
            organization_id,
            policy_code: code,
            policy_name: name,
            calculation_method: method,
            flat_percentage: flat_pct,
            status: "active".to_string(),
        };

        policies.push(policy.clone());
        Ok(policy)
    }

    pub fn add_bucket(
        &self,
        policy_id: Uuid,
        name: String,
        from: i32,
        to: Option<i32>,
        pct: f64,
    ) -> Result<AgingBucket, String> {
        let policies = self.policies.read().unwrap();
        if !policies.iter().any(|p| p.id == policy_id) {
            return Err("Policy not found".to_string());
        }

        let bucket = AgingBucket {
            id: Uuid::new_v4(),
            policy_id,
            bucket_name: name,
            from_days: from,
            to_days: to,
            provision_percentage: pct,
        };

        let mut buckets = self.buckets.write().unwrap();
        buckets.push(bucket.clone());
        Ok(bucket)
    }

    pub fn calculate_provision(&self, policy_id: Uuid, balance: f64, days_overdue: i32) -> f64 {
        let policies = self.policies.read().unwrap();
        let policy = match policies.iter().find(|p| p.id == policy_id) {
            Some(p) => p,
            None => return 0.0,
        };

        match policy.calculation_method.as_str() {
            "percentage_based" => balance * (policy.flat_percentage / 100.0),
            "aging_based" => {
                let buckets = self.buckets.read().unwrap();
                let applicable_bucket = buckets.iter()
                    .filter(|b| b.policy_id == policy_id)
                    .find(|b| days_overdue >= b.from_days && b.to_days.map_or(true, |to| days_overdue <= to));
                
                match applicable_bucket {
                    Some(b) => balance * (b.provision_percentage / 100.0),
                    None => 0.0,
                }
            },
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_based_provision() {
        let service = DoubtfulAccountService::new();
        let org_id = Uuid::new_v4();
        
        let p = service.create_policy(org_id, "FLAT_5".to_string(), "5% Flat".to_string(), "percentage_based".to_string(), 5.0).unwrap();
        
        let provision = service.calculate_provision(p.id, 1000.0, 0);
        assert_eq!(provision, 50.0);
    }

    #[test]
    fn test_aging_based_provision() {
        let service = DoubtfulAccountService::new();
        let org_id = Uuid::new_v4();
        
        let p = service.create_policy(org_id, "AGING".to_string(), "Aging".to_string(), "aging_based".to_string(), 0.0).unwrap();
        
        service.add_bucket(p.id, "Current".to_string(), 0, Some(30), 1.0).unwrap();
        service.add_bucket(p.id, "Overdue".to_string(), 31, None, 10.0).unwrap();

        // 15 days overdue -> 1%
        assert_eq!(service.calculate_provision(p.id, 1000.0, 15), 10.0);

        // 45 days overdue -> 10%
        assert_eq!(service.calculate_provision(p.id, 1000.0, 45), 100.0);
    }
}
