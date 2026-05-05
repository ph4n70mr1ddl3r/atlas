//! Payment Risk & Fraud Detection E2E Tests
//!
//! Tests risk profile CRUD, fraud alert lifecycle (workflow),
//! sanctions screening, and supplier risk assessment workflow.

// NOTE: These tests use reqwest-based integration testing pattern
// and are not fully compatible with the tower-based E2E suite.
// Gated behind a feature to avoid compilation errors.
#[cfg(feature = "integration-test")]

use super::common::helpers::*;
use super::common::workflow_helpers::*;
use serde_json::json;

// ============================================================================
// Risk Profile Tests
// ============================================================================

#[cfg(test)]
mod risk_profile_tests {
    use super::*;

    pub async fn test_create_risk_profile(client: &reqwest::Client, base_url: &str, token: &str) {
        let body = json!({
            "code": "RP-TEST-001",
            "name": "Test Risk Profile",
            "description": "Test profile for e2e",
            "profile_type": "global",
            "default_risk_level": "medium",
            "duplicate_amount_tolerance_pct": "5.00",
            "duplicate_date_tolerance_days": "3.00",
            "velocity_daily_limit": "100000.00",
            "velocity_weekly_limit": "500000.00",
            "amount_anomaly_std_dev": "2.00",
            "enable_sanctions_screening": true,
            "enable_duplicate_detection": true,
            "enable_velocity_checks": true,
            "enable_amount_anomaly": true,
            "enable_behavioral_analysis": false,
            "auto_block_critical": true,
            "auto_block_high": false,
        });

        let resp = client
            .post(&format!("{}/api/v1/payment-risk/profiles", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 201, "Create risk profile should return 201");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(data["code"], "RP-TEST-001");
        assert_eq!(data["profile_type"], "global");
    }

    pub async fn test_list_risk_profiles(client: &reqwest::Client, base_url: &str, token: &str) {
        let resp = client
            .get(&format!("{}/api/v1/payment-risk/profiles", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200, "List risk profiles should return 200");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert!(data["data"].is_array());
    }

    pub async fn test_get_risk_profile(client: &reqwest::Client, base_url: &str, token: &str) {
        let resp = client
            .get(&format!("{}/api/v1/payment-risk/profiles/RP-TEST-001", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200, "Get risk profile should return 200");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(data["code"], "RP-TEST-001");
    }
}

// ============================================================================
// Fraud Alert Tests
// ============================================================================

#[cfg(test)]
mod fraud_alert_tests {
    use super::*;

    pub async fn test_create_fraud_alert(client: &reqwest::Client, base_url: &str, token: &str) {
        let body = json!({
            "alert_type": "duplicate_payment",
            "severity": "high",
            "supplier_name": "Test Supplier Inc.",
            "amount": "50000.00",
            "currency_code": "USD",
            "risk_score": "85.50",
            "description": "Duplicate payment detected for supplier",
            "evidence": "Amount and invoice number match existing payment",
        });

        let resp = client
            .post(&format!("{}/api/v1/payment-risk/alerts", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 201, "Create fraud alert should return 201");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(data["alert_type"], "duplicate_payment");
        assert_eq!(data["severity"], "high");
        assert_eq!(data["status"], "open");
    }

    pub async fn test_list_fraud_alerts(client: &reqwest::Client, base_url: &str, token: &str) {
        let resp = client
            .get(&format!("{}/api/v1/payment-risk/alerts?status=open", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200, "List fraud alerts should return 200");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert!(data["data"].is_array());
    }
}

// ============================================================================
// Sanctions Screening Tests
// ============================================================================

#[cfg(test)]
mod sanctions_screening_tests {
    use super::*;

    pub async fn test_create_screening_result(client: &reqwest::Client, base_url: &str, token: &str) {
        let body = json!({
            "screening_type": "supplier_onboarding",
            "supplier_name": "Test Supplier Inc.",
            "screened_list": "ofac_sdn",
            "match_type": "none",
            "match_status": "no_match",
            "action_taken": "none",
        });

        let resp = client
            .post(&format!("{}/api/v1/payment-risk/screening", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 201, "Create screening result should return 201");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(data["match_status"], "no_match");
    }
}

// ============================================================================
// Supplier Risk Assessment Tests
// ============================================================================

#[cfg(test)]
mod supplier_assessment_tests {
    use super::*;

    pub async fn test_create_assessment(client: &reqwest::Client, base_url: &str, token: &str, supplier_id: &str) {
        let body = json!({
            "supplier_id": supplier_id,
            "supplier_name": "Test Supplier Inc.",
            "assessment_type": "onboarding",
            "financial_risk_score": "25.00",
            "operational_risk_score": "15.00",
            "compliance_risk_score": "10.00",
            "payment_history_score": "20.00",
            "years_in_business": 10,
            "has_financial_statements": true,
            "has_audit_reports": true,
            "has_insurance": true,
            "is_sanctions_clear": true,
            "is_aml_clear": true,
            "is_pep_clear": true,
            "findings": "Low risk supplier with clean compliance record",
            "recommendations": "Approve for standard payment terms",
        });

        let resp = client
            .post(&format!("{}/api/v1/payment-risk/assessments", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 201, "Create assessment should return 201");
        let data: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(data["status"], "pending");
        assert_eq!(data["assessment_type"], "onboarding");
    }
}
