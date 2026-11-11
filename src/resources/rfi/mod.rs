//! Requests for information (RFIs): compliance questions BlindPay asks about
//! an instance ([`InstanceRfis`]) or a customer ([`CustomerRfis`]).
//!
//! `get` returns the open RFI, or `None` when none is pending. Answer it with
//! `submit`, passing a map keyed by each [`RfiField::key`]; the map is sent as
//! the whole JSON body.
//!
//! ```no_run
//! # async fn run(client: blindpay::BlindPay) -> blindpay::Result<()> {
//! use blindpay::{RfiAnswer, RfiAnswers};
//!
//! if let Some(rfi) = client.instances.rfi.get().await? {
//!     let mut answers = RfiAnswers::new();
//!     for section in &rfi.request {
//!         for field in &section.fields {
//!             answers.insert(field.key.clone(), RfiAnswer::from("…"));
//!         }
//!     }
//!     client.instances.rfi.submit(&answers).await?;
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::Inner;
use crate::common::{AccountClass, open_enum};
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

open_enum! {
    /// The lifecycle status of an RFI.
    pub enum RfiStatus {
        /// Waiting for answers.
        Pending => "pending",
        /// Answered.
        Submitted => "submitted",
        /// Not answered in time.
        Expired => "expired",
        /// Withdrawn by BlindPay.
        Cancelled => "cancelled",
    }
}

open_enum! {
    /// Why a submit was not applied.
    pub enum RfiSubmitFailureReason {
        /// There is no pending RFI to answer.
        NoPendingRfi => "no_pending_rfi",
        /// The RFI was already answered or closed.
        AlreadyResolved => "already_resolved",
    }
}

/// A selectable option of an [`RfiField`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RfiFieldOption {
    /// Display label.
    pub label: String,
    /// Value to submit.
    pub value: String,
}

/// A question in an [`RfiSection`] (spec `RfiField`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RfiField {
    /// The key to answer under in [`RfiAnswers`].
    pub key: String,
    /// The question.
    pub label: String,
    /// Whether an answer is required.
    pub required: bool,
    /// A pattern a text answer must match.
    #[serde(default)]
    pub regex: Option<String>,
    /// Allowed values, for a choice question.
    #[serde(default)]
    pub items: Option<Vec<RfiFieldOption>>,
    /// Whether several values (e.g. several file URLs) may be given.
    #[serde(default)]
    pub multiple: Option<bool>,
}

/// A group of questions (spec `RfiSection`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RfiSection {
    /// Section title.
    pub title: String,
    /// Section description.
    pub description: String,
    /// A document BlindPay attached for context.
    #[serde(default)]
    pub supporting_document: Option<String>,
    /// The questions.
    pub fields: Vec<RfiField>,
}

/// An answer: free text, or a list of uploaded file URLs (see `Upload`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RfiAnswer {
    /// A text answer.
    Text(String),
    /// Uploaded file URLs.
    Files(Vec<String>),
}

impl From<&str> for RfiAnswer {
    fn from(text: &str) -> Self {
        RfiAnswer::Text(text.to_string())
    }
}

impl From<String> for RfiAnswer {
    fn from(text: String) -> Self {
        RfiAnswer::Text(text)
    }
}

impl From<Vec<String>> for RfiAnswer {
    fn from(files: Vec<String>) -> Self {
        RfiAnswer::Files(files)
    }
}

/// Answers keyed by [`RfiField::key`]; sent as the whole request body.
pub type RfiAnswers = BTreeMap<String, RfiAnswer>;

/// An instance RFI (spec `InstanceRfi`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct InstanceRfi {
    /// RFI id.
    pub id: String,
    /// Instance id (`in_…`).
    pub instance_id: String,
    /// Status.
    pub status: RfiStatus,
    /// The questions, grouped in sections.
    pub request: Vec<RfiSection>,
    /// Answers submitted so far.
    #[serde(default)]
    pub response: RfiAnswers,
    /// Deadline (ISO 8601).
    pub expires_at: String,
    /// When it was answered. May be `null`.
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
}

/// A customer RFI (spec `Rfi`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct CustomerRfi {
    /// RFI id.
    pub id: String,
    /// Customer id (`re_…`).
    pub customer_id: String,
    /// Instance id (`in_…`).
    pub instance_id: String,
    /// Status.
    pub status: RfiStatus,
    /// The questions, grouped in sections.
    pub request: Vec<RfiSection>,
    /// Answers submitted so far.
    #[serde(default)]
    pub response: RfiAnswers,
    /// Deadline (ISO 8601).
    pub expires_at: String,
    /// When it was answered. May be `null`.
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Individual or business.
    #[serde(default)]
    pub customer_type: Option<AccountClass>,
    /// The customer's KYC status.
    #[serde(default)]
    pub customer_kyc_status: Option<String>,
}

/// The result of an RFI submit: `success`, or why it wasn't applied.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RfiSubmitResponse {
    /// Whether the answers were recorded.
    pub success: bool,
    /// Why not, when `success` is `false`.
    #[serde(default)]
    pub reason: Option<RfiSubmitFailureReason>,
}

/// Handle for the instance RFI (`instances.rfi`).
#[derive(Clone)]
pub struct InstanceRfis {
    client: Arc<Inner>,
}

impl InstanceRfis {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    fn path(&self) -> String {
        format!("/instances/{}/rfi", self.client.instance_id)
    }

    /// The instance's open RFI, or `None` when none is pending.
    ///
    /// `GET /instances/{instance_id}/rfi`
    pub async fn get(&self) -> Result<Option<InstanceRfi>> {
        self.client.get(&self.path()).await
    }

    /// Answers the open RFI.
    ///
    /// `POST /instances/{instance_id}/rfi`
    pub async fn submit(&self, answers: &RfiAnswers) -> Result<RfiSubmitResponse> {
        self.submit_with_options(answers, &RequestOptions::default())
            .await
    }

    /// Like [`Self::submit`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/rfi`
    pub async fn submit_with_options(
        &self,
        answers: &RfiAnswers,
        options: &RequestOptions,
    ) -> Result<RfiSubmitResponse> {
        self.client
            .request_with::<_, (), _>(Method::POST, &self.path(), None, Some(answers), options)
            .await
    }
}

/// Handle for customer RFIs (`customers.rfi`).
#[derive(Clone)]
pub struct CustomerRfis {
    client: Arc<Inner>,
}

impl CustomerRfis {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    fn path(&self, customer_id: &str) -> String {
        format!(
            "/instances/{}/customers/{}/rfi",
            self.client.instance_id,
            encode_path_segment(customer_id)
        )
    }

    /// The customer's open RFI, or `None` when none is pending.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/rfi`
    pub async fn get(&self, customer_id: impl AsRef<str>) -> Result<Option<CustomerRfi>> {
        self.client.get(&self.path(customer_id.as_ref())).await
    }

    /// Answers the customer's open RFI.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/rfi`
    pub async fn submit(
        &self,
        customer_id: impl AsRef<str>,
        answers: &RfiAnswers,
    ) -> Result<RfiSubmitResponse> {
        self.submit_with_options(customer_id, answers, &RequestOptions::default())
            .await
    }

    /// Like [`Self::submit`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/rfi`
    pub async fn submit_with_options(
        &self,
        customer_id: impl AsRef<str>,
        answers: &RfiAnswers,
        options: &RequestOptions,
    ) -> Result<RfiSubmitResponse> {
        let path = self.path(customer_id.as_ref());
        self.client
            .request_with::<_, (), _>(Method::POST, &path, None, Some(answers), options)
            .await
    }
}
