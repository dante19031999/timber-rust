// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::ServiceError;
use crate::service::fallback::Fallback;
use crate::{LoggerStatus, Message};
use std::any::Any;

/// The [`Service`] trait defines the core execution logic for a logging backend.
///
/// A [`Service`] acts as the internal "Worker" or "Driver" sitting behind a [`LoggerImpl`][`crate::LoggerImpl`].
/// While the logger handles the frontend API and message queuing, the [`Service`] is responsible
/// for the actual side effects, such as disk I/O, network transmission, or database insertion.
///
/// ### Hierarchy & Resilience
/// This trait extends [`Fallback`]. Every implementation must provide a mechanism to handle
/// messages that fail to be processed after the service's internal retry logic has been exhausted.
///
/// ### Threading Model
/// Implementations are typically executed within a dedicated background thread. Therefore,
/// any state held by a [`Service`] must be thread-safe (`Send + Sync`) to ensure
/// consistent behavior across the asynchronous boundary.
///
///
pub trait Service: Fallback {
    /// Returns the current operational `[LoggerStatus`] of the service.
    fn status(&self) -> LoggerStatus;

    /// Processes a single [Message].
    ///
    /// # Arguments
    /// * `msg` - A reference to the [Message] to be processed.
    ///
    /// # Returns
    /// * [`Ok(())`][`Ok`] if the message was handled successfully.
    /// * [`Err(ServiceError)`][`ServiceError`] if an error occurred.
    ///
    /// # Concurrency & Thread Safety
    /// This method takes `&self`, meaning it can be called concurrently from
    /// multiple threads if the [`LoggerImpl`][`crate::LoggerImpl`] allows it.
    ///
    /// > [!NOTE]
    /// > If the implementation requires modification of internal state (e.g.,
    /// > incrementing a counter or writing to a shared file), it must use
    /// > **Interior Mutability** (e.g., [`AtomicU64`][`std::sync::atomic::AtomicU64`], [`Mutex`][`std::sync::Mutex`], or [`mpsc`][`std::sync::mpsc::channel()`]).
    fn work(&self, msg: &Message) -> Result<(), ServiceError>;

    /// Returns a reference to the underlying type as [Any] for downcasting.
    fn as_any(&self) -> &dyn Any;
}
