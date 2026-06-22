//! Types shared across multiple BlindPay resources.
//!
//! Everything here is re-exported from the crate root, so e.g. [`crate::Network`]
//! and [`crate::common::Network`] name the same type.

mod pagination;
mod scalars;

pub use pagination::{Limit, ListResponse, Offset, Pagination, PaginationParams};
pub use scalars::{
    AccountClass, AccountType, Country, Currency, CurrencyType, KycStatus, Network, PaymentMethod,
    ProviderName, Rail, Token, TrackingStatus, TransactionStatus, TransfersType, WebhookEvent,
};

use serde::Deserialize;

/// Defines an *open* string enum: a fixed set of known variants plus an
/// `Unknown(String)` catch-all, so a value this SDK version doesn't recognize
/// deserializes into `Unknown` instead of failing decoding.
///
/// Expands to the same shape as a hand-written reference impl — `as_str`,
/// `From<&str>`, `AsRef<str>`, `Display`, `Serialize`, and `Deserialize` — with
/// wire values matched verbatim (so casing, which varies across the API, is
/// preserved exactly). This is the single canonical pattern for categorical
/// fields.
macro_rules! open_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $wire:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
            /// A value not recognized by this version of the SDK.
            Unknown(String),
        }

        impl $name {
            /// Returns the wire-format string for this value (as sent to and
            /// received from the API).
            pub fn as_str(&self) -> &str {
                match self {
                    $( $name::$variant => $wire, )+
                    $name::Unknown(value) => value,
                }
            }
        }

        impl ::core::convert::From<&str> for $name {
            fn from(value: &str) -> Self {
                match value {
                    $( $wire => $name::$variant, )+
                    other => $name::Unknown(other.to_string()),
                }
            }
        }

        impl ::core::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let value = ::std::string::String::deserialize(deserializer)?;
                Ok($name::from(value.as_str()))
            }
        }
    };
}
pub(crate) use open_enum;

/// A generic success response (`{ "success": true }`), returned by many mutating
/// endpoints (most `DELETE`s and some `PUT`s).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Success {
    /// Whether the operation succeeded.
    pub success: bool,
}
