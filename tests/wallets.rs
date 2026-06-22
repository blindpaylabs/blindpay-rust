//! Integration tests for the `wallets` resources (blockchain, custodial, and
//! offramp), backed by a mock HTTP server.

use blindpay::{
    BlindPay, CreateBlockchainWalletWithAddressInput, CreateBlockchainWalletWithHashInput,
    CreateOfframpWalletInput, CreateWalletInput, Error, MintUsdbSolanaInput, MintUsdbStellarInput,
    Network, PrepareSolanaDelegationInput, Token,
};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn client(server: &MockServer) -> BlindPay {
    BlindPay::builder("test-api-key", "in_test")
        .base_url(format!("{}/v1", server.uri()))
        .build()
        .expect("client should build")
}

fn user_agent() -> String {
    format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"))
}

// ---------------------------------------------------------------------------
// blockchain wallets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn blockchain_list_parses_and_sends_auth_headers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets",
        ))
        .and(header("authorization", "Bearer test-api-key"))
        .and(header("user-agent", user_agent().as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "bw_000000000000",
                "name": "My Wallet",
                "network": "polygon",
                "address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
                "is_account_abstraction": true,
                "receiver_id": "re_000000000000"
            }
        ])))
        .mount(&server)
        .await;

    let wallets = client(&server)
        .await
        .wallets
        .blockchain
        .list("cus_123")
        .await
        .unwrap();

    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].id, "bw_000000000000");
    assert_eq!(wallets[0].network, Network::Polygon);
    assert!(wallets[0].is_account_abstraction);
    assert_eq!(wallets[0].receiver_id, "re_000000000000");
    assert!(wallets[0].signature_tx_hash.is_none());
}

#[tokio::test]
async fn blockchain_get_wallet_message_parses_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets/sign-message",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "message": "sign this nonce" })),
        )
        .mount(&server)
        .await;

    let msg = client(&server)
        .await
        .wallets
        .blockchain
        .get_wallet_message("cus_123")
        .await
        .unwrap();

    assert_eq!(msg.message, "sign this nonce");
}

#[tokio::test]
async fn blockchain_get_parses_single_wallet() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets/bw_000000000000",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "bw_000000000000",
            "name": "Sig Wallet",
            "network": "base",
            "signature_tx_hash": "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359",
            "is_account_abstraction": false,
            "receiver_id": "re_000000000000"
        })))
        .mount(&server)
        .await;

    let wallet = client(&server)
        .await
        .wallets
        .blockchain
        .get("cus_123", "bw_000000000000")
        .await
        .unwrap();

    assert_eq!(wallet.network, Network::Base);
    assert!(!wallet.is_account_abstraction);
    assert_eq!(
        wallet.signature_tx_hash.as_deref(),
        Some("0x3c499c542cef5e3811e1192ce70d8cc03d5c3359")
    );
    assert!(wallet.address.is_none());
}

#[tokio::test]
async fn blockchain_create_with_address_sends_true_discriminator() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets",
        ))
        .and(body_json(serde_json::json!({
            "name": "My Wallet",
            "network": "polygon",
            "address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
            "is_account_abstraction": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "bw_000000000000",
            "name": "My Wallet",
            "network": "polygon",
            "address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
            "is_account_abstraction": true,
            "receiver_id": "re_000000000000"
        })))
        .mount(&server)
        .await;

    let input = CreateBlockchainWalletWithAddressInput {
        name: "My Wallet".to_string(),
        network: Network::Polygon,
        address: "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C".to_string(),
        is_account_abstraction: true,
    };
    let wallet = client(&server)
        .await
        .wallets
        .blockchain
        .create_with_address("cus_123", &input)
        .await
        .unwrap();

    assert_eq!(wallet.id, "bw_000000000000");
    assert!(wallet.is_account_abstraction);
}

