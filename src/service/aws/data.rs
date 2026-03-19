// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "aws")))]

use crate::service::aws::MessageFormatter;
use tokio::runtime::Runtime;

/// Shared state and configuration for the Loki service.
///
/// This struct is typically wrapped in an [`Arc`][`std::sync::Arc`] to be shared between the
/// frontend logger and the background worker thread.
pub struct Data {
    /// The pre-configured HTTP client (handles connection pooling).
    pub client: aws_sdk_cloudwatchlogs::Client,
    /// Internal Tokio runtime used to drive the asynchronous SDK.
    pub rt: Runtime,
    /// The target Log Group name in AWS CloudWatch.
    pub log_group: String,
    pub formatter: Box<dyn MessageFormatter + Send + Sync>,
}
