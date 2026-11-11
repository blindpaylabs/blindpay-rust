//! Response types for payables.

use serde::Deserialize;

use crate::common::{Currency, open_enum};

open_enum! {
    /// The lifecycle status of a payable.
    pub enum PayableStatus {
        /// Not yet paid; editable.
        Draft => "draft",
        /// A payout executing it is in flight.
        Processing => "processing",
        /// Paid.
        Completed => "completed",
        /// Deleted while a draft.
        Canceled => "canceled",
    }
}

open_enum! {
    /// Why a payable returned to draft (set on that `payable.update` event).
    pub enum LastAttemptStatus {
        /// The payout failed before funding.
        Failed => "failed",
        /// The payout was refunded after funding.
        Refunded => "refunded",
    }
}

/// A payable, as returned by the payables endpoints and carried by the
/// `payable.new`, `payable.update` and `payable.complete` webhook events
/// (spec `PayableOut`).
///
/// The nested `from`, `to`, `line_items` and `bank_account` objects are kept
/// as raw JSON for now.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Payable {
    /// Payable id (`pb_…`).
    pub id: String,
    /// Owning instance id (`in_…`).
    pub instance_id: String,
    /// The customer paying it (`re_…`).
    pub customer_id: String,
    /// ISO 4217 currency of the payable.
    pub currency: Currency,
    /// Lifecycle status.
    pub status: PayableStatus,
    /// Derived total in cents of `currency` (line items + taxes − discount).
    /// May be `null`.
    #[serde(default)]
    pub amount: Option<i64>,
    /// The payer party (raw JSON).
    #[serde(default)]
    pub from: Option<serde_json::Value>,
    /// The beneficiary party (raw JSON).
    #[serde(default)]
    pub to: Option<serde_json::Value>,
    /// Invoice line items (raw JSON).
    #[serde(default)]
    pub line_items: Option<Vec<serde_json::Value>>,
    /// Free-text note. May be `null`.
    #[serde(default)]
    pub note: Option<String>,
    /// Discount in cents. May be `null`.
    #[serde(default)]
    pub discount: Option<i64>,
    /// Taxes in cents. May be `null`.
    #[serde(default)]
    pub taxes: Option<i64>,
    /// Boleto or arrecadação code. May be `null`.
    #[serde(default)]
    pub boleto_barcode: Option<String>,
    /// PIX copia-e-cola payload. May be `null`.
    #[serde(default)]
    pub pix_qrcode: Option<String>,
    /// Document file URL. May be `null`.
    #[serde(default)]
    pub document_file: Option<String>,
    /// Due date (`YYYY-MM-DD`). May be `null`.
    #[serde(default)]
    pub due_date: Option<String>,
    /// When to pay it automatically. May be `null`.
    #[serde(default)]
    pub scheduled_at: Option<String>,
    /// The payout executing it, once one exists. May be `null`.
    #[serde(default)]
    pub payout_id: Option<String>,
    /// The beneficiary bank account (raw JSON).
    #[serde(default)]
    pub bank_account: Option<serde_json::Value>,
    /// Why it returned to draft. May be `null`.
    #[serde(default)]
    pub last_attempt_status: Option<LastAttemptStatus>,
    /// Customer first name. May be `null`.
    #[serde(default)]
    pub first_name: Option<String>,
    /// Customer last name. May be `null`.
    #[serde(default)]
    pub last_name: Option<String>,
    /// Customer legal name. May be `null`.
    #[serde(default)]
    pub legal_name: Option<String>,
    /// Creation timestamp. May be `null`.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last-update timestamp. May be `null`.
    #[serde(default)]
    pub updated_at: Option<String>,
}
