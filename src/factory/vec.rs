// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::{Concurrency, DirectLogger, Logger, QueuedLogger, service};

/// A factory for creating loggers that store messages in an in-memory [`Vec`].
///
/// The `Vector` factory is ideal for unit testing, allowing you to capture
/// logs in a thread-safe list and verify them later. Unlike string-based loggers,
/// this preserves the structured [`Message`][`crate::Message`] data.
///
/// ### Default Configuration
/// - **Capacity**: `1024` (Pre-allocated slots for log messages).
/// - **Max Retries**: `3` (Attempts to re-send if the buffer is temporarily locked).
/// - **Worker Count**: `1` (Ensures sequential insertion order in async mode).
pub struct Vector {
    capacity: usize,
    max_retries: usize,
    worker_count: usize,
}

impl Vector {
    /// Creates a new `Vector` factory with explicit settings.
    pub fn new(capacity: usize, max_retries: usize, worker_count: usize) -> Self {
        Self {
            capacity,
            max_retries,
            worker_count,
        }
    }

    /// Returns the initial pre-allocated capacity for the message buffer.
    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the currently configured maximum retry attempts.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Returns the currently configured background worker count.
    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    /// Sets the initial capacity of the internal vector.
    ///
    /// Pre-allocating capacity helps avoid expensive reallocations during 
    /// high-frequency logging events.
    pub fn capacity(self, capacity: usize) -> Self {
        Self { capacity, ..self }
    }

    /// Updates the maximum number of retry attempts for the buffer.
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Updates the background worker count. 
    ///
    /// Note: Using multiple workers with a vector may result in logs being 
    /// inserted out of chronological order.
    pub fn worker_count(self, worker_count: usize) -> Self {
        Self {
            worker_count,
            ..self
        }
    }

    /// Finalizes the builder and returns a high-level [`Logger`].
    pub fn build(self, concurrency: Concurrency) -> Logger {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct()),
            Concurrency::Async => Logger::new(self.build_impl_queued()),
        }
    }

    /// Builds the underlying [`DirectLogger`] implementation.
    pub fn build_impl_direct(self) -> Box<DirectLogger> {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service(), max_retries)
    }

    /// Builds the underlying [`QueuedLogger`] implementation.
    pub fn build_impl_queued(self) -> Box<QueuedLogger> {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(self.build_service(), max_retries, worker_count)
    }

    /// Internal helper to construct the [`service::Vector`] instance.
    pub fn build_service(self) -> Box<service::Vector> {
        service::Vector::new(self.capacity)
    }
}

impl Default for Vector {
    /// Provides sensible defaults for in-memory message capturing.
    ///
    /// - **capacity**: `1024`
    /// - **max_retries**: `3`
    /// - **worker_count**: `1`
    fn default() -> Self {
        Self {
            capacity: 1024usize,
            max_retries: 3,
            worker_count: 1,
        }
    }
}