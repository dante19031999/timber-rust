// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "aws")))]

use serde::ser::SerializeStruct;
use std::time::SystemTime;

/// Configuration for the AWS CloudWatch service.
///
/// This struct holds all necessary credentials and identifiers to authenticate
/// and send logs to a specific CloudWatch Log Group.
///
/// # Note
/// While this configuration is explicit, the underlying SDK can often
/// fall back to environment variables if certain fields are left as default.
#[derive(Clone)]
pub struct Config {
    /// AWS Access Key ID used for authentication.
    access_key_id: String,
    /// AWS Secret Access Key. This field is sensitive and hidden in Debug logs.
    access_key_secret: String,
    /// Optional session token for temporary credentials (STS).
    session_token: Option<String>,
    /// Optional expiration timestamp for the credentials in seconds.
    expires_in: Option<SystemTime>,
    /// The name of the CloudWatch Log Group where logs will be sent.
    log_group: String,
    /// The AWS Region (e.g., "us-east-1").
    region: String,
    /// Identifier for the provider or application generating the logs.
    provider: &'static str,
}

impl Config {
    /// Creates a new `Config` instance with required parameters.
    ///
    /// # Arguments
    /// * `access_key_id` - The AWS Access Key.
    /// * `access_key_secret` - The AWS Secret Key.
    /// * `log_group` - The target CloudWatch Log Group name.
    /// * `region` - The AWS region string.
    ///
    /// # Defaults:
    /// provider: "timber-rust"
    ///
    /// # Example
    /// ```rust
    /// let config = Config::new("AKIA...", "secret", "my-logs", "us-east-1");
    /// ```
    pub fn new<S1, S2, S3, S4>(
        access_key_id: S1,
        access_key_secret: S2,
        log_group: S3,
        region: S4,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        Config {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            session_token: None,
            expires_in: None,
            log_group: log_group.into(),
            region: region.into(),
            provider: "timber-rust",
        }
    }

    /// Returns the AWS Access Key ID.
    pub fn get_access_key_id(&self) -> &str {
        &self.access_key_id
    }

    /// Returns the AWS Secret Access Key.
    pub fn get_access_key_secret(&self) -> &str {
        &self.access_key_secret
    }

    /// Returns the AWS Session Token, if any.
    pub fn get_session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    /// Returns the expiration time of the credentials in seconds.
    pub fn get_expires_in(&self) -> Option<SystemTime> {
        self.expires_in
    }

    /// Returns the target CloudWatch Log Group name.
    pub fn get_log_group(&self) -> &str {
        &self.log_group
    }

    /// Returns the AWS Region string.
    pub fn get_region(&self) -> &str {
        &self.region
    }

    /// Returns the provider name string.
    pub fn get_provider(&self) -> &'static str {
        &self.provider
    }

    // --- Builder Pattern Methods ---

    /// Sets the AWS Access Key ID.
    pub fn access_key_id(mut self, v: impl Into<String>) -> Self {
        self.access_key_id = v.into();
        self
    }

    /// Sets the AWS Secret Access Key.
    pub fn access_key_secret(mut self, v: impl Into<String>) -> Self {
        self.access_key_secret = v.into();
        self
    }

    /// Sets an optional AWS Session Token.
    pub fn session_token(mut self, v: Option<impl Into<String>>) -> Self {
        self.session_token = v.map(|v| v.into());
        self
    }

    /// Sets the credential expiration time (in seconds).
    pub fn expires_in(mut self, v: Option<SystemTime>) -> Self {
        self.expires_in = v;
        self
    }

    /// Sets the target CloudWatch Log Group.
    pub fn log_group(mut self, v: impl Into<String>) -> Self {
        self.log_group = v.into();
        self
    }

    /// Sets the AWS Region.
    pub fn region(mut self, v: impl Into<String>) -> Self {
        self.region = v.into();
        self
    }

    /// Sets the provider name for this configuration.
    pub fn provider(mut self, v: &'static str) -> Self {
        self.provider = v;
        self
    }
}

impl std::fmt::Debug for Config {
    /// Formats the configuration for debugging purposes.
    /// Note that `access_key_secret` is masked for security.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Config");

        d.field("access_key_id", &self.access_key_id)
            .field("access_key_secret", &"***")
            .field("log_group", &self.log_group)
            .field("region", &self.region)
            .field("provider", &self.provider);

        #[cfg(debug_assertions)]
        {
            d.field("access_key_secret", &self.access_key_secret);
        }
        #[cfg(not(debug_assertions))]
        {
            d.field("access_key_secret", &"***");
        }

        d.finish()
    }
}

impl<'de> serde::Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ShadowConfig {
            access_key_id: String,
            access_key_secret: String,
            session_token: Option<String>,
            expires_in: Option<SystemTime>,
            log_group: String,
            region: String,
        }

        let helper = ShadowConfig::deserialize(deserializer)?;

        Ok(Config {
            access_key_id: helper.access_key_id,
            access_key_secret: helper.access_key_secret,
            session_token: helper.session_token,
            expires_in: helper.expires_in,
            log_group: helper.log_group,
            region: helper.region,
            provider: "timber-rust",
        })
    }
}

impl serde::Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("CloudWatchConfig", 10)?;
        state.serialize_field("access_key_id", &self.access_key_id)?;
        state.serialize_field("access_key_secret", &self.access_key_secret)?;
        state.serialize_field("session_token", &self.session_token)?;
        state.serialize_field("expires_in", &self.expires_in)?;
        state.serialize_field("log_group", &self.log_group)?;
        state.serialize_field("region", &self.region)?;
        state.end()
    }
}
