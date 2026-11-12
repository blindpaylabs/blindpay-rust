#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod client;
pub mod common;
mod error;
mod error_code;
mod http;
mod internal;
mod options;
pub mod resources;
pub mod webhooks;

#[doc(hidden)]
pub use client::RetrySleep;
pub use client::{
    BlindPay, BlindPayBuilder, DEFAULT_BASE_URL, DEFAULT_MAX_RETRIES, DEFAULT_TIMEOUT,
};
pub use common::{
    AccountClass, AccountType, Country, Currency, CurrencyType, KycStatus, Limit, ListResponse,
    Network, Offset, Pager, Pagination, PaginationParams, PaymentMethod, ProviderName, Rail,
    Success, Token, TrackingStatus, TransactionStatus, TransfersType, WebhookEvent,
};
pub use error::{ApiError, Error, ErrorCode, ErrorCodeInfo, Result, RetryClass};
pub use options::RequestOptions;
pub use resources::available::{
    Available, BankDetail, BankDetailItem, BankDetailKey, NaicsCode, RailEntry, RequiredWhen,
    RequiredWhenOperator, SwiftCodeBankDetail,
};
pub use resources::customers::{
    AchCopDocument, AdditionalInfoItem, AdditionalInfoLabel, AmlHits, BankAccount,
    BankAccountStatus, BankAccounts, BusinessType, CreateAchCopBitsoInput, CreateAchInput,
    CreateBusinessWithStandardKybInput, CreateCustomerResponse, CreateFasterPaymentsInput,
    CreateIndividualWithEnhancedKycInput, CreateIndividualWithStandardKycInput,
    CreateInternationalSwiftInput, CreatePixInput, CreatePixSafeInput, CreatePlaidLinkInput,
    CreateRtpInput, CreateSepaInput, CreateSpeiBitsoInput, CreateTedInput,
    CreateTransfersBitsoInput, CreateWireInput, Customer, CustomerLimit, CustomerLimits, Customers,
    FraudWarning, GetCustomerLimitsResponse, IdentificationDocument, KycType, KycWarning,
    LimitIncrease, LimitIncreaseStatus, LimitIncreaseSupportingDocument, LimitWindow,
    ListBankAccountsParams, ListCustomersParams, Owner, OwnerRole, OwnerTaxType, PlaidLink,
    ProofOfAddressDocType, RecipientRelationship, RequestLimitIncreaseInput,
    RequestLimitIncreaseResponse, SpeiProtocol, SupportingDocumentType, SwiftPaymentCode,
    UpdateCustomerInput,
};
pub use resources::fees::{Fee, FeeOptions, Fees};
pub use resources::instances::{
    CreateWebhookEndpointInput, CreateWebhookEndpointResponse, InitiateTosInput,
    InitiateTosResponse, Instance, InstanceType, Instances, Member, Onboarding,
    OnboardingBusinessDetails, OnboardingBusinessProfile, OnboardingOwnershipDocuments,
    OnboardingStep, PortalAccess, Tos, UpdateInstanceInput, UserRole, WebhookEndpoint,
    WebhookEndpointSecret, WebhookEndpoints,
};
pub use resources::partner_fees::{CreatePartnerFeeInput, PartnerFee, PartnerFees};
pub use resources::payables::{
    CreatePayableInput, LastAttemptStatus, ListPayablesParams, Payable, PayableLineItem,
    PayableParty, PayableStatus, PayableType, Payables,
};
pub use resources::payins::{
    BlindpayBankAccount, BlindpayBankDetails, BlindpayBankParty, CreatePayinInput,
    CreatePayinQuoteInput, CreatePayinResponse, ListPayinsParams, ManualExecutionStatus,
    PayerRules, Payin, PayinQuote, PayinQuoteFx, PayinQuoteFxInput, PayinQuotes,
    PayinTrackingComplete, PayinTrackingPartnerFee, PayinTrackingPayment, PayinTrackingTransaction,
    PayinTransactionStatus, Payins, PseDocumentType, PseInstruction, SwiftReceivingBank,
    TedInstruction, TransfersInstruction,
};
pub use resources::payouts::{
    AuthorizeStellarTokenInput, AuthorizeStellarTokenResponse, CreateEvmPayoutInput,
    CreatePayoutResponse, CreateSolanaPayoutInput, CreateStellarPayoutInput,
    EstimatedTimeOfArrival, ListPayoutsParams, Payout, PayoutCompleteStatus, PayoutDocumentsStatus,
    PayoutLiquidityProviderStatus, PayoutPaymentProviderStatus, PayoutTrackingComplete,
    PayoutTrackingDocuments, PayoutTrackingLiquidity, PayoutTrackingPartnerFee,
    PayoutTrackingPayment, PayoutTrackingStep, PayoutTrackingTransaction, PayoutTransactionStatus,
    Payouts, SubmitPayoutDocumentsInput, SubmitPayoutDocumentsResponse, TransactionDocumentType,
};
pub use resources::quotes::{
    CreateQuoteInput, GetFxRateInput, Quote, QuoteContract, QuoteContractNetwork, QuoteFx, Quotes,
};
pub use resources::rfi::{
    CustomerRfi, CustomerRfis, InstanceRfi, InstanceRfis, RfiAnswer, RfiAnswers, RfiField,
    RfiFieldOption, RfiSection, RfiStatus, RfiSubmitFailureReason, RfiSubmitResponse,
};
pub use resources::transfers::{
    CreateTransferInput, CreateTransferQuoteInput, CreateTransferResponse, ListTransfersParams,
    Transfer, TransferQuote, TransferQuotes, TransferTrackingStep,
    TransferTrackingTransactionMonitoring, Transfers,
};
pub use resources::upload::{
    AnalyzeDocumentType, AnalyzeResponse, ApprovalRate, PresignResponse, Upload, UploadBucket,
    UploadFile, UploadResponse,
};
pub use resources::virtual_accounts::{
    BankingPartner, BreBGender, BreBInput, CoBreBDetails, CoVirtualAccountDetails,
    CreateVirtualAccountInput, MxSpeiDetails, MxVirtualAccountDetails, SoleProprietorDocType,
    UpdateVirtualAccountInput, UsAddress, UsBankCoordinates, UsSwiftIntermediaryBank,
    UsVirtualAccountDetails, VirtualAccount, VirtualAccountBeneficiary, VirtualAccountWallet,
    VirtualAccounts,
};
pub use resources::wallets::{
    BlockchainWallet, BlockchainWalletMessage, BlockchainWallets, CreateAssetTrustlineResponse,
    CreateBlockchainWalletWithAddressInput, CreateBlockchainWalletWithHashInput,
    CreateOfframpWalletInput, CreateWalletInput, CustodialWallets, DepositsLimit,
    ListWalletDepositsParams, MintUsdbSolanaInput, MintUsdbSolanaResponse, MintUsdbStellarInput,
    OfframpWallet, OfframpWallets, PrepareSolanaDelegationInput, PrepareSolanaDelegationResponse,
    UpdateOfframpWalletInput, Wallet, WalletBalance, WalletDeposit, WalletDepositStatus,
    WalletTokenBalance, Wallets, YieldStatus,
};
pub use webhooks::{
    CustomerDeleteEvent, TosAcceptEvent, WalletInboundEvent, WebhookEventPayload,
    WebhookVerificationError, construct_webhook_event, verify_webhook_signature,
};
