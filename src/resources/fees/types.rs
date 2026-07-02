//! Request and response types for the `fees` resource.

use serde::Deserialize;

/// The fee configuration for a single rail or chain.
///
/// Flat amounts are integer cents; percentage amounts are integer basis points.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct FeeOptions {
    /// Payin flat fee, in cents.
    pub payin_flat: i64,
    /// Payin percentage fee, in basis points.
    pub payin_percentage: i64,
    /// Payout flat fee, in cents.
    pub payout_flat: i64,
    /// Payout percentage fee, in basis points.
    pub payout_percentage: i64,
}

/// The full fee schedule for an instance, returned by `Fees::get`.
///
/// Each field is the [`FeeOptions`] for one rail or chain. The keys are the
/// literal wire field names (note the non-`Rail` names `domestic_wire`,
/// `ach_colombia`, `transfers_3`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Fee {
    /// Fee record identifier (prefix `fe_`).
    pub id: String,
    /// The instance these fees belong to.
    pub instance_id: String,
    /// ACH (US) fees.
    pub ach: FeeOptions,
    /// Domestic wire (US) fees.
    pub domestic_wire: FeeOptions,
    /// RTP (US) fees.
    pub rtp: FeeOptions,
    /// International SWIFT fees.
    pub international_swift: FeeOptions,
    /// PIX (Brazil) fees.
    pub pix: FeeOptions,
    /// PIX Safe (Brazil) fees.
    pub pix_safe: FeeOptions,
    /// TED (Brazil) fees. May be absent.
    #[serde(default)]
    pub ted: Option<FeeOptions>,
    /// ACH Colombia fees.
    pub ach_colombia: FeeOptions,
    /// Transfers 3.0 fees.
    pub transfers_3: FeeOptions,
    /// SPEI (Mexico) fees.
    pub spei: FeeOptions,
    /// SEPA (Europe) fees.
    pub sepa: FeeOptions,
    /// Tron chain fees.
    pub tron: FeeOptions,
    /// Ethereum chain fees.
    pub ethereum: FeeOptions,
    /// Polygon chain fees.
    pub polygon: FeeOptions,
    /// Base chain fees.
    pub base: FeeOptions,
    /// Arbitrum chain fees.
    pub arbitrum: FeeOptions,
    /// Stellar chain fees.
    pub stellar: FeeOptions,
    /// Solana chain fees.
    pub solana: FeeOptions,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// ISO-8601 last-update timestamp.
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_parses_with_optional_ted_absent() {
        let opt = serde_json::json!({
            "payin_flat": 40,
            "payin_percentage": 50,
            "payout_flat": 40,
            "payout_percentage": 50
        });
        let body = serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "ach": opt,
            "domestic_wire": opt,
            "rtp": opt,
            "international_swift": opt,
            "pix": opt,
            "pix_safe": opt,
            "ach_colombia": opt,
            "transfers_3": opt,
            "spei": opt,
            "sepa": opt,
            "tron": opt,
            "ethereum": opt,
            "polygon": opt,
            "base": opt,
            "arbitrum": opt,
            "stellar": opt,
            "solana": opt,
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-02T00:00:00.000Z"
        });

        let fee: Fee = serde_json::from_value(body).unwrap();
        assert_eq!(fee.id, "fe_123");
        assert_eq!(fee.ach.payin_flat, 40);
        assert_eq!(fee.sepa.payout_percentage, 50);
        assert!(fee.ted.is_none());
    }

    #[test]
    fn fee_parses_with_ted_present() {
        let opt = serde_json::json!({
            "payin_flat": 1,
            "payin_percentage": 2,
            "payout_flat": 3,
            "payout_percentage": 4
        });
        let mut body = serde_json::json!({
            "id": "fe_123",
            "instance_id": "in_test",
            "created_at": "2024-01-01T00:00:00.000Z",
            "updated_at": "2024-01-02T00:00:00.000Z"
        });
        for key in [
            "ach",
            "domestic_wire",
            "rtp",
            "international_swift",
            "pix",
            "pix_safe",
            "ted",
            "ach_colombia",
            "transfers_3",
            "spei",
            "sepa",
            "tron",
            "ethereum",
            "polygon",
            "base",
            "arbitrum",
            "stellar",
            "solana",
        ] {
            body[key] = opt.clone();
        }

        let fee: Fee = serde_json::from_value(body).unwrap();
        let ted = fee.ted.expect("ted should be present");
        assert_eq!(ted.payin_flat, 1);
        assert_eq!(ted.payout_percentage, 4);
    }
}
