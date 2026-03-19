// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "aws")))]

/// A container for a log message and its associated CloudWatch metadata.
///
/// This struct pairs a generic [`crate::Message`] with a Unix timestamp (in milliseconds).
/// By capturing the timestamp at the moment of creation, the logger ensures that
/// events remain chronologically sortable even when buffered or processed
/// asynchronously across multiple threads.
///
/// # CloudWatch Requirements
/// CloudWatch Logs requires events within a single `PutLogEvents` batch to be
/// sorted by timestamp in ascending order. While timestamps may be identical for
/// high-frequency logs, the order of entries in the transmission determines
/// their display sequence.
pub struct Message {
    /// The original log content, including its level and payload.
    pub message: crate::Message,
    /// The Unix epoch timestamp in milliseconds.
    ///
    /// Represented as an [`i64`] to comply with the AWS SDK requirements.
    pub timestamp: i64,
}