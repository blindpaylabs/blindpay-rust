//! Request and response types for payables.

use serde::{Deserialize, Serialize};

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
/// The beneficiary `bank_account` is kept as raw JSON.
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
    /// Who issued the invoice / is paying the bill. May be `null`.
    #[serde(default)]
    pub from: Option<PayableParty>,
    /// The beneficiary being paid. May be `null`.
    #[serde(default)]
    pub to: Option<PayableParty>,
    /// Invoice line items.
    #[serde(default)]
    pub line_items: Option<Vec<PayableLineItem>>,
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

open_enum! {
    /// The kind of payable, used to filter `Payables::list`.
    pub enum PayableType {
        /// A Brazilian boleto.
        Boleto => "boleto",
        /// A PIX copia-e-cola code.
        PixCode => "pix_code",
        /// An invoice with line items.
        Invoice => "invoice",
    }
}

/// A party on a payable (spec `PayableParty`): every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PayableParty {
    /// Legal name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,
    /// First address line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_line_2: Option<String>,
    /// City.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// State, province or region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_province_region: Option<String>,
    /// Postal code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Tax id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
}

impl PayableParty {
    /// Creates an empty party; set the fields you know.
    pub fn new() -> Self {
        Self::default()
    }
}

/// An invoice line item (spec `PayableLineItem`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PayableLineItem {
    /// Item description.
    pub name: String,
    /// Quantity (at least 1).
    pub quantity: i64,
    /// Unit price in integer cents of the payable currency (`150000` =
    /// 1,500.00). May be `null`.
    pub price: Option<i64>,
}

impl PayableLineItem {
    /// Creates a line item; `price` is in integer cents.
    pub fn new(name: impl Into<String>, quantity: i64, price: i64) -> Self {
        Self {
            name: name.into(),
            quantity,
            price: Some(price),
        }
    }
}

/// Body for `Payables::create` (spec `CreatePayableIn`).
///
/// Amounts (`discount`, `taxes`, line-item prices) are integer cents of
/// `currency`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CreatePayableInput {
    /// The customer paying it (`re_…`).
    pub customer_id: String,
    /// ISO 4217 currency of the payable.
    pub currency: Currency,
    /// Who issued the invoice / is paying the bill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<PayableParty>,
    /// The beneficiary being paid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<PayableParty>,
    /// Invoice line items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<PayableLineItem>>,
    /// Free-text note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Discount in integer cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<i64>,
    /// Taxes in integer cents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxes: Option<i64>,
    /// Boleto or arrecadação barcode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boleto_barcode: Option<String>,
    /// PIX copia-e-cola payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pix_qrcode: Option<String>,
    /// Document file URL (see `Upload`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_file: Option<String>,
    /// Due date (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// When to pay it automatically (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
    /// The beneficiary bank account, shaped like a bank-account create body
    /// (spec `CreateBankAccountIn`, including its `type`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account: Option<serde_json::Value>,
}

impl CreatePayableInput {
    /// Creates a payable body for a customer and currency.
    pub fn new(customer_id: impl Into<String>, currency: Currency) -> Self {
        Self {
            customer_id: customer_id.into(),
            currency,
            from: None,
            to: None,
            line_items: None,
            note: None,
            discount: None,
            taxes: None,
            boleto_barcode: None,
            pix_qrcode: None,
            document_file: None,
            due_date: None,
            scheduled_at: None,
            bank_account: None,
        }
    }

    /// Sets `from`: who issued the invoice / is paying the bill.
    pub fn from(mut self, from: PayableParty) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets `to`: the beneficiary being paid.
    pub fn to(mut self, to: PayableParty) -> Self {
        self.to = Some(to);
        self
    }

    /// Sets `line_items`: invoice line items.
    pub fn line_items(mut self, line_items: Vec<PayableLineItem>) -> Self {
        self.line_items = Some(line_items);
        self
    }

    /// Sets `note`: free-text note.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Sets `discount`: discount in integer cents.
    pub fn discount(mut self, discount: i64) -> Self {
        self.discount = Some(discount);
        self
    }

    /// Sets `taxes`: taxes in integer cents.
    pub fn taxes(mut self, taxes: i64) -> Self {
        self.taxes = Some(taxes);
        self
    }

    /// Sets `boleto_barcode`: boleto or arrecadação barcode.
    pub fn boleto_barcode(mut self, boleto_barcode: impl Into<String>) -> Self {
        self.boleto_barcode = Some(boleto_barcode.into());
        self
    }

    /// Sets `pix_qrcode`: pIX copia-e-cola payload.
    pub fn pix_qrcode(mut self, pix_qrcode: impl Into<String>) -> Self {
        self.pix_qrcode = Some(pix_qrcode.into());
        self
    }

    /// Sets `document_file`: document file URL (see `Upload`).
    pub fn document_file(mut self, document_file: impl Into<String>) -> Self {
        self.document_file = Some(document_file.into());
        self
    }

    /// Sets `due_date`: due date (`YYYY-MM-DD`).
    pub fn due_date(mut self, due_date: impl Into<String>) -> Self {
        self.due_date = Some(due_date.into());
        self
    }

    /// Sets `scheduled_at`: when to pay it automatically (ISO 8601).
    pub fn scheduled_at(mut self, scheduled_at: impl Into<String>) -> Self {
        self.scheduled_at = Some(scheduled_at.into());
        self
    }

    /// Sets `bank_account`: (spec `CreateBankAccountIn`, including its `type`).
    pub fn bank_account(mut self, bank_account: serde_json::Value) -> Self {
        self.bank_account = Some(bank_account);
        self
    }
}

/// Query parameters for `Payables::list` and `Payables::list_auto_paging`.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct ListPayablesParams {
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<crate::common::Limit>,
    /// Cursor: return items after this payable id (`next_page`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_after: Option<String>,
    /// Cursor: return items before this payable id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_before: Option<String>,
    /// Filter by status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PayableStatus>,
    /// Filter by customer id (`re_…`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    /// Filter by customer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<String>,
    /// Filter by payable kind.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub payable_type: Option<PayableType>,
    /// Filter by beneficiary name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary_name: Option<String>,
}

impl ListPayablesParams {
    /// Creates empty list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `limit`: maximum number of items to return.
    pub fn limit(mut self, limit: crate::common::Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets `starting_after`: cursor: return items after this payable id (`next_page`).
    pub fn starting_after(mut self, starting_after: impl Into<String>) -> Self {
        self.starting_after = Some(starting_after.into());
        self
    }

    /// Sets `ending_before`: cursor: return items before this payable id.
    pub fn ending_before(mut self, ending_before: impl Into<String>) -> Self {
        self.ending_before = Some(ending_before.into());
        self
    }

    /// Sets `status`: filter by status.
    pub fn status(mut self, status: PayableStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets `customer_id`: filter by customer id (`re_…`).
    pub fn customer_id(mut self, customer_id: impl Into<String>) -> Self {
        self.customer_id = Some(customer_id.into());
        self
    }

    /// Sets `customer_name`: filter by customer name.
    pub fn customer_name(mut self, customer_name: impl Into<String>) -> Self {
        self.customer_name = Some(customer_name.into());
        self
    }

    /// Sets `payable_type`: filter by payable kind.
    pub fn payable_type(mut self, payable_type: PayableType) -> Self {
        self.payable_type = Some(payable_type);
        self
    }

    /// Sets `beneficiary_name`: filter by beneficiary name.
    pub fn beneficiary_name(mut self, beneficiary_name: impl Into<String>) -> Self {
        self.beneficiary_name = Some(beneficiary_name.into());
        self
    }
}
