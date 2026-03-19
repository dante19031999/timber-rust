// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "loki")]
#![cfg_attr(docsrs, doc(cfg(feature = "loki")))]

use crate::service::serde::{BasicAuth, FlexibleDuration};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer};
use std::time::Duration;

/// Default app for loki streams.
pub const LOKI_DEFAULT_APP: &str = "rust-app";
/// Default job for loki streams.
pub const LOKI_DEFAULT_JOB: &str = "rust-job";
/// Default env for loki streams.
pub const LOKI_DEFAULT_ENV: &str = "rust-env";
/// Default retrie number for loki.
pub const LOKI_DEFAULT_RETRIES: usize = 3;
/// Default worker number for loki.
pub const LOKI_DEFAULT_WORKERS: usize = 1;
/// Default connection timeout for loki (1 second).
pub const LOKI_DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(1);
/// Default request timeout for loki (2 seconds).
pub const LOKI_DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Configuration parameters for connecting to a Grafana Loki instance.
///
/// This struct follows the **Builder Pattern**, allowing you to specify
/// metadata (labels) that will be attached to every log stream sent to Loki.
///
/// Only available when the `loki` feature is enabled.
///
/// ### Example
/// ```rust
/// # use timber_rust::Logger;
/// # use timber_rust::service::{LokiConfig};
/// # use timber_rust::LokiLogger;
/// let config = LokiConfig::new("[https://logs-prod-us-central1.grafana.net/loki/api/v1/push](https://logs-prod-us-central1.grafana.net/loki/api/v1/push)")
///     .job("api-server")
///     .app("billing-v2")
///     .env("prod")
///     .basic_auth(Some(("12345", Some("your-api-key"))))
///     .workers(4);
///
/// let logger = LokiLogger::new(config);
/// let logger = Logger::new(logger);
/// ```
#[derive(Clone)]
pub struct Config {
    pub(crate) url: String,
    pub(crate) app: String,
    pub(crate) job: String,
    pub(crate) env: String,
    pub(crate) basic_auth: Option<BasicAuth>,
    pub(crate) bearer_auth: Option<String>,
    pub(crate) connection_timeout: Duration,
    pub(crate) request_timeout: Duration,
    pub(crate) max_retries: usize,
    pub(crate) workers: usize,
}

