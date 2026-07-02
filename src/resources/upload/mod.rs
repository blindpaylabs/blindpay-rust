//! The `upload` resource: a single multipart `POST /upload` endpoint for
//! uploading files (avatars, onboarding documents, limit-increase documents) to
//! BlindPay storage.
//!
//! Unlike the other resources, `/upload` is instance-agnostic: the path carries
//! no instance segment and the instance is passed as the `instance_id` query
//! parameter (matching node/go/cli). The bound client instance id is sent
//! automatically.

mod types;

pub use types::{UploadBucket, UploadResponse};

use std::sync::Arc;

use reqwest::multipart::{Form, Part};

use crate::client::Inner;
use crate::error::Result;

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

    /// Uploads a file to the given storage `bucket` and returns its public URL.
    ///
    /// The file is sent as multipart `file` + `bucket` form fields; the client's
    /// instance id is sent as the `instance_id` query parameter.
    ///
    /// `POST /upload?instance_id={instance_id}`
    pub async fn upload(
        &self,
        file_name: impl Into<String>,
        file: impl Into<Vec<u8>>,
        bucket: UploadBucket,
    ) -> Result<UploadResponse> {
        let part = Part::bytes(file.into()).file_name(file_name.into());
        let form = Form::new()
            .part("file", part)
            .text("bucket", bucket.as_str().to_string());

        self.client
            .post_multipart(
                "/upload",
                &[("instance_id", &self.client.instance_id)],
                form,
            )
            .await
    }
}
