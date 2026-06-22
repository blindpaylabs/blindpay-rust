//! Integration tests for the `customers` resource (and its `bank_accounts`
//! sub-resource), backed by a mock HTTP server.

use blindpay::{
    AccountClass, BankAccountStatus, BlindPay, CreateAchInput, CreateBusinessWithStandardKybInput,
    CreateIndividualWithStandardKycInput, CreateInternationalSwiftInput, CreatePixInput, Error,
    KycStatus, KycType, ListBankAccountsParams, ListCustomersParams, Rail, RecipientRelationship,
    RequestLimitIncreaseInput, SupportingDocumentType, UpdateCustomerInput,
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Builds a client pointed at the mock server. The mock's `/v1` prefix mirrors
/// the production base URL's path.
async fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test-api-key", "in_test")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .expect("client should build")
}

#[tokio::test]
async fn list_customers_parses_paginated_response_and_sends_auth_headers() {
    let server = MockServer::start().await;
    let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers"))
        .and(query_param("limit", "50"))
        .and(query_param("country", "BR"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "re_123",
                    "type": "individual",
                    "kyc_type": "standard",
                    "kyc_status": "verifying",
                    "email": "ana@example.com",
                    "country": "BR",
                    "first_name": "Ana",
                    "last_name": "Silva",
                    "limit": { "per_transaction": 100000, "daily": 200000, "monthly": 1000000 }
                }
            ],
            "pagination": { "has_more": false, "next_page": null, "prev_page": null }
        })))
        .mount(&server)
        .await;

    let params = ListCustomersParams::new()
        .limit(blindpay::Limit::Fifty)
        .country("BR");
    let resp = client(&server).await.customers.list(&params).await.unwrap();

    assert_eq!(resp.data().len(), 1);
    let customer = &resp.data()[0];
    assert_eq!(customer.id, "re_123");
    assert_eq!(customer.type_, AccountClass::Individual);
    assert_eq!(customer.kyc_type, KycType::Standard);
    assert_eq!(customer.kyc_status, KycStatus::Verifying);
    assert_eq!(customer.first_name.as_deref(), Some("Ana"));
    assert_eq!(customer.limit.as_ref().unwrap().monthly, 1000000);
    assert!(resp.pagination().is_some());
    assert!(!resp.pagination().unwrap().has_more);
}

#[tokio::test]
async fn list_customers_parses_bare_array_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "re_123",
                "type": "business",
                "kyc_type": "standard",
                "kyc_status": "approved",
                "legal_name": "Acme LLC"
            }
        ])))
        .mount(&server)
        .await;

    let resp = client(&server)
        .await
        .customers
        .list(&ListCustomersParams::new())
        .await
        .unwrap();

    assert_eq!(resp.data().len(), 1);
    assert_eq!(resp.data()[0].type_, AccountClass::Business);
    assert_eq!(resp.data()[0].legal_name.as_deref(), Some("Acme LLC"));
    assert!(resp.pagination().is_none());
}

#[tokio::test]
async fn create_individual_with_standard_kyc_injects_discriminators() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers"))
        .and(body_json(serde_json::json!({
            "type": "individual",
            "kyc_type": "standard",
            "country": "BR",
            "email": "ana@example.com",
            "first_name": "Ana"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "re_123",
            "customer_id": "re_123"
        })))
        .mount(&server)
        .await;

    let input = CreateIndividualWithStandardKycInput {
        country: blindpay::Country::from("BR"),
        email: "ana@example.com".to_string(),
        first_name: Some("Ana".to_string()),
        ..Default::default()
    };
    let resp = client(&server)
        .await
        .customers
        .create_individual_with_standard_kyc(&input)
        .await
        .unwrap();

    assert_eq!(resp.id, "re_123");
    assert_eq!(resp.customer_id, "re_123");
}