/// A network-based logging backend that pushes logs to Grafana Loki.
///
/// [`LokiService`][`crate::service::Loki`] transforms internal [`Message`][`crate::Message`] objects into Loki's
/// JSON "Push" format. It uses a blocking HTTP client, which is intended
/// to be executed within a dedicated background worker thread to avoid
/// blocking the main application.
///
/// ### Stream Labels
/// Every log sent via this service is tagged with the following labels:
/// - `job`: The logical group (e.g., "logger-service").
/// - `app`: The specific application name.
/// - `env`: The deployment environment (e.g., "production", "dev").
/// - `level`: The severity of the log (automatically extracted from the message).
///
/// ### Client Data
/// - `url`: Base url for loki
/// - `connection_timeout`: Connection timeout (how much time to wait for the connection to happen)
/// - `request_timeout`: Request timeout (how much time to wait for the request's response)
///
/// ### Logger data:
/// - `max_retries`: Maximum number of retries (only used in the [`LoggerFactory`][`crate::LoggerFactory`])
/// - `workers`: Number of workers to use (only used in the [`LoggerFactory`][`crate::LoggerFactory`])
impl Config {
    /// Creates a new [`LokiConfig`][`Config`] with default settings.
    ///
    /// # Parameters
    /// - `url`: Base url for loki.
    ///
    /// # Default Values:
    /// - **App:** [`LOKI_DEFAULT_APP`]
    /// - **Job:** [`LOKI_DEFAULT_JOB`]
    /// - **Env:** [`LOKI_DEFAULT_ENV`]
    /// - **Workers:** [`LOKI_DEFAULT_WORKERS`]
    /// - **Connection timeout**: [`LOKI_DEFAULT_CONNECTION_TIMEOUT`]
    /// - **Request timeout**: [`LOKI_DEFAULT_REQUEST_TIMEOUT`]
    /// - **Maximum retries**: [`LOKI_DEFAULT_RETRIES`]
    pub fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self::with_labels(
            url,
            LOKI_DEFAULT_APP.to_string(),
            LOKI_DEFAULT_JOB.to_string(),
            LOKI_DEFAULT_ENV.to_string(),
        )
    }

    /// Creates a new [`LokiConfig`][`Config`] with customized labels default settings.
    ///
    /// # Parameters
    /// - `url`: Base url for loki.
    /// - `job`: The logical group (e.g., "logger-service").
    /// - `app`: The specific application name.
    /// - `env`: The deployment environment (e.g., "production", "dev").
    /// - `level`: The severity of the log (automatically extracted from the message).
    ///
    /// # Default Values:
    /// - **Workers:** [`LOKI_DEFAULT_WORKERS`]
    /// - **Connection timeout**: [`LOKI_DEFAULT_CONNECTION_TIMEOUT`]
    /// - **Request timeout**: [`LOKI_DEFAULT_REQUEST_TIMEOUT`]
    /// - **Maximum retries**: [`LOKI_DEFAULT_RETRIES`]
    pub fn with_labels<S1, S2, S3, S4>(url: S1, app: S3, job: S2, env: S4) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        let mut url = url.into();
        if !url.ends_with('/') {
            url.push('/');
        }

        Self {
            url,
            app: app.into(),
            job: job.into(),
            env: env.into(),
            basic_auth: None,
            bearer_auth: None,
            connection_timeout: LOKI_DEFAULT_CONNECTION_TIMEOUT,
            request_timeout: LOKI_DEFAULT_REQUEST_TIMEOUT,
            max_retries: LOKI_DEFAULT_RETRIES,
            workers: LOKI_DEFAULT_WORKERS,
        }
    }

    /// Returns the destination Loki base URL.
    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// Returns the value of the `app` label.
    pub fn get_app(&self) -> &str {
        &self.app
    }

    /// Returns the value of the `job` label.
    pub fn get_job(&self) -> &str {
        &self.job
    }

    /// Returns the value of the `env` label.
    pub fn get_env(&self) -> &str {
        &self.env
    }

    /// Returns the Basic Authentication credentials if configured.
    pub fn get_basic_auth(&self) -> Option<&BasicAuth> {
        self.basic_auth.as_ref()
    }

    /// Returns the Bearer Token if configured.
    pub fn get_bearer_auth(&self) -> Option<&str> {
        self.bearer_auth.as_ref().map(|auth| auth.as_str())
    }

    /// Returns the connection timeout duration.
    pub fn get_connection_timeout(&self) -> Duration {
        self.connection_timeout
    }

    /// Returns the request timeout duration.
    pub fn get_request_timeout(&self) -> Duration {
        self.request_timeout
    }

    /// Returns the number of background worker threads requested for this service.
    pub fn get_workers(&self) -> usize {
        self.workers
    }

    /// Returns the maximum number of teries allowed for this service.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Sets the destination Loki base URL (e.g., `http://localhost:3100`).
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        let mut url = url.into();
        if !url.ends_with('/') {
            url.push('/');
        }
        self.url = url;
        self
    }

    /// Sets the `app` label to identify this specific application instance.
    pub fn app<S: Into<String>>(mut self, app: S) -> Self {
        self.app = app.into();
        self
    }

    /// Sets the `job` label used by Loki for indexing.
    pub fn job<S: Into<String>>(mut self, job: S) -> Self {
        self.job = job.into();
        self
    }

    /// Sets the `env` label used by Loki for indexing.
    pub fn env<S: Into<String>>(mut self, env: S) -> Self {
        self.env = env.into();
        self
    }

    /// Configures the number of parallel workers that should process logs for this service.
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Configures the number of maximum retries that the process should be attempted.
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Enables Basic Authentication for the Loki connection.
    ///
    /// # Arguments
    /// * `basic_auth` [Basic auth][`BasicAuth`] object representing the login credentials.
    pub fn basic_auth<BA>(mut self, basic_auth: Option<BA>) -> Self
    where
        BA: Into<BasicAuth>,
    {
        self.basic_auth = basic_auth.map(|auth| auth.into());
        self
    }

    /// Enables Bearer Token authentication (e.g., JWT).
    pub fn bearer_auth<S>(mut self, token: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.bearer_auth = token.map(|token| token.into());
        self
    }

    /// Sets the connection timeout to try to log in loki.
    pub fn connection_timeout<D: Into<Duration>>(mut self, connection_timeout: D) -> Self {
        self.connection_timeout = connection_timeout.into();
        self
    }

    /// Sets the request timeout to try to log in loki.
    pub fn request_timeout<S: Into<Duration>>(mut self, request_timeout: S) -> Self {
        self.request_timeout = request_timeout.into();
        self
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("LokiConfig");

        // Show normal fields
        d.field("url", &self.url)
            .field("app", &self.app)
            .field("job", &self.job)
            .field("env", &self.env);

        // Conditional display for secrets
        #[cfg(debug_assertions)]
        {
            d.field("basic_auth", &self.basic_auth)
                .field("bearer_auth", &self.bearer_auth);
        }

        #[cfg(not(debug_assertions))]
        {
            // In release, we just show that a value exists without revealing it
            let auth_status = if self.bearer_auth.is_some() || self.basic_auth.is_some() {
                "REDACTED (Set)"
            } else {
                "None"
            };
            d.field("auth", &auth_status);
        }

        d.field("connection_timeout", &self.connection_timeout)
            .field("request_timeout", &self.request_timeout)
            .field("max_retries", &self.max_retries)
            .field("workers", &self.workers)
            .finish()
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Count your fields: we have 10 top-level keys
        let mut state = serializer.serialize_struct("LokiConfig", 10)?;

        state.serialize_field("url", &self.url)?;
        state.serialize_field("app", &self.app)?;
        state.serialize_field("job", &self.job)?;
        state.serialize_field("env", &self.env)?;

        if let Some(basic_auth) = &self.basic_auth {
            state.serialize_field("basic_auth", &basic_auth)?;
        }

        if let Some(bearer_auth) = &self.bearer_auth {
            state.serialize_field("bearer_auth", bearer_auth)?;
        }

        // Connect Timeout
        state.serialize_field(
            "connection_timeout",
            &FlexibleDuration::from_duration(self.connection_timeout),
        )?;

        // Request Timeout
        state.serialize_field(
            "request_timeout",
            &FlexibleDuration::from_duration(self.connection_timeout),
        )?;

        state.serialize_field("max_retries", &self.max_retries)?;
        state.serialize_field("workers", &self.workers)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct LokiConfigHelper {
            url: String,
            app: Option<String>,
            job: Option<String>,
            env: Option<String>,
            basic_auth: Option<BasicAuth>,
            bearer_auth: Option<String>,
            connection_timeout: Option<FlexibleDuration>,
            request_timeout: Option<FlexibleDuration>,
            max_retries: Option<usize>,
            workers: Option<usize>,
        }

        let mut data = LokiConfigHelper::deserialize(deserializer)?;

        // Fix url
        if !data.url.ends_with('/') {
            data.url.push('/');
        }

        Ok(Config {
            url: data.url,
            app: data.app.unwrap_or(LOKI_DEFAULT_APP.to_string()),
            job: data.job.unwrap_or(LOKI_DEFAULT_JOB.to_string()),
            env: data.env.unwrap_or(LOKI_DEFAULT_ENV.to_string()),
            basic_auth: data.basic_auth,
            bearer_auth: data.bearer_auth,
            connection_timeout: data
                .connection_timeout
                .map(|timeout| timeout.as_duration())
                .unwrap_or(Ok(LOKI_DEFAULT_CONNECTION_TIMEOUT))?,
            request_timeout: data
                .request_timeout
                .map(|timeout| timeout.as_duration())
                .unwrap_or(Ok(LOKI_DEFAULT_REQUEST_TIMEOUT))?,
            workers: data.workers.unwrap_or(LOKI_DEFAULT_WORKERS),
            max_retries: data.max_retries.unwrap_or(LOKI_DEFAULT_RETRIES),
        })
    }
}
