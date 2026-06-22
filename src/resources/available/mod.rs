//! The `available` resource: discovery endpoints for payment rails, bank-detail
//! field definitions, NAICS industry codes, and SWIFT/BIC lookups.
//!
//! These endpoints are instance-agnostic and require only a valid API key.

mod types;

pub use crate::common::Rail;
pub use types::{
    BankDetail, BankDetailItem, BankDetailKey, NaicsCode, RailEntry, RequiredWhen,
    RequiredWhenOperator, SwiftCodeBankDetail,
};

use std::sync::Arc;

use serde::Serialize;

use crate::client::Inner;
use crate::error::{Error, Result};
use crate::internal::encode_path_segment;

/// Handle for the `available` resource.
///
/// Obtained from the `available` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Available {
    client: Arc<Inner>,
}

impl Available {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Lists every payment rail available for the account.
    ///
    /// `GET /available/rails`
    pub async fn get_rails(&self) -> Result<Vec<RailEntry>> {
        self.client.get("/available/rails").await
    }

    /// Returns the bank-detail field definitions needed to create a bank account
    /// for the given `rail`.
    ///
    /// `GET /available/bank-details?rail={rail}`
    pub async fn get_bank_details(&self, rail: Rail) -> Result<Vec<BankDetail>> {
        #[derive(Serialize)]
        struct Query<'a> {
            rail: &'a str,
        }

        self.client
            .get_query(
                "/available/bank-details",
                &Query {
                    rail: rail.as_str(),
                },
            )
            .await
    }

    /// Lists the NAICS business-industry codes accepted by BlindPay.
    ///
    /// `GET /available/naics`
    pub async fn get_naics_codes(&self) -> Result<Vec<NaicsCode>> {
        self.client.get("/available/naics").await
    }

    /// Looks up bank details for a SWIFT/BIC code (8 or 11 characters).
    ///
    /// `GET /available/swift/{swift}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if `swift` is empty.
    pub async fn get_swift_code_bank_details(
        &self,
        swift: impl AsRef<str>,
    ) -> Result<Vec<SwiftCodeBankDetail>> {
        let swift = swift.as_ref().trim();
        if swift.is_empty() {
            return Err(Error::Config("swift code cannot be empty".to_string()));
        }
        let path = format!("/available/swift/{}", encode_path_segment(swift));
        self.client.get(&path).await
    }
}