#[tokio::test]
async fn create_business_with_standard_kyb_injects_business_type() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers"))
        .and(body_json(serde_json::json!({
            "type": "business",
            "kyc_type": "standard",
            "country": "US",
            "email": "ops@acme.com",
            "legal_name": "Acme LLC"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "re_456",
            "customer_id": "re_456"
        })))
        .mount(&server)
        .await;

    let input = CreateBusinessWithStandardKybInput {
        country: blindpay::Country::from("US"),
        email: "ops@acme.com".to_string(),
        legal_name: Some("Acme LLC".to_string()),
        ..Default::default()
    };
    let resp = client(&server)
        .await
        .customers
        .create_business_with_standard_kyb(&input)
        .await
        .unwrap();

    assert_eq!(resp.id, "re_456");
}

#[tokio::test]
async fn get_customer_parses_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers/re_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "re_123",
            "type": "individual",
            "kyc_type": "enhanced",
            "kyc_status": "approved",
            "email": "ana@example.com",
            "aml_status": "clear",
            "aml_hits": {
                "has_sanction_match": false,
                "has_pep_match": false,
                "has_watchlist_match": false,
                "has_crimelist_match": false,
                "has_adversemedia_match": false
            }
        })))
        .mount(&server)
        .await;

    let customer = client(&server).await.customers.get("re_123").await.unwrap();
    assert_eq!(customer.kyc_type, KycType::Enhanced);
    assert_eq!(customer.kyc_status, KycStatus::Approved);
    assert_eq!(customer.aml_status.as_deref(), Some("clear"));
    assert!(!customer.aml_hits.as_ref().unwrap().has_pep_match);
}

#[tokio::test]
async fn update_customer_sends_put_and_returns_success() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/instances/in_test/customers/re_123"))
        .and(body_json(serde_json::json!({ "email": "new@example.com" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let input = UpdateCustomerInput {
        email: Some("new@example.com".to_string()),
        ..Default::default()
    };
    let resp = client(&server)
        .await
        .customers
        .update("re_123", &input)
        .await
        .unwrap();
    assert!(resp.success);
}

#[tokio::test]
async fn delete_customer_returns_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/instances/in_test/customers/re_123"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let resp = client(&server)
        .await
        .customers
        .delete("re_123")
        .await
        .unwrap();
    assert!(resp.success);
}

#[tokio::test]
async fn get_limits_parses_payin_and_payout() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/limits/customers/re_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "limits": {
                "payin": { "daily": 10000, "monthly": 50000 },
                "payout": { "daily": 20000, "monthly": 100000 }
            }
        })))
        .mount(&server)
        .await;

    let limits = client(&server)
        .await
        .customers
        .get_limits("re_123")
        .await
        .unwrap();
    assert_eq!(limits.limits.payin.daily, 10000);
    assert_eq!(limits.limits.payout.monthly, 100000);
}

#[tokio::test]
async fn get_limit_increase_requests_parses_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/re_123/limit-increase",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "rl_123",
                "receiver_id": "re_123",
                "status": "in_review",
                "daily": 50000,
                "monthly": 250000,
                "per_transaction": 25000,
                "supporting_document_file": "https://example.com/doc.pdf",
                "supporting_document_type": "individual_bank_statement"
            }
        ])))
        .mount(&server)
        .await;

    let requests = client(&server)
        .await
        .customers
        .get_limit_increase_requests("re_123")
        .await
        .unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].id, "rl_123");
    assert_eq!(requests[0].receiver_id, "re_123");
    assert_eq!(requests[0].status, blindpay::LimitIncreaseStatus::InReview);
    assert_eq!(
        requests[0].supporting_document_type,
        Some(SupportingDocumentType::IndividualBankStatement)
    );
}

