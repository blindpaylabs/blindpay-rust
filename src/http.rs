//! Internal HTTP transport shared by all resources.

use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::client::Inner;
use crate::error::{ApiError, Error, Result};

impl Inner {
    /// Performs a `GET` request with no query parameters.
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request::<T, (), ()>(Method::GET, path, None, None)
            .await
    }

    /// Performs a `GET` request whose query string is the serialized `query`.
    ///
    /// Pass a `#[derive(Serialize)]` params struct (use
    /// `skip_serializing_if = "Option::is_none"` for optional params).
    pub(crate) async fn get_query<T, Q>(&self, path: &str, query: &Q) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        self.request::<T, Q, ()>(Method::GET, path, Some(query), None)
            .await
    }

    /// Performs an HTTP request and deserializes the JSON response into `T`.
    ///
    /// `path` is appended to the configured base URL (e.g. `/available/rails`).
    /// `query`, when present, is serialized to the query string; `body`, when
    /// present, is serialized as a JSON request body.
    pub(crate) async fn request<T, Q, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    {
        let url = format!("{}{}", self.base_url, path);

        let mut req = self
            .http
            .request(method, &url)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .header(reqwest::header::ACCEPT, "application/json");

        if let Some(query) = query {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(body);
        }

        let response = req.send().await.map_err(Error::Http)?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(Error::Http)?;

        if !status.is_success() {
            return Err(Error::Api(parse_api_error(status, &bytes)));
        }

        if status == StatusCode::NO_CONTENT || bytes.is_empty() {
            // No body to decode. Try `null` first so optional/scalar return
            // types resolve to `None`, then fall back to `[]` so the array
            // responses used by every current endpoint resolve to an empty Vec.
            return serde_json::from_slice(b"null")
                .or_else(|_| serde_json::from_slice(b"[]"))
                .map_err(|source| Error::Decode {
                    source,
                    body: String::new(),
                });
        }

        serde_json::from_slice(&bytes).map_err(|source| Error::Decode {
            source,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }

    /// Performs a multipart `POST` request and deserializes the JSON response.
    ///
    /// Mirrors [`Inner::request`]'s headers and error handling, but sends a
    /// `reqwest::multipart::Form` body. The `Content-Type`/boundary is set by
    /// reqwest — do not set it manually.
    pub(crate) async fn post_multipart<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        form: reqwest::multipart::Form,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .header(reqwest::header::ACCEPT, "application/json")
            .query(query)
            .multipart(form)
            .send()
            .await
            .map_err(Error::Http)?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(Error::Http)?;
        if !status.is_success() {
            return Err(Error::Api(parse_api_error(status, &bytes)));
        }
        if status == StatusCode::NO_CONTENT || bytes.is_empty() {
            return serde_json::from_slice(b"null")
                .or_else(|_| serde_json::from_slice(b"[]"))
                .map_err(|source| Error::Decode {
                    source,
                    body: String::new(),
                });
        }
        serde_json::from_slice(&bytes).map_err(|source| Error::Decode {
            source,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }
}

/// Builds an [`ApiError`] from a non-2xx response, parsing the documented
/// `{ "message": ... }` body shape and falling back to a synthesized message.
fn parse_api_error(status: StatusCode, body: &[u8]) -> ApiError {
    #[derive(serde::Deserialize)]
    struct Body {
        message: Option<String>,
    }

    let raw_body = String::from_utf8_lossy(body).into_owned();
    let message = match serde_json::from_slice::<Body>(body) {
        Ok(Body {
            message: Some(message),
        }) => message,
        _ => {
            let mut message = format!("HTTP {} error", status.as_u16());
            if !raw_body.is_empty() && raw_body.len() < 1000 {
                message.push_str(": ");
                message.push_str(&raw_body);
            }
            message
        }
    };

    ApiError {
        status: status.as_u16(),
        message,
        raw_body,
    }
}
