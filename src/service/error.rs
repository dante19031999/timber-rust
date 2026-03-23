// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::serror::FeatureDisabledError;
#[cfg(feature = "network")]
use crate::service::HttpError;
use std::fmt::{Display, Formatter};
use std::sync::{MutexGuard, PoisonError};

/// Represents the possible failure modes for a [`Service`][`crate::Service`].
///
/// This enum wraps specific backend errors into a unified type, allowing
/// the [`LoggerImpl`][`crate::LoggerImpl`] to handle various failure scenarios consistently.
#[derive(Debug)]
pub enum ServiceError {
    /// Errors related to the file system or standard I/O operations.
    ///
    /// This is typically triggered by permission issues, missing files, or
    /// interrupted streams during log rotation or local storage.
    Io(std::io::Error),
    /// Errors occurring during string formatting operations.
    ///
    /// Usually happens when writing to a buffer or formatting complex
    /// log structures for output.
    Fmt(std::fmt::Error),
    /// Errors occurring during JSON serialization or deserialization.
    ///
    /// Triggered when the log message cannot be converted to valid JSON
    /// or when a configuration file has an invalid JSON format.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    Json(serde_json::Error),
    /// Errors occurring during network requests (e.g., HTTP).
    ///
    /// This includes connection timeouts, DNS resolution failures, or
    /// transport-level issues when sending logs to a remote server.
    #[cfg(feature = "network")]
    #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
    Network(reqwest::Error),
    /// Errors returned by the HTTP server or protocol.
    ///
    /// Unlike [`ServiceError::Network`], this variant represents valid HTTP
    /// responses that indicate a failure (e.g., 4xx or 5xx status codes).
    #[cfg(feature = "network")]
    #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
    Http(HttpError),
    /// A synchronization primitive has been "poisoned".
    ///
    /// This occurs when a thread panics while holding a [`std::sync::Mutex`]
    /// or [`std::sync::RwLock`]. The data protected by the lock might be
    /// in an inconsistent state.
    LockPoisoned,
    /// Failed to map the raw configuration into the internal settings structure.
    ///
    /// This indicates that the configuration exists and is syntactically correct,
    /// but contains invalid values or missing required fields.
    ConfigDeserialization,
    /// The requested operation requires a feature that is not currently enabled.
    ///
    /// Check the [`FeatureDisabledError`] for details on which feature is missing
    /// and how to enable it in your `Cargo.toml`.
    FeatureDisabled(FeatureDisabledError),
    /// A catch-all for errors not covered by the specific variants.
    ///
    /// Use this for wrapping third-party errors that don't justify
    /// a dedicated variant or for rare, unexpected conditions.
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}
impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Fmt(err) => write!(f, "Fmt error: {}", err),
            #[cfg(feature = "json")]
            #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
            Self::Json(err) => write!(f, "JSON error: {}", err),
            #[cfg(feature = "network")]
            #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
            Self::Network(err) => write!(f, "Network error: {}", err),
            #[cfg(feature = "network")]
            #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
            Self::Http(err) => write!(f, "Http error: {}", err.status_code()),
            Self::LockPoisoned => write!(f, "Lock poisoned"),
            Self::ConfigDeserialization => write!(f, "Config deserialization error"),
            Self::FeatureDisabled(err) => write!(f, "{}", err),
            Self::Unknown(err) => write!(f, "Unknown service error: {}", err),
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Fmt(err) => Some(err),
            #[cfg(feature = "json")]
            #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
            Self::Json(err) => Some(err),
            #[cfg(feature = "network")]
            #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
            Self::Network(err) => Some(err),
            #[cfg(feature = "network")]
            #[cfg_attr(docsrs, doc(cfg(feature = "network")))]
            Self::Http(err) => Some(err),
            Self::LockPoisoned => None,
            Self::ConfigDeserialization => None,
            Self::FeatureDisabled(err) => Some(err),
            Self::Unknown(err) => Some(err.as_ref()),
        }
    }
}

impl From<std::fmt::Error> for ServiceError {
    fn from(err: std::fmt::Error) -> Self {
        ServiceError::Fmt(err)
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> Self {
        ServiceError::Io(err)
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for ServiceError {
    fn from(_err: PoisonError<MutexGuard<'_, T>>) -> Self {
        ServiceError::LockPoisoned
    }
}