#[tokio::test]
async fn request_limit_increase_sends_body_and_parses_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/re_123/limit-increase",
        ))
        .and(body_json(serde_json::json!({
            "per_transaction": 50000,
            "daily": 100000,
            "monthly": 500000,
            "supporting_document_type": "individual_tax_return",
            "supporting_document_file": "https://example.com/tax.pdf"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": "rl_456" })),
        )
        .mount(&server)
        .await;

    let input = RequestLimitIncreaseInput {
        per_transaction: 50000,
        daily: 100000,
        monthly: 500000,
        supporting_document_type: SupportingDocumentType::IndividualTaxReturn,
        supporting_document_file: "https://example.com/tax.pdf".to_string(),
    };
    let resp = client(&server)
        .await
        .customers
        .request_limit_increase("re_123", &input)
        .await
        .unwrap();
    assert_eq!(resp.id, "rl_456");
}

#[tokio::test]
async fn list_customers_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers"))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(serde_json::json!({ "message": "forbidden" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .customers
        .list(&ListCustomersParams::new())
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 403);
            assert_eq!(api.message, "forbidden");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

// --- bank_accounts sub-resource ---

#[tokio::test]
async fn list_bank_accounts_sends_filters_and_parses_offramp_wallets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(query_param("type", "pix"))
        .and(query_param("status", "approved"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "ba_123",
                "type": "pix",
                "name": "My PIX",
                "pix_key": "11122233344",
                "status": "approved",
                "offramp_wallets": [
                    {
                        "id": "ow_123",
                        "external_id": "ext_1",
                        "network": "tron",
                        "address": "TALJN9zTTEL9TVBb4WuTt6wLvPqJZr3hvb"
                    }
                ]
            }
        ])))
        .mount(&server)
        .await;

    let params = ListBankAccountsParams::new()
        .rail(Rail::Pix)
        .status(BankAccountStatus::Approved);
    let accounts = client(&server)
        .await
        .customers
        .bank_accounts
        .list("re_123", &params)
        .await
        .unwrap();

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "ba_123");
    assert_eq!(accounts[0].type_, Rail::Pix);
    assert_eq!(accounts[0].pix_key.as_deref(), Some("11122233344"));
    let wallets = accounts[0].offramp_wallets.as_ref().unwrap();
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].id, "ow_123");
}

#[tokio::test]
async fn list_bank_accounts_decodes_null_offramp_wallets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "ba_123",
                "type": "pix",
                "name": "My PIX",
                "offramp_wallets": null
            }
        ])))
        .mount(&server)
        .await;

    let accounts = client(&server)
        .await
        .customers
        .bank_accounts
        .list("re_123", &ListBankAccountsParams::new())
        .await
        .unwrap();

    assert_eq!(accounts.len(), 1);
    assert!(accounts[0].offramp_wallets.is_none());
}

#[tokio::test]
async fn create_pix_bank_account_injects_type_discriminator() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "pix",
            "name": "My PIX",
            "pix_key": "11122233344"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_123",
            "type": "pix",
            "name": "My PIX",
            "pix_key": "11122233344"
        })))
        .mount(&server)
        .await;

    let input = CreatePixInput {
        name: "My PIX".to_string(),
        pix_key: "11122233344".to_string(),
        force_cpf_cnpj: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_pix("re_123", &input)
        .await
        .unwrap();

    assert_eq!(account.id, "ba_123");
    assert_eq!(account.type_, Rail::Pix);
}

#[tokio::test]
async fn create_ach_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "ach",
            "name": "US Checking",
            "account_class": "individual",
            "account_number": "1001001234",
            "account_type": "checking",
            "beneficiary_name": "Ana Silva",
            "routing_number": "012345678",
            "recipient_relationship": "first_party",
            "address_line_1": "1 Main St",
            "city": "New York",
            "state_province_region": "NY",
            "country": "US",
            "postal_code": "10001"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_456",
            "type": "ach",
            "name": "US Checking",
            "account_class": "individual",
            "recipient_relationship": "first_party"
        })))
        .mount(&server)
        .await;

    let input = CreateAchInput {
        name: "US Checking".to_string(),
        account_class: AccountClass::Individual,
        account_number: "1001001234".to_string(),
        account_type: blindpay::AccountType::Checking,
        beneficiary_name: "Ana Silva".to_string(),
        routing_number: "012345678".to_string(),
        recipient_relationship: RecipientRelationship::FirstParty,
        address_line_1: "1 Main St".to_string(),
        city: "New York".to_string(),
        state_province_region: "NY".to_string(),
        country: blindpay::Country::from("US"),
        postal_code: "10001".to_string(),
        address_line_2: None,
        business_industry: None,
        phone_number: None,
        tax_id: None,
        date_of_birth: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_ach("re_123", &input)
        .await
        .unwrap();

    assert_eq!(account.id, "ba_456");
    assert_eq!(account.type_, Rail::Ach);
    assert_eq!(
        account.recipient_relationship,
        Some(RecipientRelationship::FirstParty)
    );
}

