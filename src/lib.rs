#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod client;
pub mod common;
mod error;
mod http;
mod internal;
pub mod resources;
pub mod webhooks;

pub use client::{BlindPay, BlindPayBuilder, DEFAULT_BASE_URL, DEFAULT_TIMEOUT};
pub use common::{
    AccountClass, AccountType, Country, Currency, CurrencyType, KycStatus, Limit, ListResponse,
    Network, Offset, Pagination, PaginationParams, PaymentMethod, Rail, Success, Token,
    TransactionStatus, WebhookEvent,
};
pub use error::{ApiError, Error, Result};
pub use resources::available::{
    Available, BankDetail, BankDetailItem, BankDetailKey, NaicsCode, RailEntry, RequiredWhen,
    RequiredWhenOperator, SwiftCodeBankDetail,
};
pub use resources::customers::{
    AchCopDocument, AmlHits, BankAccount, BankAccountStatus, BankAccounts, BusinessType,
    CreateAchCopBitsoInput, CreateAchInput, CreateBusinessWithStandardKybInput,
    CreateCustomerResponse, CreateIndividualWithEnhancedKycInput,
    CreateIndividualWithStandardKycInput, CreateInternationalSwiftInput, CreatePixInput,
    CreatePixSafeInput, CreateRtpInput, CreateSepaInput, CreateSpeiBitsoInput, CreateTedInput,
    CreateTransfersBitsoInput, CreateWireInput, Customer, CustomerLimit, CustomerLimits, Customers,
    FraudWarning, GetCustomerLimitsResponse, IdentificationDocument, KycType, KycWarning,
    LimitIncrease, LimitIncreaseStatus, LimitWindow, ListBankAccountsParams, ListCustomersParams,
    Owner, OwnerRole, OwnerTaxType, ProofOfAddressDocType, RecipientRelationship,
    RequestLimitIncreaseInput, RequestLimitIncreaseResponse, SpeiProtocol, SupportingDocumentType,
    SwiftPaymentCode, TransfersType, UpdateCustomerInput,
};
pub use resources::fees::{Fee, FeeOptions, Fees};
pub use resources::instances::{
    CreateWebhookEndpointInput, CreateWebhookEndpointResponse, InitiateTosInput,
    InitiateTosResponse, Instances, Member, PortalAccess, Tos, UpdateInstanceInput, UserRole,
    WebhookEndpoint, WebhookEndpointSecret, WebhookEndpoints,
};
pub use resources::partner_fees::{CreatePartnerFeeInput, PartnerFee, PartnerFees};
pub use resources::payins::{
    BlindpayBankAccount, BlindpayBankDetails, BlindpayBankParty, CreatePayinInput,
    CreatePayinQuoteInput, CreatePayinResponse, ListPayinsParams, PayerRules, Payin, PayinQuote,
    PayinQuoteFx, PayinQuoteFxInput, PayinQuotes, PayinTrackingComplete, PayinTrackingPartnerFee,
    PayinTrackingPayment, PayinTrackingTransaction, PayinTransactionStatus, Payins,
    PseDocumentType, PseInstruction, SwiftReceivingBank, TedInstruction, TransfersInstruction,
};
pub use resources::payouts::{
    AuthorizeStellarTokenInput, AuthorizeStellarTokenResponse, CreateEvmPayoutInput,
    CreatePayoutResponse, CreateSolanaPayoutInput, CreateStellarPayoutInput,
    EstimatedTimeOfArrival, ListPayoutsParams, Payout, PayoutCompleteStatus, PayoutDocumentsStatus,
    PayoutLiquidityProviderStatus, PayoutPaymentProviderStatus, PayoutTrackingComplete,
    PayoutTrackingDocuments, PayoutTrackingLiquidity, PayoutTrackingPartnerFee,
    PayoutTrackingPayment, PayoutTrackingStep, PayoutTrackingTransaction, PayoutTransactionStatus,
    Payouts, ProviderName, SubmitPayoutDocumentsInput, SubmitPayoutDocumentsResponse,
    TransactionDocumentType,
};
pub use resources::quotes::{
    CreateQuoteInput, GetFxRateInput, Quote, QuoteContract, QuoteContractNetwork, QuoteFx, Quotes,
};
pub use resources::transfers::{
    CreateTransferInput, CreateTransferQuoteInput, CreateTransferResponse, TrackingStatus,
    Transfer, TransferQuote, TransferQuotes, TransferTrackingStep,
    TransferTrackingTransactionMonitoring, Transfers,
};
pub use resources::upload::{Upload, UploadBucket, UploadResponse};
pub use resources::virtual_accounts::{
    BankingPartner, CreateVirtualAccountInput, SoleProprietorDocType, UpdateVirtualAccountInput,
    UsAddress, UsBankCoordinates, UsSwiftIntermediaryBank, UsVirtualAccountDetails, VirtualAccount,
    VirtualAccountWallet, VirtualAccounts,
};
pub use resources::wallets::{
    BlockchainWallet, BlockchainWalletMessage, BlockchainWallets, CreateAssetTrustlineResponse,
    CreateBlockchainWalletWithAddressInput, CreateBlockchainWalletWithHashInput,
    CreateOfframpWalletInput, CreateWalletInput, CustodialWallets, MintUsdbSolanaInput,
    MintUsdbSolanaResponse, MintUsdbStellarInput, OfframpWallet, OfframpWallets,
    PrepareSolanaDelegationInput, PrepareSolanaDelegationResponse, Wallet, WalletBalance,
    WalletTokenBalance, Wallets,
};
pub use webhooks::{WebhookVerificationError, verify_webhook_signature};
