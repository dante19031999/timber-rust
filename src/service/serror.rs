// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// Represents an error returned by a remote HTTP service.
///
/// This struct specifically captures failures where the connection was successful,
/// but the server returned a non-success status code (e.g., 4xx or 5xx).
///
/// ### Feature Requirement
/// This type is only available when the **`network`** feature is enabled in your `Cargo.toml`.
///
/// ### Common Scenarios
/// - **429 Too Many Requests**: Rate limiting is active.
/// - **400 Bad Request**: Often indicates "Out of Order" entries in service like Loki.
/// - **503 Service Unavailable**: The remote logging endpoint is temporarily down.
#[cfg(feature = "network")]
pub struct HttpError {
    /// The 3-digit HTTP response status code.
    status_code: u16,
}

#[cfg(feature = "network")]
impl HttpError {
    /// Creates a new [`HttpError`] from a raw status code.
    ///
    /// # Parameters
    /// - `status_code`: The [u16] representation of the HTTP response (e.g., 404).
    ///
    /// # Example
    /// ```ignore
    /// // Requires: cargo test --features network
    /// let error = HttpError::new(429);
    /// assert_eq!(error.status_code(), 429);
    /// ```
    pub fn new(status_code: u16) -> HttpError {
        HttpError { status_code }
    }

    /// Returns the HTTP status code associated with this error.
    ///
    /// This is useful in `fallback` logic to determine if the error
    /// is recoverable or if it requires a specific alerting path.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
}

#[cfg(feature = "network")]
impl Debug for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Http error: {}", self.status_code)
    }
}

#[cfg(feature = "network")]
impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Http error: {}", self.status_code)
    }
}

#[cfg(feature = "network")]
impl std::error::Error for HttpError {}

/// An error indicating that a requested functionality is unavailable because its
/// corresponding crate feature was not enabled at compile time.
///
/// This error typically occurs when a configuration file attempts to use a service
/// (like a network logger) in a build that was optimized for local-use only.
///
/// ### How to resolve
/// If you encounter this error, ensure the required feature is enabled in your `Cargo.toml`:
/// ```toml
/// [dependencies]
/// my_logger = { version = "Your version here", features = ["Feature name here"] }
/// ```
pub struct FeatureDisabledError {
    name: String,
}

impl FeatureDisabledError {
    /// Creates a new error instance for the specified feature name.
    ///
    /// # Example
    /// ```
    /// # use timber_rust::service::FeatureDisabledError;
    /// let err = FeatureDisabledError::new("loki".to_string());
    /// ```
    pub fn new(name: String) -> Self {
        FeatureDisabledError { name }
    }

    /// Returns the name of the feature that caused the error.
    ///
    /// This can be used to programmatically decide whether to fallback
    /// to a different service.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Debug for FeatureDisabledError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Debug output often used for developer logs
        write!(f, "FeatureDisabledError: '{}' is not compiled into this binary", self.name)
    }
}

impl Display for FeatureDisabledError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // User-friendly message
        write!(f, "The feature '{}' is currently disabled. Please recompile with the appropriate feature flag.", self.name)
    }
}

impl std::error::Error for FeatureDisabledError {}