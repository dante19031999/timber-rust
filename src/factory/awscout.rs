// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "awscout")]
#![cfg_attr(docsrs, doc(cfg(feature = "awscout")))]

use crate::{Concurrency, DirectLogger, Logger, QueuedLogger, service};

/// A factory for creating various AWS CloudWatch logging implementations.
///
/// This factory provides entry points for three primary CloudWatch logging strategies:
/// 1. **Config-based**: Explicitly providing AWS credentials and region.
/// 2. **Env-based**: Automatic credential discovery using the standard AWS environment variables.
/// 3. **Cout-based**: JSON-formatted stdout logging for Lambda, ECS, and Fargate.
pub struct CloudWatch {}

impl CloudWatch {
    /// Begins building a CloudWatch `stdout` JSON logger.
    ///
    /// This is recommended for AWS Lambda and containerized services where the
    /// log driver handles the actual AWS API interaction.
    pub fn cout(self) -> CloudWatchCout {
        CloudWatchCout::default()
    }
}

/// A builder state for the high-performance JSON-to-stdout logging service.
///
/// Defaults to 3 retries and 1 background worker to ensure ordered log ingestion.
pub struct CloudWatchCout {
    max_retries: usize,
    worker_count: usize,
}

impl CloudWatchCout {
    /// Creates a new Cout factory with specific retry and worker settings.
    pub fn new(max_retries: usize, worker_count: usize) -> Self {
        Self {
            max_retries,
            worker_count,
        }
    }

    /// Returns the configured background worker count.
    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    /// Returns the configured maximum retry attempts.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Sets the number of background workers (only applicable for [`Concurrency::Async`]).
    pub fn worker_count(self, worker_count: usize) -> CloudWatchCout {
        CloudWatchCout {
            worker_count,
            ..self
        }
    }

    /// Sets the maximum retry attempts if the standard output write fails.
    pub fn max_retries(self, max_retries: usize) -> CloudWatchCout {
        CloudWatchCout {
            max_retries,
            ..self
        }
    }

    /// Finalizes the logger using the specified [`Concurrency`] model.
    pub fn build(self, concurrency: Concurrency) -> Logger {
        match concurrency {
            Concurrency::Sync => self.build_direct(),
            Concurrency::Async => self.build_queued(),
        }
    }

    /// Shortcut for building a synchronous [`DirectLogger`].
    pub fn build_direct(self) -> Logger {
        Logger::new(self.build_impl_direct())
    }

    /// Shortcut for building an asynchronous [`QueuedLogger`].
    pub fn build_queued(self) -> Logger {
        Logger::new(self.build_impl_queued())
    }

    /// Builds the underlying [`DirectLogger`] implementation.
    pub fn build_impl_direct(self) -> Box<DirectLogger> {
        DirectLogger::new(service::CloudWatchCout::new(), self.max_retries)
    }

    /// Builds the underlying [`QueuedLogger`] implementation.
    pub fn build_impl_queued(self) -> Box<QueuedLogger> {
        QueuedLogger::new(
            service::CloudWatchCout::new(),
            self.max_retries,
            self.worker_count,
        )
    }
}

impl Default for CloudWatchCout {
    /// Provides default settings: 3 retries and 1 worker thread.
    fn default() -> Self {
        CloudWatchCout {
            worker_count: 1,
            max_retries: 3,
        }
    }
}
