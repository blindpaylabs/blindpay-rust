//! The `customers` resource: KYC/KYB customer records and their bank accounts.
//!
//! Customers are instance-scoped. The handle exposes the
//! [`bank_accounts`](Customers::bank_accounts) sub-resource.

mod bank_accounts;
mod types;

pub use bank_accounts::{
    AchCopDocument, BankAccount, BankAccountStatus, BankAccounts, CreateAchCopBitsoInput,
    CreateAchInput, CreateInternationalSwiftInput, CreatePixInput, CreatePixSafeInput,
    CreateRtpInput, CreateSepaInput, CreateSpeiBitsoInput, CreateTedInput,
    CreateTransfersBitsoInput, CreateWireInput, ListBankAccountsParams, RecipientRelationship,
    SpeiProtocol, SwiftPaymentCode,
};
pub use types::{
    AmlHits, BusinessType, CreateBusinessWithStandardKybInput, CreateCustomerResponse,
    CreateIndividualWithEnhancedKycInput, CreateIndividualWithStandardKycInput, Customer,
    CustomerLimit, CustomerLimits, FraudWarning, GetCustomerLimitsResponse, IdentificationDocument,
    KycType, KycWarning, LimitIncrease, LimitIncreaseStatus, LimitWindow, ListCustomersParams,
    Owner, OwnerRole, OwnerTaxType, ProofOfAddressDocType, RequestLimitIncreaseInput,
    RequestLimitIncreaseResponse, SupportingDocumentType, UpdateCustomerInput,
};

use std::sync::Arc;

use reqwest::Method;
use serde::Serialize;

use crate::client::Inner;
use crate::common::{ListResponse, Success};
use crate::error::Result;
use crate::internal::encode_path_segment;

/// Handle for the `customers` resource.
///
/// Obtained from the `customers` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Customers {
    client: Arc<Inner>,
    /// Bank accounts nested under a customer.
    pub bank_accounts: BankAccounts,
}

impl Customers {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self {
            bank_accounts: BankAccounts::new(Arc::clone(&client)),
            client,
        }
    }

    /// Lists customers for the instance.
    ///
    /// `GET /instances/{instance_id}/customers`
    pub async fn list(&self, params: &ListCustomersParams) -> Result<ListResponse<Customer>> {
        let path = format!("/instances/{}/customers", self.client.instance_id);
        self.client.get_query(&path, params).await
    }

    /// Creates an individual customer with standard KYC.
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_individual_with_standard_kyc(
        &self,
        input: &CreateIndividualWithStandardKycInput,
    ) -> Result<CreateCustomerResponse> {
        self.create(Discriminated {
            account_type: "individual",
            kyc_type: "standard",
            input,
        })
        .await
    }

    /// Creates an individual customer with enhanced KYC.
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_individual_with_enhanced_kyc(
        &self,
        input: &CreateIndividualWithEnhancedKycInput,
    ) -> Result<CreateCustomerResponse> {
        self.create(Discriminated {
            account_type: "individual",
            kyc_type: "enhanced",
            input,
        })
        .await
    }

    /// Creates a business customer with standard KYB.
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_business_with_standard_kyb(
        &self,
        input: &CreateBusinessWithStandardKybInput,
    ) -> Result<CreateCustomerResponse> {
        self.create(Discriminated {
            account_type: "business",
            kyc_type: "standard",
            input,
        })
        .await
    }

    async fn create<T: Serialize>(
        &self,
        body: Discriminated<'_, T>,
    ) -> Result<CreateCustomerResponse> {
        let path = format!("/instances/{}/customers", self.client.instance_id);
        self.client
            .request(Method::POST, &path, None::<&()>, Some(&body))
            .await
    }

    /// Retrieves a single customer by ID.
    ///
    /// `GET /instances/{instance_id}/customers/{id}`
    pub async fn get(&self, customer_id: impl AsRef<str>) -> Result<Customer> {
        let path = format!(
            "/instances/{}/customers/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Updates a customer. All body fields are optional.
    ///
    /// `PUT /instances/{instance_id}/customers/{id}`
    pub async fn update(
        &self,
        customer_id: impl AsRef<str>,
        input: &UpdateCustomerInput,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request(Method::PUT, &path, None::<&()>, Some(input))
            .await
    }

    /// Deletes a customer.
    ///
    /// `DELETE /instances/{instance_id}/customers/{id}`
    pub async fn delete(&self, customer_id: impl AsRef<str>) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request(Method::DELETE, &path, None::<&()>, None::<&()>)
            .await
    }

    /// Retrieves a customer's aggregate payin/payout limits.
    ///
    /// `GET /instances/{instance_id}/limits/customers/{id}`
    pub async fn get_limits(
        &self,
        customer_id: impl AsRef<str>,
    ) -> Result<GetCustomerLimitsResponse> {
        let path = format!(
            "/instances/{}/limits/customers/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Lists a customer's limit-increase requests.
    ///
    /// `GET /instances/{instance_id}/customers/{customer_id}/limit-increase`
    pub async fn get_limit_increase_requests(
        &self,
        customer_id: impl AsRef<str>,
    ) -> Result<Vec<LimitIncrease>> {
        let path = format!(
            "/instances/{}/customers/{}/limit-increase",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client.get(&path).await
    }

    /// Requests a limit increase for a customer.
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/limit-increase`
    pub async fn request_limit_increase(
        &self,
        customer_id: impl AsRef<str>,
        input: &RequestLimitIncreaseInput,
    ) -> Result<RequestLimitIncreaseResponse> {
        let path = format!(
            "/instances/{}/customers/{}/limit-increase",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request(Method::POST, &path, None::<&()>, Some(input))
            .await
    }
}

/// Wraps a per-variant create input with the `type` + `kyc_type` discriminators
/// the API's discriminated union expects, flattening the input alongside them.
#[derive(Serialize)]
struct Discriminated<'a, T: Serialize> {
    #[serde(rename = "type")]
    account_type: &'static str,
    kyc_type: &'static str,
    #[serde(flatten)]
    input: &'a T,
}
