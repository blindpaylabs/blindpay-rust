//! The BlindPay API client and its builder.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;

use crate::error::{Error, Result};
use crate::resources::available::Available;
use crate::resources::customers::Customers;
use crate::resources::fees::Fees;
use crate::resources::instances::Instances;
use crate::resources::partner_fees::PartnerFees;
use crate::resources::payins::Payins;
use crate::resources::payouts::Payouts;
use crate::resources::quotes::Quotes;
use crate::resources::transfers::Transfers;
use crate::resources::upload::Upload;
use crate::resources::virtual_accounts::VirtualAccounts;
use crate::resources::wallets::Wallets;

/// The default BlindPay API base URL (`https://api.blindpay.com/v1`).
pub const DEFAULT_BASE_URL: &str = "https://api.blindpay.com/v1";

/// The default per-request timeout (30 seconds).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Shared, reference-counted client state.
pub(crate) struct Inner {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) instance_id: String,
    pub(crate) user_agent: String,
}

/// A client for the BlindPay API.
///
/// `BlindPay` is cheap to clone — its internal state is reference-counted, so
/// clones share the same connection pool and configuration.
///
/// # Example
///
/// ```no_run
/// # async fn run() -> blindpay::Result<()> {
/// let client = blindpay::BlindPay::new("your-api-key", "your-instance-id")?;
/// let rails = client.available.get_rails().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct BlindPay {
    inner: Arc<Inner>,
    /// The `available` resource: payment rails, bank-detail field definitions,
    /// NAICS codes, and SWIFT/BIC lookups.
    pub available: Available,
    /// The `instances` resource: instance settings, ToS, members, and webhook
    /// endpoints.
    pub instances: Instances,
    /// The `partner_fees` resource: partner fee schedules for the instance.
    pub partner_fees: PartnerFees,
    /// The `fees` resource: billing fees for the instance.
    pub fees: Fees,
    /// The `customers` resource: customer KYC/KYB and bank accounts.
    pub customers: Customers,
    /// The `wallets` resource: blockchain, custodial, and offramp wallets.
    pub wallets: Wallets,
    /// The `virtual_accounts` resource: customer virtual accounts.
    pub virtual_accounts: VirtualAccounts,
    /// The `quotes` resource: payout quotes and FX rates.
    pub quotes: Quotes,
    /// The `payins` resource: pay-ins and their quotes.
    pub payins: Payins,
    /// The `payouts` resource: payouts and their tracking.
    pub payouts: Payouts,
    /// The `transfers` resource: transfers and their quotes.
    pub transfers: Transfers,
    /// The `upload` resource: file uploads.
    pub upload: Upload,
}

impl BlindPay {
    /// Verifies a Svix webhook signature against the raw request body.
    ///
    /// Delegates to [`crate::verify_webhook_signature`]. See that function for
    /// header/secret/payload details.
    ///
    /// # Errors
    ///
    /// Returns [`crate::webhooks::WebhookVerificationError`] if the signature is
    /// invalid or the inputs are malformed.
    #[allow(clippy::unused_self)] // inherent method for parity with other SDKs (PLAN Tier 2)
    pub fn verify_webhook_signature(
        &self,
        secret: &str,
        id: &str,
        timestamp: &str,
        payload: &[u8],
        signature: &str,
    ) -> std::result::Result<(), crate::webhooks::WebhookVerificationError> {
        crate::verify_webhook_signature(secret, id, timestamp, payload, signature)
    }
}

impl BlindPay {
    /// Creates a client with the given API key and instance ID, using the
    /// default base URL and timeout.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if `api_key` or `instance_id` is empty.
    pub fn new(api_key: impl Into<String>, instance_id: impl Into<String>) -> Result<Self> {
        Self::builder(api_key, instance_id).build()
    }

    /// Starts building a customized client (custom base URL, timeout, or
    /// [`reqwest::Client`]).
    pub fn builder(api_key: impl Into<String>, instance_id: impl Into<String>) -> BlindPayBuilder {
        BlindPayBuilder::new(api_key.into(), instance_id.into())
    }

    /// Returns the instance ID this client was configured with.
    pub fn instance_id(&self) -> &str {
        &self.inner.instance_id
    }
}

impl std::fmt::Debug for BlindPay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlindPay")
            .field("base_url", &self.inner.base_url)
            .field("instance_id", &self.inner.instance_id)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

/// A builder for [`BlindPay`], created via [`BlindPay::builder`].
pub struct BlindPayBuilder {
    api_key: String,
    instance_id: String,
    base_url: String,
    timeout: Duration,
    http_client: Option<reqwest::Client>,
}

impl BlindPayBuilder {
    fn new(api_key: String, instance_id: String) -> Self {
        Self {
            api_key,
            instance_id,
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            http_client: None,
        }
    }

    /// Overrides the API base URL. Defaults to [`DEFAULT_BASE_URL`].
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Overrides the per-request timeout. Defaults to [`DEFAULT_TIMEOUT`].
    ///
    /// Ignored when a custom client is supplied via [`Self::http_client`].
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Supplies a fully configured [`reqwest::Client`] (for proxies, custom TLS,
    /// connection pooling, etc.). When set, [`Self::timeout`] is ignored —
    /// configure the timeout on your own client.
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Builds the [`BlindPay`] client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the API key or instance ID is empty, or if
    /// the base URL cannot be parsed.
    pub fn build(self) -> Result<BlindPay> {
        if self.api_key.trim().is_empty() {
            return Err(Error::Config(
                "api key not provided, get your api key on the blindpay dashboard".to_string(),
            ));
        }
        if self.instance_id.trim().is_empty() {
            return Err(Error::Config(
                "instance id not provided, get your instance id on the blindpay dashboard"
                    .to_string(),
            ));
        }

        // Trim a trailing slash so request paths (which start with `/`) join
        // cleanly, and validate that the result is a well-formed URL.
        let base_url = {
            let trimmed = self.base_url.trim_end_matches('/');
            Url::parse(trimmed).map_err(|e| Error::Config(format!("invalid base url: {e}")))?;
            trimmed.to_string()
        };

        let http = match self.http_client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .timeout(self.timeout)
                .build()
                .map_err(Error::Http)?,
        };

        let user_agent = format!("blindpay-rust/{}", env!("CARGO_PKG_VERSION"));

        let inner = Arc::new(Inner {
            http,
            base_url,
            api_key: self.api_key,
            instance_id: self.instance_id,
            user_agent,
        });
        let available = Available::new(Arc::clone(&inner));
        let instances = Instances::new(Arc::clone(&inner));
        let partner_fees = PartnerFees::new(Arc::clone(&inner));
        let fees = Fees::new(Arc::clone(&inner));
        let customers = Customers::new(Arc::clone(&inner));
        let wallets = Wallets::new(Arc::clone(&inner));
        let virtual_accounts = VirtualAccounts::new(Arc::clone(&inner));
        let quotes = Quotes::new(Arc::clone(&inner));
        let payins = Payins::new(Arc::clone(&inner));
        let payouts = Payouts::new(Arc::clone(&inner));
        let transfers = Transfers::new(Arc::clone(&inner));
        let upload = Upload::new(Arc::clone(&inner));

        Ok(BlindPay {
            inner,
            available,
            instances,
            partner_fees,
            fees,
            customers,
            wallets,
            virtual_accounts,
            quotes,
            payins,
            payouts,
            transfers,
            upload,
        })
    }
}