#[tokio::test]
async fn create_pix_safe_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "pix_safe",
            "name": "PIX Safe",
            "beneficiary_name": "Ana Silva",
            "account_number": "12345",
            "account_type": "checking",
            "pix_safe_bank_code": "001",
            "pix_safe_branch_code": "0001",
            "pix_safe_cpf_cnpj": "11122233344"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_ps",
            "type": "pix_safe",
            "name": "PIX Safe"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreatePixSafeInput {
        name: "PIX Safe".to_string(),
        beneficiary_name: "Ana Silva".to_string(),
        account_number: "12345".to_string(),
        account_type: blindpay::AccountType::Checking,
        pix_safe_bank_code: "001".to_string(),
        pix_safe_branch_code: "0001".to_string(),
        pix_safe_cpf_cnpj: "11122233344".to_string(),
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_pix_safe("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_ps");
}

#[tokio::test]
async fn create_ted_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "ted",
            "name": "TED",
            "beneficiary_name": "Ana Silva",
            "account_number": "12345",
            "account_type": "saving",
            "ted_bank_code": "341",
            "ted_branch_code": "0001",
            "ted_cpf_cnpj": "11122233344"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_ted",
            "type": "ted",
            "name": "TED"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateTedInput {
        name: "TED".to_string(),
        beneficiary_name: "Ana Silva".to_string(),
        account_number: "12345".to_string(),
        account_type: blindpay::AccountType::Saving,
        ted_bank_code: "341".to_string(),
        ted_branch_code: "0001".to_string(),
        ted_cpf_cnpj: "11122233344".to_string(),
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_ted("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_ted");
}

#[tokio::test]
async fn create_spei_bitso_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "spei_bitso",
            "name": "SPEI",
            "beneficiary_name": "Juan Perez",
            "spei_protocol": "clabe",
            "spei_clabe": "012345678901234567"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_spei",
            "type": "spei_bitso",
            "name": "SPEI"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateSpeiBitsoInput {
        name: "SPEI".to_string(),
        beneficiary_name: "Juan Perez".to_string(),
        spei_protocol: blindpay::SpeiProtocol::Clabe,
        spei_clabe: "012345678901234567".to_string(),
        spei_institution_code: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_spei_bitso("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_spei");
}

#[tokio::test]
async fn create_transfers_bitso_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "transfers_bitso",
            "name": "Transfers",
            "beneficiary_name": "Juan Perez",
            "transfers_type": "CVU",
            "transfers_account": "0000000000000000000000"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_tr",
            "type": "transfers_bitso",
            "name": "Transfers"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateTransfersBitsoInput {
        name: "Transfers".to_string(),
        beneficiary_name: "Juan Perez".to_string(),
        transfers_type: blindpay::TransfersType::Cvu,
        transfers_account: "0000000000000000000000".to_string(),
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_transfers_bitso("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_tr");
}

#[tokio::test]
async fn create_ach_cop_bitso_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "ach_cop_bitso",
            "name": "ACH COP",
            "account_type": "checking",
            "ach_cop_beneficiary_first_name": "Juan",
            "ach_cop_beneficiary_last_name": "Perez",
            "ach_cop_document_id": "1234567890",
            "ach_cop_document_type": "CC",
            "ach_cop_email": "juan@example.com",
            "ach_cop_bank_code": "1007",
            "ach_cop_bank_account": "12345678"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_cop",
            "type": "ach_cop_bitso",
            "name": "ACH COP"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateAchCopBitsoInput {
        name: "ACH COP".to_string(),
        account_type: blindpay::AccountType::Checking,
        ach_cop_beneficiary_first_name: "Juan".to_string(),
        ach_cop_beneficiary_last_name: "Perez".to_string(),
        ach_cop_document_id: "1234567890".to_string(),
        ach_cop_document_type: blindpay::AchCopDocument::Cc,
        ach_cop_email: "juan@example.com".to_string(),
        ach_cop_bank_code: "1007".to_string(),
        ach_cop_bank_account: "12345678".to_string(),
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_ach_cop_bitso("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_cop");
}

