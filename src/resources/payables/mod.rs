//! The `payables` resource: bills (boletos, PIX codes, invoices) a customer
//! pays through a payout. Quote a payable with
//! [`CreateQuoteInput::payable_id`](crate::CreateQuoteInput::payable_id).

mod types;

pub use types::{
    CreatePayableInput, LastAttemptStatus, ListPayablesParams, Payable, PayableLineItem,
    PayableParty, PayableStatus, PayableType,
};

use std::sync::Arc;

use reqwest::Method;

use crate::client::Inner;
use crate::common::{ListResponse, Pager};
use crate::error::Result;
use crate::internal::encode_path_segment;
use crate::options::RequestOptions;

/// Handle for the `payables` resource.
///
/// Obtained from the `payables` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Payables {
    client: Arc<Inner>,
}

impl Payables {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    fn collection_path(&self) -> String {
        self.client.instance_path("/payables")
    }

    fn item_path(&self, id: &str) -> String {
        format!("{}/{}", self.collection_path(), encode_path_segment(id))
    }

    /// Creates a payable in `draft`.
    ///
    /// `POST /instances/{instance_id}/payables`
    pub async fn create(&self, input: &CreatePayableInput) -> Result<Payable> {
        self.create_with_options(input, &RequestOptions::default())
            .await
    }

    /// Like [`Self::create`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /instances/{instance_id}/payables`
    pub async fn create_with_options(
        &self,
        input: &CreatePayableInput,
        options: &RequestOptions,
    ) -> Result<Payable> {
        self.client
            .request_with::<_, (), _>(
                Method::POST,
                &self.collection_path(),
                None,
                Some(input),
                options,
            )
            .await
    }

    /// Lists payables. Returns every row unless `limit` or a cursor is set
    /// (then the paginated envelope, default page size 10); prefer
    /// [`Self::list_auto_paging`] for large instances.
    ///
    /// `GET /instances/{instance_id}/payables`
    pub async fn list(&self, params: &ListPayablesParams) -> Result<ListResponse<Payable>> {
        self.client.get_query(&self.collection_path(), params).await
    }

    /// Iterates every payable matching `params`, page by page (see [`Pager`]).
    ///
    /// `GET /instances/{instance_id}/payables`
    pub fn list_auto_paging(&self, params: &ListPayablesParams) -> Pager<Payable> {
        Pager::new(Arc::clone(&self.client), self.collection_path(), params)
    }

    /// Retrieves a payable by id.
    ///
    /// `GET /instances/{instance_id}/payables/{id}`
    pub async fn get(&self, id: impl AsRef<str>) -> Result<Payable> {
        self.client.get(&self.item_path(id.as_ref())).await
    }

    /// Deletes (cancels) a draft payable and returns it.
    ///
    /// `DELETE /instances/{instance_id}/payables/{id}`
    pub async fn delete(&self, id: impl AsRef<str>) -> Result<Payable> {
        self.delete_with_options(id, &RequestOptions::default())
            .await
    }

    /// Like [`Self::delete`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `DELETE /instances/{instance_id}/payables/{id}`
    pub async fn delete_with_options(
        &self,
        id: impl AsRef<str>,
        options: &RequestOptions,
    ) -> Result<Payable> {
        self.client
            .request_with::<_, (), ()>(
                Method::DELETE,
                &self.item_path(id.as_ref()),
                None,
                None,
                options,
            )
            .await
    }
}
