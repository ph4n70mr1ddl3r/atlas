pub struct AssetRetirementService;

pub struct AssetRetirementResult {
    pub retirement_id: String,
    pub asset_id: String,
    pub net_book_value: f64,
    pub proceeds_of_sale: f64,
    pub cost_of_removal: f64,
    pub gain_or_loss: f64,
    pub status: String,
}

impl AssetRetirementService {
    /// Processes the retirement of a fixed asset.
    /// This is an Oracle Fusion Fixed Assets feature that records the removal
    /// of an asset and calculates the resulting gain or loss based on its
    /// Net Book Value (NBV), proceeds of sale, and cost of removal.
    #[must_use]
    pub fn process_retirement(
        asset_id: &str,
        net_book_value: f64,
        proceeds_of_sale: f64,
        cost_of_removal: f64,
    ) -> AssetRetirementResult {
        if asset_id.is_empty() {
            return AssetRetirementResult {
                retirement_id: "".to_string(),
                asset_id: asset_id.to_string(),
                net_book_value: 0.0,
                proceeds_of_sale: 0.0,
                cost_of_removal: 0.0,
                gain_or_loss: 0.0,
                status: "REJECTED_INVALID_ASSET".to_string(),
            };
        }

        if net_book_value < 0.0 {
            return AssetRetirementResult {
                retirement_id: "".to_string(),
                asset_id: asset_id.to_string(),
                net_book_value,
                proceeds_of_sale: 0.0,
                cost_of_removal: 0.0,
                gain_or_loss: 0.0,
                status: "REJECTED_NEGATIVE_NBV".to_string(),
            };
        }

        // Calculate Gain or Loss
        // Gain/Loss = Proceeds of Sale - Cost of Removal - Net Book Value
        let gain_or_loss = proceeds_of_sale - cost_of_removal - net_book_value;

        AssetRetirementResult {
            retirement_id: format!("RET-{}", asset_id),
            asset_id: asset_id.to_string(),
            net_book_value,
            proceeds_of_sale,
            cost_of_removal,
            gain_or_loss,
            status: "RETIRED".to_string(),
        }
    }

    /// Reinstates a previously retired asset.
    #[must_use]
    pub fn reinstate_asset(retirement_id: &str) -> String {
        if retirement_id.is_empty() {
            "INVALID_RETIREMENT_ID".to_string()
        } else {
            "REINSTATED".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_retirement_gain() {
        // NBV = 1000, Sold for 1500, Cost of removal 100
        // Gain = 1500 - 100 - 1000 = 400
        let result = AssetRetirementService::process_retirement("ASSET-101", 1000.0, 1500.0, 100.0);
        assert_eq!(result.asset_id, "ASSET-101");
        assert_eq!(result.gain_or_loss, 400.0);
        assert_eq!(result.retirement_id, "RET-ASSET-101");
        assert_eq!(result.status, "RETIRED");
    }

    #[test]
    fn test_process_retirement_loss() {
        // NBV = 2000, Sold for 500, Cost of removal 50
        // Loss = 500 - 50 - 2000 = -1550
        let result = AssetRetirementService::process_retirement("ASSET-102", 2000.0, 500.0, 50.0);
        assert_eq!(result.asset_id, "ASSET-102");
        assert_eq!(result.gain_or_loss, -1550.0);
        assert_eq!(result.status, "RETIRED");
    }

    #[test]
    fn test_process_retirement_scrapped() {
        // Scrapped: Proceeds = 0
        // NBV = 500, Cost of removal = 0
        // Loss = 0 - 0 - 500 = -500
        let result = AssetRetirementService::process_retirement("ASSET-103", 500.0, 0.0, 0.0);
        assert_eq!(result.gain_or_loss, -500.0);
        assert_eq!(result.status, "RETIRED");
    }

    #[test]
    fn test_process_retirement_invalid_asset() {
        let result = AssetRetirementService::process_retirement("", 1000.0, 1500.0, 100.0);
        assert_eq!(result.retirement_id, "");
        assert_eq!(result.status, "REJECTED_INVALID_ASSET");
    }

    #[test]
    fn test_process_retirement_negative_nbv() {
        let result = AssetRetirementService::process_retirement("ASSET-104", -100.0, 500.0, 0.0);
        assert_eq!(result.status, "REJECTED_NEGATIVE_NBV");
    }

    #[test]
    fn test_reinstate_asset_valid() {
        let result = AssetRetirementService::reinstate_asset("RET-ASSET-101");
        assert_eq!(result, "REINSTATED");
    }

    #[test]
    fn test_reinstate_asset_invalid() {
        let result = AssetRetirementService::reinstate_asset("");
        assert_eq!(result, "INVALID_RETIREMENT_ID");
    }
}
