//! The `customers` resource: KYC/KYB customer records and their bank accounts.
//!
//! Customers are instance-scoped. The handle exposes the
//! [`bank_accounts`](Customers::bank_accounts) sub-resource.

mod bank_accounts;
mod types;

pub use bank_accounts::{
    AchCopDocument, BankAccount, BankAccountStatus, BankAccounts, CreateAchCopBitsoInput,
    CreateAchInput, CreateFasterPaymentsInput, CreateInternationalSwiftInput, CreatePixInput,
    CreatePixSafeInput, CreatePlaidLinkInput, CreateRtpInput, CreateSepaInput,
    CreateSpeiBitsoInput, CreateTedInput, CreateTransfersBitsoInput, CreateWireInput,
    ListBankAccountsParams, PlaidLink, RecipientRelationship, SpeiProtocol, SwiftPaymentCode,
};
pub use types::{
    AdditionalInfoItem, AdditionalInfoLabel, AmlHits, BusinessType,
    CreateBusinessWithStandardKybInput, CreateCustomerResponse,
    CreateIndividualWithEnhancedKycInput, CreateIndividualWithStandardKycInput, Customer,
    CustomerLimit, CustomerLimits, FraudWarning, GetCustomerLimitsResponse, IdentificationDocument,
    KycType, KycWarning, LimitIncrease, LimitIncreaseStatus, LimitIncreaseSupportingDocument,
    LimitWindow, ListCustomersParams, Owner, OwnerRole, OwnerTaxType, ProofOfAddressDocType,
    RequestLimitIncreaseInput, RequestLimitIncreaseResponse, SupportingDocumentType,
    UpdateCustomerInput,
};

use std::sync::Arc;

use reqwest::Method;
use serde::Serialize;

use crate::client::Inner;
use crate::common::{ListResponse, Pager, Success};
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;
use crate::resources::rfi::CustomerRfis;

/// Handle for the `customers` resource.
///
/// Obtained from the `customers` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Customers {
    client: Arc<Inner>,
    /// Bank accounts nested under a customer.
    pub bank_accounts: BankAccounts,
    /// Customer requests-for-information.
    pub rfi: CustomerRfis,
}

impl Customers {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self {
            bank_accounts: BankAccounts::new(Arc::clone(&client)),
            rfi: CustomerRfis::new(Arc::clone(&client)),
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

    /// Iterates every customer matching `params`, page by page (see [`Pager`]).
    /// Sends `limit` (100 unless set) and follows `pagination.next_page`.
    ///
    /// `GET /instances/{instance_id}/customers`
    pub fn list_auto_paging(&self, params: &ListCustomersParams) -> Pager<Customer> {
        let path = format!("/instances/{}/customers", self.client.instance_id);
        Pager::new(Arc::clone(&self.client), path, params)
    }

    /// Creates an individual customer with standard KYC.
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_individual_with_standard_kyc(
        &self,
        input: &CreateIndividualWithStandardKycInput,
    ) -> Result<CreateCustomerResponse> {
        self.create_individual_with_standard_kyc_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_individual_with_standard_kyc`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_individual_with_standard_kyc_with_options(
        &self,
        input: &CreateIndividualWithStandardKycInput,
        options: &RequestOptions,
    ) -> Result<CreateCustomerResponse> {
        self.create(
            Discriminated {
                account_type: "individual",
                kyc_type: "standard",
                input,
            },
            options,
        )
        .await
    }

    /// Creates an individual customer with enhanced KYC.
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_individual_with_enhanced_kyc(
        &self,
        input: &CreateIndividualWithEnhancedKycInput,
    ) -> Result<CreateCustomerResponse> {
        self.create_individual_with_enhanced_kyc_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_individual_with_enhanced_kyc`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_individual_with_enhanced_kyc_with_options(
        &self,
        input: &CreateIndividualWithEnhancedKycInput,
        options: &RequestOptions,
    ) -> Result<CreateCustomerResponse> {
        self.create(
            Discriminated {
                account_type: "individual",
                kyc_type: "enhanced",
                input,
            },
            options,
        )
        .await
    }

    /// Creates a business customer with standard KYB.
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_business_with_standard_kyb(
        &self,
        input: &CreateBusinessWithStandardKybInput,
    ) -> Result<CreateCustomerResponse> {
        self.create_business_with_standard_kyb_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create_business_with_standard_kyb`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers`
    pub async fn create_business_with_standard_kyb_with_options(
        &self,
        input: &CreateBusinessWithStandardKybInput,
        options: &RequestOptions,
    ) -> Result<CreateCustomerResponse> {
        self.create(
            Discriminated {
                account_type: "business",
                kyc_type: "standard",
                input,
            },
            options,
        )
        .await
    }

    async fn create<T: Serialize>(
        &self,
        body: Discriminated<'_, T>,
        options: &RequestOptions,
    ) -> Result<CreateCustomerResponse> {
        let path = format!("/instances/{}/customers", self.client.instance_id);
        self.client
            .request_with(Method::POST, &path, None::<&()>, Some(&body), options)
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
        self.update_with_options(customer_id, input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::update`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `PUT /instances/{instance_id}/customers/{id}`
    pub async fn update_with_options(
        &self,
        customer_id: impl AsRef<str>,
        input: &UpdateCustomerInput,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request_with(Method::PUT, &path, None::<&()>, Some(input), options)
            .await
    }

    /// Deletes a customer.
    ///
    /// `DELETE /instances/{instance_id}/customers/{id}`
    pub async fn delete(&self, customer_id: impl AsRef<str>) -> Result<Success> {
        self.delete_with_options(customer_id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::delete`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{instance_id}/customers/{id}`
    pub async fn delete_with_options(
        &self,
        customer_id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Success> {
        let path = format!(
            "/instances/{}/customers/{}",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request_with(Method::DELETE, &path, None::<&()>, None::<&()>, options)
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
        self.request_limit_increase_with_options(customer_id, input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::request_limit_increase`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/customers/{customer_id}/limit-increase`
    pub async fn request_limit_increase_with_options(
        &self,
        customer_id: impl AsRef<str>,
        input: &RequestLimitIncreaseInput,
        options: &RequestOptions,
    ) -> Result<RequestLimitIncreaseResponse> {
        let path = format!(
            "/instances/{}/customers/{}/limit-increase",
            self.client.instance_id,
            encode_path_segment(customer_id.as_ref())
        );
        self.client
            .request_with(Method::POST, &path, None::<&()>, Some(input), options)
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
