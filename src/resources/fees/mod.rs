//! The `fees` resource: the per-rail and per-chain fee schedule configured for
//! an instance.

mod types;

pub use types::{Fee, FeeOptions};

use std::sync::Arc;

use crate::client::Inner;
use crate::error::Result;

/// Handle for the `fees` resource.
///
/// Obtained from the `fees` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Fees {
    client: Arc<Inner>,
}

impl Fees {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Returns the fee schedule for the instance.
    ///
    /// `GET /instances/{instance_id}/billing/fees`
    pub async fn get(&self) -> Result<Fee> {
        let path = format!("/instances/{}/billing/fees", self.client.instance_id);
        self.client.get(&path).await
    }
}
