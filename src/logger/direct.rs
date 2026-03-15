// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::{LoggerImpl, LoggerStatus, Message, Service};
use std::any::Any;

/// A synchronous logging implementation that performs immediate writes.
///
/// [`SyncLogger`][`Direct`] blocks the current thread until the [`Service`] successfully
/// processes the [`Message`] or the `max_retries` threshold is reached.
///
/// Because it implements [`LoggerImpl`], it can be wrapped in the primary
/// [`Logger`][`crate::Logger`] struct and shared across threads.
pub struct Direct {
    service: Box<dyn Service + Send + Sync>,
    max_retries: usize,
}

impl Direct {
    /// Creates a new [`DirectLogger`][`Direct`] with a specified [backend service][`Service`] and retry policy.
    ///
    /// # Arguments
    /// * `service` - The backend execution logic (e.g., File, Console).
    /// * `max_retries` - How many additional times to try if the first attempt fails.
    pub fn new(service: Box<dyn Service + Send + Sync>, max_retries: usize) -> Box<Self> {
        Box::new(Self {
            service,
            max_retries,
        })
    }

    /// Returns the base service
    pub fn get_service(&self) -> &dyn Service {
        self.service.as_ref()
    }
}

impl LoggerImpl for Direct {
    /// Delegates the health check to the underlying [Service].
    fn status(&self) -> LoggerStatus {
        self.service.status()
    }

    /// Attempts to log the message synchronously.
    ///
    /// If the first attempt fails, it will retry up to `max_retries` times.
    /// Since the high-level API is fire-and-forget, failures after all
    /// retries are exhausted are currently dropped silently to maintain
    /// the [`LoggerImpl`] contract.
    ///
    /// If all retries fail the fallback is used.
    fn log(&self, message: Message) {
        // Initial attempt
        let mut result = self.service.work(&message);
        if result.is_ok() {
            return;
        }

        // Retry logic
        for _ in 0..self.max_retries {
            result = self.service.work(&message);
            if result.is_ok() {
                return;
            }
        }

        self.service.fallback(&result.unwrap_err(), &message);
    }

    /// Enables runtime downcasting to [`DirectLogger`][`Direct`].
    fn as_any(&self) -> &dyn Any {
        self
    }
}
