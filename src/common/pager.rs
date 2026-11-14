//! A forward-only auto-pager over cursor-paginated list endpoints.

use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use super::pagination::ListResponse;
use crate::client::Inner;
use crate::error::{Error, Result};

/// The page size the pager requests when the caller sets no `limit`. Valid in
/// every list's `limit` enum (including wallet deposits).
const DEFAULT_PAGE_SIZE: &str = "100";

/// Walks every item of a cursor-paginated list, one page at a time.
///
/// Returned by the `list_auto_paging` methods (e.g.
/// [`Payouts::list_auto_paging`](crate::Payouts::list_auto_paging)). It sends
/// your filters plus `limit` (100 unless you set one, so the API returns the
/// paginated envelope), and keeps requesting the next page with
/// `starting_after = pagination.next_page` while `pagination.has_more` is
/// true. Each page goes through the normal retry policy. Forward only: setting
/// `ending_before` is an error. If a page says `has_more` but its `next_page`
/// is a cursor this pager already requested (the same cursor again, or a
/// cycle such as A → B → A), the pager returns that page, then stops with
/// [`Error::Pagination`] instead of requesting pages forever.
///
/// ```no_run
/// # async fn run(client: blindpay::BlindPay) -> blindpay::Result<()> {
/// use blindpay::ListPayoutsParams;
///
/// let mut payouts = client.payouts.list_auto_paging(&ListPayoutsParams::new());
/// while let Some(payout) = payouts.next().await {
///     let payout = payout?;
///     println!("{} {}", payout.id, payout.status);
/// }
/// # Ok(())
/// # }
/// ```
pub struct Pager<T> {
    client: Arc<Inner>,
    path: String,
    query: std::result::Result<Map<String, Value>, String>,
    buffer: VecDeque<T>,
    next_cursor: Option<String>,
    done: bool,
    /// Every cursor this pager has sent as `starting_after`.
    sent: HashSet<String>,
    /// Returned (once) by the call after the page whose cursor was revisited.
    stalled: Option<String>,
    _item: PhantomData<fn() -> T>,
}

impl<T> std::fmt::Debug for Pager<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pager")
            .field("path", &self.path)
            .field("buffered", &self.buffer.len())
            .field("next_cursor", &self.next_cursor)
            .field("done", &self.done)
            .finish_non_exhaustive()
    }
}

impl<T: DeserializeOwned> Pager<T> {
    pub(crate) fn new<P: serde::Serialize + ?Sized>(
        client: Arc<Inner>,
        path: String,
        params: &P,
    ) -> Self {
        Self {
            client,
            path,
            query: initial_query(params),
            buffer: VecDeque::new(),
            next_cursor: None,
            done: false,
            sent: HashSet::new(),
            stalled: None,
            _item: PhantomData,
        }
    }

    /// The next item, fetching the next page when the current one is used up.
    /// `None` once every page has been read; after an error, the pager stops.
    pub async fn next(&mut self) -> Option<Result<T>> {
        loop {
            if let Some(item) = self.buffer.pop_front() {
                return Some(Ok(item));
            }
            match self.next_page().await? {
                Ok(page) => self.buffer.extend(page),
                Err(err) => return Some(Err(err)),
            }
        }
    }

    /// The remaining items of the current page, or the next page. `None` once
    /// every page has been read; after an error, the pager stops.
    pub async fn next_page(&mut self) -> Option<Result<Vec<T>>> {
        if !self.buffer.is_empty() {
            return Some(Ok(self.buffer.drain(..).collect()));
        }
        if let Some(cursor) = self.stalled.take() {
            return Some(Err(Error::Pagination { cursor }));
        }
        if self.done {
            return None;
        }
        let mut query = match &self.query {
            Ok(query) => query.clone(),
            Err(err) => {
                self.done = true;
                return Some(Err(Error::Config(err.clone())));
            }
        };
        if let Some(cursor) = self.next_cursor.take() {
            query.insert("starting_after".into(), Value::String(cursor.clone()));
            self.sent.insert(cursor);
        }
        let page = self
            .client
            .get_query::<ListResponse<T>, _>(&self.path, &query)
            .await;
        match page {
            Err(err) => {
                self.done = true;
                Some(Err(err))
            }
            Ok(ListResponse::Paginated { data, pagination }) => {
                let next = pagination.next_page.filter(|_| pagination.has_more);
                if next.as_ref().is_some_and(|c| self.sent.contains(c)) {
                    // A cursor we already requested: following it again
                    // would loop forever.
                    self.stalled = next;
                    self.done = true;
                } else {
                    self.done = next.is_none();
                    self.next_cursor = next;
                }
                Some(Ok(data))
            }
            Ok(ListResponse::Bare(data)) => {
                self.done = true;
                Some(Ok(data))
            }
        }
    }

    /// Reads every remaining page into one `Vec`.
    ///
    /// # Errors
    ///
    /// The first error any page returns.
    pub async fn collect_all(mut self) -> Result<Vec<T>> {
        let mut all = Vec::new();
        while let Some(page) = self.next_page().await {
            all.extend(page?);
        }
        Ok(all)
    }
}

/// The caller's params as a flat query map, with `limit` defaulted to 100.
fn initial_query<P: serde::Serialize + ?Sized>(
    params: &P,
) -> std::result::Result<Map<String, Value>, String> {
    let mut query = match serde_json::to_value(params) {
        Ok(Value::Object(map)) => map,
        Ok(Value::Null) => Map::new(),
        Ok(other) => {
            return Err(format!(
                "list params must serialize to an object, got {other}"
            ));
        }
        Err(e) => return Err(format!("invalid list params: {e}")),
    };
    if query.contains_key("ending_before") {
        return Err("the auto-pager only goes forward; unset `ending_before`".into());
    }
    query
        .entry("limit")
        .or_insert_with(|| Value::String(DEFAULT_PAGE_SIZE.into()));
    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Limit, PaginationParams};

    #[test]
    fn defaults_limit_and_rejects_ending_before() {
        let q = initial_query(&PaginationParams::new()).unwrap();
        assert_eq!(q.get("limit"), Some(&Value::String("100".into())));
        let q = initial_query(&PaginationParams::new().limit(Limit::Ten)).unwrap();
        assert_eq!(q.get("limit"), Some(&Value::String("10".into())));
        assert!(initial_query(&PaginationParams::new().ending_before("x")).is_err());
    }
}
