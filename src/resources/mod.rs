//! API resources.
//!
//! Each resource groups a set of related endpoints. Resources are accessed
//! through the [`BlindPay`](crate::BlindPay) client (for example
//! [`BlindPay::available`](crate::BlindPay::available)).

pub mod available;
pub mod customers;
pub mod fees;
pub mod instances;
pub mod partner_fees;
pub mod payins;
pub mod payouts;
pub mod quotes;
pub mod transfers;
pub mod upload;
pub mod virtual_accounts;
pub mod wallets;
