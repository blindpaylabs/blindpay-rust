//! The `instances.onboarding` sub-resource: read the instance's onboarding
//! (KYB) answers.

use std::sync::Arc;

use serde::Deserialize;

use crate::client::Inner;
use crate::common::Country;
use crate::error::Result;

/// The instance's business details (`GET …/onboarding/business_details`).
/// Every field is optional: only what was filled in is returned.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OnboardingBusinessDetails {
    /// Legal name.
    #[serde(default)]
    pub legal_name: Option<String>,
    /// Tax id.
    #[serde(default)]
    pub tax_id: Option<String>,
    /// Formation date (ISO 8601).
    #[serde(default)]
    pub formation_date: Option<String>,
    /// Trading names.
    #[serde(default)]
    pub trading_names: Option<Vec<String>>,
    /// First address line.
    #[serde(default)]
    pub address_line_1: Option<String>,
    /// Second address line.
    #[serde(default)]
    pub address_line_2: Option<String>,
    /// City.
    #[serde(default)]
    pub city: Option<String>,
    /// State, province or region.
    #[serde(default)]
    pub state_province_region: Option<String>,
    /// Postal code.
    #[serde(default)]
    pub postal_code: Option<String>,
    /// Country.
    #[serde(default)]
    pub country: Option<Country>,
    /// Entity type (`corporation`, `company`, …).
    #[serde(default)]
    pub entity_type: Option<String>,
    /// State of incorporation.
    #[serde(default)]
    pub state_of_incorporation: Option<String>,
}

/// The instance's business profile (`GET …/onboarding/business_profile`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct OnboardingBusinessProfile {
    /// Custody scenario (`custodial`, `non_custodial`, …).
    #[serde(default)]
    pub scenario: Option<String>,
    /// Scenario description.
    #[serde(default)]
    pub scenario_description: Option<String>,
    /// Whether the business runs its own KYC screening.
    #[serde(default)]
    pub has_kyc_screening: Option<bool>,
    /// Whether the business is regulated.
    #[serde(default)]
    pub is_regulated: Option<bool>,
    /// Industry (`wallet`, `exchange`, …).
    #[serde(default)]
    pub business_industry: Option<String>,
    /// Website.
    #[serde(default)]
    pub website: Option<String>,
    /// Countries with the most volume.
    #[serde(default)]
    pub top_volume_by_country: Option<Vec<Country>>,
    /// Estimated monthly volume bucket.
    #[serde(default)]
    pub estimated_volume_per_month: Option<String>,
}

/// The instance's ownership documents (`GET …/onboarding/ownership_documents`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct OnboardingOwnershipDocuments {
    /// Incorporation document URL.
    #[serde(default)]
    pub incorporation_doc_file: Option<String>,
    /// Proof-of-ownership document URL.
    #[serde(default)]
    pub proof_of_ownership_doc_file: Option<String>,
    /// Proof-of-address document type.
    #[serde(default)]
    pub proof_of_address_doc_type: Option<String>,
    /// Proof-of-address document URL.
    #[serde(default)]
    pub proof_of_address_doc_file: Option<String>,
    /// Beneficial owners and controlling persons, as raw JSON.
    #[serde(default)]
    pub owners: Option<Vec<serde_json::Value>>,
}

/// Handle for `instances.onboarding`.
#[derive(Clone)]
pub struct Onboarding {
    client: Arc<Inner>,
}

impl Onboarding {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    fn path(&self, step: &str) -> String {
        format!("/instances/{}/onboarding/{step}", self.client.instance_id)
    }

    /// The instance's business details.
    ///
    /// `GET /instances/{instance_id}/onboarding/business_details`
    pub async fn get_business_details(&self) -> Result<OnboardingBusinessDetails> {
        self.client.get(&self.path("business_details")).await
    }

    /// The instance's business profile.
    ///
    /// `GET /instances/{instance_id}/onboarding/business_profile`
    pub async fn get_business_profile(&self) -> Result<OnboardingBusinessProfile> {
        self.client.get(&self.path("business_profile")).await
    }

    /// The instance's ownership documents.
    ///
    /// `GET /instances/{instance_id}/onboarding/ownership_documents`
    pub async fn get_ownership_documents(&self) -> Result<OnboardingOwnershipDocuments> {
        self.client.get(&self.path("ownership_documents")).await
    }
}