#[tokio::test]
async fn create_wire_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "wire",
            "name": "US Wire",
            "account_class": "individual",
            "account_number": "1001001234",
            "beneficiary_name": "Ana Silva",
            "routing_number": "012345678",
            "recipient_relationship": "first_party",
            "address_line_1": "1 Main St",
            "city": "New York",
            "state_province_region": "NY",
            "country": "US",
            "postal_code": "10001"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_wire",
            "type": "wire",
            "name": "US Wire"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateWireInput {
        name: "US Wire".to_string(),
        account_class: AccountClass::Individual,
        account_number: "1001001234".to_string(),
        beneficiary_name: "Ana Silva".to_string(),
        routing_number: "012345678".to_string(),
        recipient_relationship: RecipientRelationship::FirstParty,
        address_line_1: "1 Main St".to_string(),
        city: "New York".to_string(),
        state_province_region: "NY".to_string(),
        country: blindpay::Country::from("US"),
        postal_code: "10001".to_string(),
        address_line_2: None,
        business_industry: None,
        phone_number: None,
        tax_id: None,
        date_of_birth: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_wire("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.type_, Rail::Wire);
}

#[tokio::test]
async fn create_rtp_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "rtp",
            "name": "RTP",
            "account_class": "individual",
            "beneficiary_name": "Ana Silva",
            "routing_number": "012345678",
            "account_number": "1001001234",
            "recipient_relationship": "first_party",
            "address_line_1": "1 Main St",
            "city": "New York",
            "state_province_region": "NY",
            "country": "US",
            "postal_code": "10001"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_rtp",
            "type": "rtp",
            "name": "RTP"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateRtpInput {
        name: "RTP".to_string(),
        account_class: AccountClass::Individual,
        beneficiary_name: "Ana Silva".to_string(),
        routing_number: "012345678".to_string(),
        account_number: "1001001234".to_string(),
        recipient_relationship: RecipientRelationship::FirstParty,
        address_line_1: "1 Main St".to_string(),
        city: "New York".to_string(),
        state_province_region: "NY".to_string(),
        country: blindpay::Country::from("US"),
        postal_code: "10001".to_string(),
        address_line_2: None,
        business_industry: None,
        phone_number: None,
        tax_id: None,
        date_of_birth: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_rtp("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.type_, Rail::Rtp);
}

#[tokio::test]
async fn create_international_swift_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "international_swift",
            "name": "SWIFT",
            "account_class": "individual",
            "recipient_relationship": "first_party",
            "swift_account_holder_name": "Ana Silva",
            "swift_account_number_iban": "DE89370400440532013000",
            "swift_code_bic": "DEUTDEFF",
            "swift_beneficiary_address_line_1": "1 Main St",
            "swift_beneficiary_city": "Berlin",
            "swift_beneficiary_country": "DE",
            "swift_beneficiary_postal_code": "10115",
            "swift_beneficiary_state_province_region": "BE",
            "swift_bank_name": "Deutsche Bank",
            "swift_bank_address_line_1": "Bank St",
            "swift_bank_city": "Frankfurt",
            "swift_bank_country": "DE",
            "swift_bank_postal_code": "60311",
            "swift_bank_state_province_region": "HE"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_swift",
            "type": "international_swift",
            "name": "SWIFT"
        })))
        .mount(&server)
        .await;

    let input = CreateInternationalSwiftInput {
        name: "SWIFT".to_string(),
        account_class: AccountClass::Individual,
        recipient_relationship: RecipientRelationship::FirstParty,
        swift_account_holder_name: "Ana Silva".to_string(),
        swift_account_number_iban: "DE89370400440532013000".to_string(),
        swift_code_bic: "DEUTDEFF".to_string(),
        swift_beneficiary_address_line_1: "1 Main St".to_string(),
        swift_beneficiary_city: "Berlin".to_string(),
        swift_beneficiary_country: blindpay::Country::from("DE"),
        swift_beneficiary_postal_code: "10115".to_string(),
        swift_beneficiary_state_province_region: "BE".to_string(),
        swift_bank_name: "Deutsche Bank".to_string(),
        swift_bank_address_line_1: "Bank St".to_string(),
        swift_bank_city: "Frankfurt".to_string(),
        swift_bank_country: blindpay::Country::from("DE"),
        swift_bank_postal_code: "60311".to_string(),
        swift_bank_state_province_region: "HE".to_string(),
        swift_beneficiary_address_line_2: None,
        swift_bank_address_line_2: None,
        swift_intermediary_bank_swift_code_bic: None,
        swift_intermediary_bank_account_number_iban: None,
        swift_intermediary_bank_name: None,
        swift_intermediary_bank_country: None,
        swift_payment_code: None,
        swift_ifsc_branch_code: None,
        business_industry: None,
        phone_number: None,
        tax_id: None,
        date_of_birth: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_international_swift("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_swift");
}

