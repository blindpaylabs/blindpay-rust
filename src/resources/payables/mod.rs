//! Types for payables (bills a customer pays through a payout).
//!
//! Only the response type exists so far, for the `payable.*` webhook events;
//! the `payables` resource methods arrive in a later release.

mod types;

pub use types::{LastAttemptStatus, Payable, PayableStatus};
