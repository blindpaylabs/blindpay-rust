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
    }
}

/// Response from `Upload::upload`: the URL of the stored file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[non_exhaustive]
pub struct UploadResponse {
    /// Public URL of the uploaded file.
    pub file_url: String,
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