#[tokio::test]
async fn create_sepa_bank_account_sends_typed_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .and(body_json(serde_json::json!({
            "type": "sepa",
            "name": "SEPA",
            "account_class": "individual",
            "sepa_iban": "DE89370400440532013000",
            "sepa_beneficiary_bic": "DEUTDEFF",
            "sepa_beneficiary_legal_name": "Ana Silva",
            "sepa_beneficiary_address_line_1": "1 Main St",
            "sepa_beneficiary_city": "Berlin",
            "sepa_beneficiary_postal_code": "10115",
            "sepa_beneficiary_country": "DE"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_sepa",
            "type": "sepa",
            "name": "SEPA"
        })))
        .mount(&server)
        .await;

    let input = blindpay::CreateSepaInput {
        name: "SEPA".to_string(),
        account_class: AccountClass::Individual,
        sepa_iban: "DE89370400440532013000".to_string(),
        sepa_beneficiary_bic: "DEUTDEFF".to_string(),
        sepa_beneficiary_legal_name: "Ana Silva".to_string(),
        sepa_beneficiary_address_line_1: "1 Main St".to_string(),
        sepa_beneficiary_city: "Berlin".to_string(),
        sepa_beneficiary_postal_code: "10115".to_string(),
        sepa_beneficiary_country: blindpay::Country::from("DE"),
        sepa_beneficiary_address_line_2: None,
        sepa_beneficiary_state_province_region: None,
    };
    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .create_sepa("re_123", &input)
        .await
        .unwrap();
    assert_eq!(account.id, "ba_sepa");
}

#[tokio::test]
async fn get_bank_account_parses_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/re_123/bank-accounts/ba_123",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ba_123",
            "type": "wire",
            "name": "Wire account"
        })))
        .mount(&server)
        .await;

    let account = client(&server)
        .await
        .customers
        .bank_accounts
        .get("re_123", "ba_123")
        .await
        .unwrap();
    assert_eq!(account.type_, Rail::Wire);
}

#[tokio::test]
async fn delete_bank_account_returns_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/v1/instances/in_test/customers/re_123/bank-accounts/ba_123",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let resp = client(&server)
        .await
        .customers
        .bank_accounts
        .delete("re_123", "ba_123")
        .await
        .unwrap();
    assert!(resp.success);
}

#[tokio::test]
async fn bank_accounts_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers/re_123/bank-accounts"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({ "message": "not found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .customers
        .bank_accounts
        .list("re_123", &ListBankAccountsParams::new())
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 404);
            assert_eq!(api.message, "not found");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
