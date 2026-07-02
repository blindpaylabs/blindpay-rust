//! Request and response types for the `available` resource.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::common::Rail;

/// An available payment rail entry returned by `Available::get_rails`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RailEntry {
    /// Human-readable label, e.g. `"Domestic Wire"`.
    pub label: String,
    /// The rail identifier.
    pub value: Rail,
    /// ISO 3166-1 alpha-2 country code, e.g. `"US"`. Kept as a string because
    /// the API may return non-ISO display codes (e.g. `eu` for SEPA).
    pub country: String,
}

/// The key identifying a bank-detail field (e.g. `account_number`, `pix_key`).
///
/// The API defines this as a large, evolving set of values, so it is modeled as
/// a transparent newtype rather than an enumerated type — strongly typed (never
/// a bare `String`) yet forward-compatible with new keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[non_exhaustive]
pub struct BankDetailKey(String);

impl BankDetailKey {
    /// Returns the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for BankDetailKey {
    fn from(value: &str) -> Self {
        BankDetailKey(value.to_string())
    }
}

impl From<String> for BankDetailKey {
    fn from(value: String) -> Self {
        BankDetailKey(value)
    }
}

impl AsRef<str> for BankDetailKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BankDetailKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<str> for BankDetailKey {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for BankDetailKey {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// The comparison operator in a [`RequiredWhen`] rule.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequiredWhenOperator {
    /// The field's value is one of the listed values.
    In,
    /// The field's value equals the listed value.
    Eq,
    /// The field's value is none of the listed values.
    NotIn,
    /// The field's value does not equal the listed value.
    NotEq,
    /// An operator not recognized by this version of the SDK.
    Unknown(String),
}

impl RequiredWhenOperator {
    /// Returns the wire-format string for this operator.
    pub fn as_str(&self) -> &str {
        match self {
            RequiredWhenOperator::In => "in",
            RequiredWhenOperator::Eq => "eq",
            RequiredWhenOperator::NotIn => "notIn",
            RequiredWhenOperator::NotEq => "notEq",
            RequiredWhenOperator::Unknown(value) => value,
        }
    }
}

impl From<&str> for RequiredWhenOperator {
    fn from(value: &str) -> Self {
        match value {
            "in" => RequiredWhenOperator::In,
            "eq" => RequiredWhenOperator::Eq,
            "notIn" => RequiredWhenOperator::NotIn,
            "notEq" => RequiredWhenOperator::NotEq,
            other => RequiredWhenOperator::Unknown(other.to_string()),
        }
    }
}

impl AsRef<str> for RequiredWhenOperator {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for RequiredWhenOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for RequiredWhenOperator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RequiredWhenOperator {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(RequiredWhenOperator::from(value.as_str()))
    }
}

/// A selectable option within a [`BankDetail`] field (e.g. one entry of a
/// dropdown).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BankDetailItem {
    /// Human-readable label for the option.
    pub label: String,
    /// The option's value.
    pub value: String,
    /// Whether the option is currently active. Absent for some options.
    #[serde(default)]
    pub is_active: Option<bool>,
}

/// A conditional requirement rule attached to a [`BankDetail`] field.
///
/// The field becomes required only when the referenced `field`'s value matches,
/// according to `operator`, one of `values`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RequiredWhen {
    /// The key of the field this rule depends on.
    pub field: BankDetailKey,
    /// The comparison operator.
    pub operator: RequiredWhenOperator,
    /// The values compared against the referenced field.
    pub values: Vec<String>,
}

/// A bank-detail field definition for a rail, returned by
/// `Available::get_bank_details`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct BankDetail {
    /// Human-readable label for the field.
    pub label: String,
    /// Validation regular expression for the field's value (may be empty).
    pub regex: String,
    /// The field key, e.g. `"account_number"` or `"pix_key"`.
    pub key: BankDetailKey,
    /// Predefined options for the field (e.g. a dropdown). Empty when the field
    /// is free-form.
    #[serde(default)]
    pub items: Vec<BankDetailItem>,
    /// Whether the field is required. May be absent or `null` in the response.
    #[serde(default)]
    pub required: Option<bool>,
    /// A conditional requirement rule, present when the field is only required
    /// in certain cases.
    #[serde(default, rename = "requiredWhen")]
    pub required_when: Option<RequiredWhen>,
}

/// A NAICS business-industry code, returned by `Available::get_naics_codes`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct NaicsCode {
    /// Human-readable label, e.g. `"(339910) Jewelry and Silverware Manufacturing"`.
    pub label: String,
    /// The NAICS code value, e.g. `"339910"`.
    pub value: String,
}

/// Bank details for a SWIFT/BIC code, returned by
/// `Available::get_swift_code_bank_details`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SwiftCodeBankDetail {
    /// Internal identifier for the record.
    pub id: String,
    /// Bank name.
    pub bank: String,
    /// City.
    pub city: String,
    /// Branch description.
    pub branch: String,
    /// The SWIFT/BIC code.
    pub swift_code: String,
    /// A link to more information about the SWIFT code.
    pub swift_code_link: String,
    /// Country name.
    pub country: String,
    /// URL-friendly country slug.
    pub country_slug: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_when_operator_round_trips() {
        // `notIn`/`notEq` are camelCase on the wire — they must survive serde
        // unchanged, not get snake/lower-cased.
        for (variant, wire) in [
            (RequiredWhenOperator::In, "\"in\""),
            (RequiredWhenOperator::Eq, "\"eq\""),
            (RequiredWhenOperator::NotIn, "\"notIn\""),
            (RequiredWhenOperator::NotEq, "\"notEq\""),
        ] {
            assert_eq!(serde_json::to_string(&variant).unwrap(), wire);
            assert_eq!(
                serde_json::from_str::<RequiredWhenOperator>(wire).unwrap(),
                variant
            );
        }
        let parsed: RequiredWhenOperator = serde_json::from_str("\"weird\"").unwrap();
        assert_eq!(parsed, RequiredWhenOperator::Unknown("weird".to_string()));
    }

    #[test]
    fn bank_detail_key_is_transparent() {
        let key: BankDetailKey = serde_json::from_str("\"account_number\"").unwrap();
        assert_eq!(key.as_str(), "account_number");
        assert_eq!(key, "account_number");
        assert_eq!(serde_json::to_string(&key).unwrap(), "\"account_number\"");
    }
}
