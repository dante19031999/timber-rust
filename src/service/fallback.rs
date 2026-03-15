// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::Message;
use crate::service::ServiceError;
use chrono::{SecondsFormat, Utc};

/// A strategy for handling log delivery failures.
///
/// The `Fallback` trait defines how the system should react when a log service
/// is unable to process a [message][`Message`] after exhausting all retry attempts.
///
/// ### Use Cases
/// - **Local Logging**: Writing failed logs to `stdout`, `stderr`, or a local file.
/// - **Alerting**: Triggering a secondary notification system if critical logs are lost.
/// - **Buffering**: Storing failed [messages][`Message`] in a persistent queue for later recovery.
///
/// ### Thread Safety
/// Since the worker thread calls this trait's method, implementations must be
/// thread-safe if they involve shared state.
pub trait Fallback {
    /// Handles a [message][`Message`] that could not be delivered to the primary service.
    ///
    /// This method is invoked by the background worker when a [`ServiceError`] occurs
    /// that cannot be recovered through retries.
    ///
    /// # Default Implementation
    ///
    /// The default implementation prints a formatted critical error message to **stderr**.
    /// It includes:
    /// 1. The specific [`ServiceError`] description.
    /// 2. An RFC3339-compliant timestamp with nanosecond precision.
    /// 3. The log level and the original message content.
    ///
    /// This ensures that even if the remote collector is unreachable,
    /// the logs are preserved in the system's standard error stream.
    ///
    /// # Parameters
    /// - `error`: The specific error that caused the delivery failure.
    /// - `message`: The original log [message][`Message`] that failed to be delivered.
    fn fallback(&self, error: &ServiceError, message: &Message) {
        let now: chrono::DateTime<Utc> = message.instant().into();
        eprintln!(
            "[CRITICAL LOGGER FAILURE] {}: {} [ {} ] {}",
            error,
            now.to_rfc3339_opts(SecondsFormat::Nanos, true),
            message.level(),
            message.content()
        );
    }
}
