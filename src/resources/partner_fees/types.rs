//! Request and response types for the `partner_fees` resource.

use serde::{Deserialize, Serialize};

/// A partner fee configuration.
///
/// Models the full `PartnerFee` superset returned by the list endpoint. The
/// `get` and `create` endpoints return a narrower create-output shape that omits
/// `virtual_account_set`, `created_at`, and `updated_at`, so those fields carry
/// `#[serde(default)]` and stay `None` when absent.
///
/// Fee amounts are integers: the `*_flat_fee` fields are in cents and the
/// `*_percentage_fee` fields are in basis points.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct PartnerFee {
    /// Unique identifier, prefixed `pf_`.
    pub id: String,
    /// The instance this partner fee belongs to.
    pub instance_id: String,
    /// Human-readable display name.
    pub name: String,
    /// Payout percentage fee, in basis points.
    pub payout_percentage_fee: i64,
    /// Payout flat fee, in cents.
    pub payout_flat_fee: i64,
    /// Payin percentage fee, in basis points.
    pub payin_percentage_fee: i64,
    /// Payin flat fee, in cents.
    pub payin_flat_fee: i64,
    /// Whether the fee is tied to a virtual account set. Absent on the
    /// `get`/`create` responses.
    #[serde(default)]
    pub virtual_account_set: Option<bool>,
    /// Creation timestamp. Absent on the `get`/`create` responses.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last-update timestamp. Absent on the `get`/`create` responses.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Body for creating a partner fee.
///
/// All fee amounts are integers (flat fees in cents, percentage fees in basis
/// points).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatePartnerFeeInput {
    /// Human-readable display name.
    pub name: String,
    /// Payout percentage fee, in basis points.
    pub payout_percentage_fee: i64,
    /// Payout flat fee, in cents.
    pub payout_flat_fee: i64,
    /// Payin percentage fee, in basis points.
    pub payin_percentage_fee: i64,
    /// Payin flat fee, in cents.
    pub payin_flat_fee: i64,
    /// Whether the fee is tied to a virtual account set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_account_set: Option<bool>,
}

impl CreatePartnerFeeInput {
    /// Creates a partner-fee input with the required fields.
    pub fn new(
        name: impl Into<String>,
        payout_percentage_fee: i64,
        payout_flat_fee: i64,
        payin_percentage_fee: i64,
        payin_flat_fee: i64,
    ) -> Self {
        Self {
            name: name.into(),
            payout_percentage_fee,
            payout_flat_fee,
            payin_percentage_fee,
            payin_flat_fee,
            virtual_account_set: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_input_omits_unset_virtual_account_set() {
        let input = CreatePartnerFeeInput::new("Display Name", 100, 1000, 50, 500);
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json["name"], "Display Name");
        assert_eq!(json["payout_percentage_fee"], 100);
        assert!(json.get("virtual_account_set").is_none());
    }

    #[test]
    fn create_input_serializes_virtual_account_set_when_set() {
        let mut input = CreatePartnerFeeInput::new("Display Name", 0, 0, 0, 0);
        input.virtual_account_set = Some(true);
        let json = serde_json::to_value(&input).unwrap();
        assert_eq!(json["virtual_account_set"], true);
    }

    #[test]
    fn partner_fee_parses_create_out_shape_without_extras() {
        let fee: PartnerFee = serde_json::from_value(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "name": "Display Name",
            "payout_percentage_fee": 0,
            "payout_flat_fee": 0,
            "payin_percentage_fee": 0,
            "payin_flat_fee": 0
        }))
        .unwrap();
        assert_eq!(fee.id, "fe_123");
        assert_eq!(fee.virtual_account_set, None);
        assert_eq!(fee.created_at, None);
        assert_eq!(fee.updated_at, None);
    }

    #[test]
    fn partner_fee_parses_full_superset() {
        let fee: PartnerFee = serde_json::from_value(serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "name": "Display Name",
            "payout_percentage_fee": 100,
            "payout_flat_fee": 1000,
            "payin_percentage_fee": 50,
            "payin_flat_fee": 500,
            "virtual_account_set": true,
            "created_at": "2021-01-01T00:00:00Z",
            "updated_at": "2021-01-02T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(fee.virtual_account_set, Some(true));
        assert_eq!(fee.payout_flat_fee, 1000);
        assert_eq!(fee.created_at.as_deref(), Some("2021-01-01T00:00:00Z"));
    }
}