#[tokio::test]
async fn blockchain_create_with_hash_sends_false_discriminator() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets",
        ))
        .and(body_json(serde_json::json!({
            "name": "Sig Wallet",
            "network": "base",
            "signature_tx_hash": "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359",
            "is_account_abstraction": false
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "bw_000000000001",
            "name": "Sig Wallet",
            "network": "base",
            "signature_tx_hash": "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359",
            "is_account_abstraction": false,
            "receiver_id": "re_000000000000"
        })))
        .mount(&server)
        .await;

    let input = CreateBlockchainWalletWithHashInput {
        name: "Sig Wallet".to_string(),
        network: Network::Base,
        signature_tx_hash: "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359".to_string(),
        is_account_abstraction: false,
    };
    let wallet = client(&server)
        .await
        .wallets
        .blockchain
        .create_with_hash("cus_123", &input)
        .await
        .unwrap();

    assert_eq!(wallet.id, "bw_000000000001");
    assert!(!wallet.is_account_abstraction);
}

#[tokio::test]
async fn blockchain_delete_returns_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets/bw_000000000000",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .wallets
        .blockchain
        .delete("cus_123", "bw_000000000000")
        .await
        .unwrap();

    assert!(res.success);
}

#[tokio::test]
async fn blockchain_list_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/blockchain-wallets",
        ))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({ "message": "not found" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .wallets
        .blockchain
        .list("cus_123")
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

// ---------------------------------------------------------------------------
// custodial wallets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn custodial_list_parses_wallets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/instances/in_test/customers/cus_123/wallets"))
        .and(header("authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "bl_000000000000",
                "name": "Blindpay Wallet",
                "external_id": "your-database-id",
                "address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C",
                "network": "polygon",
                "created_at": "2026-01-01T00:00:00.000Z"
            }
        ])))
        .mount(&server)
        .await;

    let wallets = client(&server)
        .await
        .wallets
        .custodial
        .list("cus_123")
        .await
        .unwrap();

    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].id, "bl_000000000000");
    assert_eq!(wallets[0].external_id.as_deref(), Some("your-database-id"));
    assert_eq!(wallets[0].network, Network::Polygon);
}

#[tokio::test]
async fn custodial_get_parses_wallet_with_null_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/wallets/bl_000000000000",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "bl_000000000000",
            "name": "Blindpay Wallet",
            "external_id": null,
            "address": null,
            "network": "base",
            "created_at": "2026-01-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let wallet = client(&server)
        .await
        .wallets
        .custodial
        .get("cus_123", "bl_000000000000")
        .await
        .unwrap();

    assert!(wallet.external_id.is_none());
    assert!(wallet.address.is_none());
    assert_eq!(wallet.network, Network::Base);
}

#[tokio::test]
async fn custodial_get_balance_parses_token_map() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/wallets/bl_000000000000/balance",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "USDC": { "address": "0xabc", "id": "tok_1", "symbol": "USDC", "amount": 1000 },
            "USDT": { "address": "0xdef", "id": "tok_2", "symbol": "USDT", "amount": 0 },
            "USDB": { "address": "0xghi", "id": "tok_3", "symbol": "USDB", "amount": 25.5 }
        })))
        .mount(&server)
        .await;

    let balance = client(&server)
        .await
        .wallets
        .custodial
        .get_balance("cus_123", "bl_000000000000")
        .await
        .unwrap();

    assert_eq!(balance.usdc.symbol, Token::Usdc);
    assert_eq!(balance.usdc.amount, 1000.0);
    assert_eq!(balance.usdt.amount, 0.0);
    assert_eq!(balance.usdb.amount, 25.5);
    assert_eq!(balance.usdb.id, "tok_3");
}

