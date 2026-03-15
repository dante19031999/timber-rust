// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "loki")]
#![cfg_attr(docsrs, doc(cfg(feature = "loki")))]

use reqwest::blocking::Client;
use crate::service::loki::config::Config;

/// Shared state and configuration for the Loki service.
///
/// This struct is typically wrapped in an [`Arc`][`std::sync::Arc`] to be shared between the
/// frontend logger and the background worker thread.
pub struct Data {
    /// The pre-configured HTTP client (handles connection pooling).
    pub client: Client,
    /// User-defined configuration (retries, timeouts, labels).
    pub config: Config,
    /// The full computed URL (e.g., `http://host:port/loki/api/v1/push`).
    pub post_url: String,
}