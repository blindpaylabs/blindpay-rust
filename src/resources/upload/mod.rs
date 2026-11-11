//! The `upload` resource: a single multipart `POST /upload` endpoint for
//! uploading files (avatars, onboarding documents, limit-increase documents) to
//! BlindPay storage.
//!
//! Unlike the other resources, `/upload` is instance-agnostic: the path carries
//! no instance segment and the instance is passed as the `instance_id` query
//! parameter (matching node/go/cli). The bound client instance id is sent
//! automatically.

mod types;

pub use types::{
    AnalyzeDocumentType, AnalyzeResponse, ApprovalRate, PresignResponse, UploadBucket, UploadFile,
    UploadResponse,
};

use std::sync::Arc;

use reqwest::multipart::{Form, Part};

use crate::client::Inner;
use crate::error::Result;
use crate::options::RequestOptions;
use reqwest::Method;

/// Handle for the `upload` resource.
///
/// Obtained from the `upload` field of a [`BlindPay`](crate::BlindPay) client.
#[derive(Clone)]
pub struct Upload {
    client: Arc<Inner>,
}

impl Upload {
    pub(crate) fn new(client: Arc<Inner>) -> Self {
        Self { client }
    }

    /// Uploads a file to the given storage `bucket` and returns its URL.
    ///
    /// The file is sent as the multipart `file` field (with its name and
    /// content type) next to a `bucket` field; the client's instance id is
    /// sent as the `instance_id` query parameter. Sent once (multipart is
    /// never retried).
    ///
    /// `POST /upload?instance_id={instance_id}`
    pub async fn upload(&self, file: UploadFile, bucket: UploadBucket) -> Result<UploadResponse> {
        self.upload_with_options(file, bucket, &RequestOptions::default())
            .await
    }

    /// Like [`Self::upload`], with per-request [`RequestOptions`]
    /// (timeout, extra headers; multipart requests carry no idempotency key).
    ///
    /// `POST /upload?instance_id={instance_id}`
    pub async fn upload_with_options(
        &self,
        file: UploadFile,
        bucket: UploadBucket,
        options: &RequestOptions,
    ) -> Result<UploadResponse> {
        let form = Form::new()
            .part("file", file_part(file)?)
            .text("bucket", bucket.as_str().to_string());
        self.client
            .post_multipart_with(
                "/upload",
                &[("instance_id", &self.client.instance_id)],
                form,
                options,
            )
            .await
    }

    /// Asks BlindPay's AI to check a document before you submit it: returns
    /// an approval likelihood and what it found. `metadata` holds expected
    /// values to compare against (e.g. `{"full_name": "Jane Doe"}`); a clear
    /// mismatch caps the rating at `low`. Sent once (multipart is never
    /// retried).
    ///
    /// `POST /upload/analyze?instance_id={instance_id}`
    pub async fn analyze(
        &self,
        file: UploadFile,
        document_type: AnalyzeDocumentType,
        metadata: Option<&serde_json::Value>,
    ) -> Result<AnalyzeResponse> {
        self.analyze_with_options(file, document_type, metadata, &RequestOptions::default())
            .await
    }

    /// Like [`Self::analyze`], with per-request [`RequestOptions`]
    /// (timeout, extra headers; multipart requests carry no idempotency key).
    ///
    /// `POST /upload/analyze?instance_id={instance_id}`
    pub async fn analyze_with_options(
        &self,
        file: UploadFile,
        document_type: AnalyzeDocumentType,
        metadata: Option<&serde_json::Value>,
        options: &RequestOptions,
    ) -> Result<AnalyzeResponse> {
        let mut part = Part::bytes(file.bytes).file_name(file.file_name);
        if let Some(content_type) = &file.content_type {
            part = part
                .mime_str(content_type)
                .map_err(|e| crate::error::Error::Config(format!("invalid content type: {e}")))?;
        }
        let mut form = Form::new()
            .part("file", part)
            .text("type", document_type.as_str().to_string());
        if let Some(metadata) = metadata {
            form = form.text("metadata", metadata.to_string());
        }
        self.client
            .post_multipart_with(
                "/upload/analyze",
                &[("instance_id", &self.client.instance_id)],
                form,
                options,
            )
            .await
    }

    /// Returns a temporary signed URL for a BlindPay-hosted file (e.g. one
    /// returned by [`Self::upload`]).
    ///
    /// `POST /presign?instance_id={instance_id}`
    pub async fn presign(&self, file_url: impl Into<String>) -> Result<PresignResponse> {
        self.presign_with_options(file_url, &RequestOptions::default())
            .await
    }

    /// Like [`Self::presign`], with per-request [`RequestOptions`]
    /// (idempotency key, timeout, extra headers).
    ///
    /// `POST /presign?instance_id={instance_id}`
    pub async fn presign_with_options(
        &self,
        file_url: impl Into<String>,
        options: &RequestOptions,
    ) -> Result<PresignResponse> {
        #[derive(serde::Serialize)]
        struct Body {
            file_url: String,
        }
        let query = [("instance_id", self.client.instance_id.as_str())];
        self.client
            .request_with::<_, _, _>(
                Method::POST,
                "/presign",
                Some(&query),
                Some(&Body {
                    file_url: file_url.into(),
                }),
                options,
            )
            .await
    }
}

/// The multipart `file` part: bytes, file name and (optional) content type.
fn file_part(file: UploadFile) -> Result<Part> {
    let part = Part::bytes(file.bytes).file_name(file.file_name);
    match &file.content_type {
        Some(content_type) => part
            .mime_str(content_type)
            .map_err(|e| crate::error::Error::Config(format!("invalid content type: {e}"))),
        None => Ok(part),
    }
}
