//! Request and response types for the `upload` resource.

use crate::common::open_enum;

open_enum! {
    /// The storage bucket a file is uploaded into.
    pub enum UploadBucket {
        /// Profile / instance avatar images.
        Avatar => "avatar",
        /// Onboarding (KYC/KYB) supporting documents.
        Onboarding => "onboarding",
        /// Limit-increase supporting documents.
        LimitIncrease => "limit_increase",
        /// General documents (e.g. payable or RFI attachments).
        Documents => "documents",
    }
}

/// Response from `Upload::upload`: the URL of the stored file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[non_exhaustive]
pub struct UploadResponse {
    /// Public URL of the uploaded file.
    pub file_url: String,
}

open_enum! {
    /// The kind of document `Upload::analyze` checks; selects its rule set.
    pub enum AnalyzeDocumentType {
        /// Proof of address.
        ProofOfAddress => "proof_of_address",
        /// Identity document.
        IdentityDocument => "identity_document",
        /// Incorporation document.
        IncorporationDocument => "incorporation_document",
        /// Proof of ownership.
        ProofOfOwnership => "proof_of_ownership",
        /// Source of funds.
        SourceOfFunds => "source_of_funds",
        /// Selfie.
        Selfie => "selfie",
        /// Bank statement.
        BankStatement => "bank_statement",
        /// Tax return.
        TaxReturn => "tax_return",
        /// Proof of income.
        ProofOfIncome => "proof_of_income",
        /// Financial statement.
        FinancialStatement => "financial_statement",
        /// Invoice.
        Invoice => "invoice",
        /// Transaction document.
        TransactionDocument => "transaction_document",
        /// Passport.
        Passport => "passport",
        /// Pay stub.
        PayStub => "pay_stub",
        /// Employment letter.
        EmploymentLetter => "employment_letter",
        /// Investment statement.
        Investment => "investment",
        /// Crypto exchange statement.
        CryptoExchange => "crypto_exchange",
        /// Blockchain wallet statement.
        BlockchainWallet => "blockchain_wallet",
        /// Contract.
        Contract => "contract",
        /// Accounts receivable.
        AccountsReceivable => "accounts_receivable",
        /// Merchant processor statement.
        MerchantProcessor => "merchant_processor",
        /// Shareholder loan.
        ShareholderLoan => "shareholder_loan",
    }
}

open_enum! {
    /// How likely the analyzed document is to be approved.
    pub enum ApprovalRate {
        /// Unlikely.
        Low => "low",
        /// Possibly.
        Medium => "medium",
        /// Likely.
        High => "high",
    }
}

/// A file to send as multipart: bytes, a file name and an optional content
/// type (e.g. `application/pdf`; defaults to `application/octet-stream`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UploadFile {
    /// File name sent with the part.
    pub file_name: String,
    /// File contents.
    pub bytes: Vec<u8>,
    /// MIME type of the contents.
    pub content_type: Option<String>,
}

impl UploadFile {
    /// Creates a file part.
    pub fn new(file_name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            file_name: file_name.into(),
            bytes: bytes.into(),
            content_type: None,
        }
    }

    /// Sets the MIME type.
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

/// The result of `Upload::analyze` (spec `UploadAnalyzeOut`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[non_exhaustive]
pub struct AnalyzeResponse {
    /// How likely the document is to be approved.
    pub approval_rate: ApprovalRate,
    /// What the analysis found.
    pub description: String,
}

/// The result of `Upload::presign` (spec `PresignOut`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[non_exhaustive]
pub struct PresignResponse {
    /// A temporary, signed URL for the file.
    pub file_url: String,
    /// When the signed URL stops working (ISO 8601).
    pub expires_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_bucket_round_trips() {
        for (variant, wire) in [
            (UploadBucket::Avatar, "\"avatar\""),
            (UploadBucket::Onboarding, "\"onboarding\""),
            (UploadBucket::LimitIncrease, "\"limit_increase\""),
        ] {
            assert_eq!(serde_json::to_string(&variant).unwrap(), wire);
            assert_eq!(serde_json::from_str::<UploadBucket>(wire).unwrap(), variant);
        }
        assert_eq!(UploadBucket::LimitIncrease.as_str(), "limit_increase");
    }
}