#[tokio::test]
async fn custodial_create_sends_body_and_parses_wallet() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/cus_123/wallets"))
        .and(body_json(serde_json::json!({
            "network": "polygon",
            "name": "My Custodial Wallet",
            "external_id": "ext_123"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "bl_000000000000",
            "name": "My Custodial Wallet",
            "external_id": "ext_123",
            "address": null,
            "network": "polygon",
            "created_at": "2026-01-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let input = CreateWalletInput {
        network: Network::Polygon,
        name: "My Custodial Wallet".to_string(),
        external_id: Some("ext_123".to_string()),
    };
    let wallet = client(&server)
        .await
        .wallets
        .custodial
        .create("cus_123", &input)
        .await
        .unwrap();

    assert_eq!(wallet.id, "bl_000000000000");
    assert_eq!(wallet.external_id.as_deref(), Some("ext_123"));
}

#[tokio::test]
async fn custodial_create_omits_unset_external_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/customers/cus_123/wallets"))
        .and(body_json(serde_json::json!({
            "network": "base",
            "name": "No External"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "bl_000000000001",
            "name": "No External",
            "external_id": null,
            "address": null,
            "network": "base",
            "created_at": "2026-01-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let input = CreateWalletInput {
        network: Network::Base,
        name: "No External".to_string(),
        external_id: None,
    };
    let wallet = client(&server)
        .await
        .wallets
        .custodial
        .create("cus_123", &input)
        .await
        .unwrap();

    assert_eq!(wallet.id, "bl_000000000001");
}

#[tokio::test]
async fn custodial_delete_returns_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/wallets/bl_000000000000",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .wallets
        .custodial
        .delete("cus_123", "bl_000000000000")
        .await
        .unwrap();

    assert!(res.success);
}

#[tokio::test]
async fn custodial_get_balance_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/wallets/bl_000000000000/balance",
        ))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({ "message": "balance unavailable" })),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .wallets
        .custodial
        .get_balance("cus_123", "bl_000000000000")
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 500);
            assert_eq!(api.message, "balance unavailable");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// offramp wallets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn offramp_list_parses_full_wallets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/bank-accounts/ba_123/offramp-wallets",
        ))
        .and(header("authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "ow_000000000000",
                "external_id": "your_external_id",
                "instance_id": "in_test",
                "receiver_id": "re_000000000000",
                "bank_account_id": "ba_123",
                "circle_wallet_id": "01234567-abcd-efgh-ijkl-012345678901",
                "network": "tron",
                "address": "TALJN9zTTEL9TVBb4WuTt6wLvPqJZr3hvb",
                "created_at": "2026-01-01T00:00:00.000Z",
                "updated_at": "2026-01-02T00:00:00.000Z"
            }
        ])))
        .mount(&server)
        .await;

    let wallets = client(&server)
        .await
        .wallets
        .offramp
        .list("cus_123", "ba_123")
        .await
        .unwrap();

    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].id, "ow_000000000000");
    assert_eq!(wallets[0].network, Network::Tron);
    assert_eq!(wallets[0].bank_account_id.as_deref(), Some("ba_123"));
    assert_eq!(wallets[0].receiver_id.as_deref(), Some("re_000000000000"));
}

#[tokio::test]
async fn offramp_create_sends_body_and_parses_create_shape() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/bank-accounts/ba_123/offramp-wallets",
        ))
        .and(body_json(serde_json::json!({
            "external_id": "ext_456",
            "network": "base"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ow_000000000001",
            "external_id": "ext_456",
            "circle_wallet_id": null,
            "network": "base",
            "address": "0xDD6a3aD0949396e57C7738ba8FC1A46A5a1C372C"
        })))
        .mount(&server)
        .await;

    let input = CreateOfframpWalletInput {
        external_id: Some("ext_456".to_string()),
        network: Network::Base,
    };
    let wallet = client(&server)
        .await
        .wallets
        .offramp
        .create("cus_123", "ba_123", &input)
        .await
        .unwrap();

    assert_eq!(wallet.id, "ow_000000000001");
    assert_eq!(wallet.network, Network::Base);
    // The create response omits these fields, so they default to None.
    assert!(wallet.bank_account_id.is_none());
    assert!(wallet.created_at.is_none());
}

#[tokio::test]
async fn offramp_get_parses_wallet() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/bank-accounts/ba_123/offramp-wallets/ow_000000000000",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ow_000000000000",
            "external_id": null,
            "instance_id": "in_test",
            "receiver_id": "re_000000000000",
            "bank_account_id": "ba_123",
            "circle_wallet_id": null,
            "network": "solana",
            "address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            "created_at": "2026-01-01T00:00:00.000Z",
            "updated_at": "2026-01-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let wallet = client(&server)
        .await
        .wallets
        .offramp
        .get("cus_123", "ba_123", "ow_000000000000")
        .await
        .unwrap();

    assert_eq!(wallet.id, "ow_000000000000");
    assert_eq!(wallet.network, Network::Solana);
    assert!(wallet.external_id.is_none());
}

#[tokio::test]
async fn offramp_create_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/instances/in_test/customers/cus_123/bank-accounts/ba_123/offramp-wallets",
        ))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(serde_json::json!({ "message": "unsupported network" })),
        )
        .mount(&server)
        .await;

    let input = CreateOfframpWalletInput {
        external_id: None,
        network: Network::Stellar,
    };
    let err = client(&server)
        .await
        .wallets
        .offramp
        .create("cus_123", "ba_123", &input)
        .await
        .unwrap_err();

    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "unsupported network");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// blockchain chain ops (Stellar trustline, USDB mint, Solana delegation)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_asset_trustline_sends_address_and_parses_xdr() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/create-asset-trustline"))
        .and(header("authorization", "Bearer test-api-key"))
        .and(body_json(serde_json::json!({ "address": "GABC123" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "xdr": "AAAAencoded"
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .wallets
        .blockchain
        .create_asset_trustline("GABC123")
        .await
        .unwrap();
    assert_eq!(res.xdr, "AAAAencoded");
}

#[tokio::test]
async fn mint_usdb_stellar_omits_unset_signed_xdr() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/mint-usdb-stellar"))
        .and(body_json(serde_json::json!({
            "address": "GABC123",
            "amount": "100"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .wallets
        .blockchain
        .mint_usdb_stellar(&MintUsdbStellarInput {
            address: "GABC123".to_string(),
            amount: "100".to_string(),
            signed_xdr: None,
        })
        .await
        .unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn mint_usdb_stellar_serializes_signed_xdr_as_camel_case() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/mint-usdb-stellar"))
        .and(body_json(serde_json::json!({
            "address": "GABC123",
            "amount": "100",
            "signedXdr": "signed"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "success": true })),
        )
        .mount(&server)
        .await;

    client(&server)
        .await
        .wallets
        .blockchain
        .mint_usdb_stellar(&MintUsdbStellarInput {
            address: "GABC123".to_string(),
            amount: "100".to_string(),
            signed_xdr: Some("signed".to_string()),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn mint_usdb_solana_parses_signature() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/mint-usdb-solana"))
        .and(body_json(
            serde_json::json!({ "address": "So1ABC", "amount": "100" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "signature": "sig_abc"
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .wallets
        .blockchain
        .mint_usdb_solana(&MintUsdbSolanaInput {
            address: "So1ABC".to_string(),
            amount: "100".to_string(),
        })
        .await
        .unwrap();
    assert!(res.success);
    assert_eq!(res.signature.as_deref(), Some("sig_abc"));
}

#[tokio::test]
async fn prepare_solana_delegation_omits_unset_fields_and_parses_transaction() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/prepare-delegate-solana"))
        .and(body_json(serde_json::json!({
            "owner_address": "So1ABC",
            "quote_id": "qt_123456789012"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "transaction": "base64tx"
        })))
        .mount(&server)
        .await;

    let res = client(&server)
        .await
        .wallets
        .blockchain
        .prepare_solana_delegation_transaction(&PrepareSolanaDelegationInput {
            owner_address: "So1ABC".to_string(),
            quote_id: Some("qt_123456789012".to_string()),
            token_address: None,
            amount: None,
        })
        .await
        .unwrap();
    assert!(res.success);
    assert_eq!(res.transaction.as_deref(), Some("base64tx"));
}

#[tokio::test]
async fn create_asset_trustline_non_success_returns_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/instances/in_test/create-asset-trustline"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(
                serde_json::json!({ "message": "blockchain_wallet_address_missing" }),
            ),
        )
        .mount(&server)
        .await;

    let err = client(&server)
        .await
        .wallets
        .blockchain
        .create_asset_trustline("GABC123")
        .await
        .unwrap_err();
    match err {
        Error::Api(api) => {
            assert_eq!(api.status, 400);
            assert_eq!(api.message, "blockchain_wallet_address_missing");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
