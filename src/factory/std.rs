use crate::service::{StandardWriteMessageFormatter, WriteMessageFormatter};
use crate::{Concurrency, DirectLogger, Logger, QueuedLogger, service};

/// A factory for creating loggers that write directly to the standard output (stdout).
///
/// `CoutWrite` provides a high-level builder for console logging. It is the most
/// common choice for CLI applications and containerized environments (like Docker or K8s)
/// where logs are expected to be captured from the process's stdout stream.
///
/// ### Default Configuration
/// - **Max Retries**: `3` (Standard resilience against transient I/O pressure).
/// - **Worker Count**: `1` (Ensures logs appear in chronological order).
pub struct CoutWrite {
    max_retries: usize,
    worker_count: usize,
}

impl CoutWrite {
    /// Returns the currently configured maximum retry attempts.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Returns the currently configured background worker count.
    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    /// Updates the maximum number of retry attempts for the console service.
    ///
    /// This is useful if the terminal or pipe is under heavy load and requires
    /// multiple attempts to flush data.
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Updates the background worker count for asynchronous console logging.
    ///
    /// **Note:** Using a worker count of 1 is highly recommended for console output
    /// to prevent log lines from overlapping or appearing out of order.
    pub fn worker_count(self, worker_count: usize) -> Self {
        Self {
            worker_count,
            ..self
        }
    }

    /// Finalizes the logger using the [`StandardWriteMessageFormatter`] and a [`Concurrency`] strategy.
    pub fn build(self, concurrency: Concurrency) -> Logger {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct()),
            Concurrency::Async => Logger::new(self.build_impl_queued()),
        }
    }

    /// Finalizes the logger using a custom [`WriteMessageFormatter`] and a [`Concurrency`] strategy.
    ///
    /// Use this to apply custom styling, colors, or structured formats (like JSON)
    /// to the console output.
    pub fn build_with_formatter<MF>(self, concurrency: Concurrency, formatter: MF) -> Logger
    where
        MF: WriteMessageFormatter + 'static,
    {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct_with_formatter(formatter)),
            Concurrency::Async => Logger::new(self.build_impl_queued_with_formatter(formatter)),
        }
    }

    /// Builds the underlying [`DirectLogger`] implementation for stdout.
    pub fn build_impl_direct(self) -> Box<DirectLogger> {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service(), max_retries)
    }

    /// Builds the underlying [`QueuedLogger`] implementation for stdout.
    pub fn build_impl_queued(self) -> Box<QueuedLogger> {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(self.build_service(), max_retries, worker_count)
    }

    /// Builds a [`DirectLogger`] with a specific formatter.
    pub fn build_impl_direct_with_formatter<MF>(self, formatter: MF) -> Box<DirectLogger>
    where
        MF: WriteMessageFormatter + 'static,
    {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service_with_formatter(formatter), max_retries)
    }

    /// Builds a [`QueuedLogger`] with a specific formatter.
    pub fn build_impl_queued_with_formatter<MF>(self, formatter: MF) -> Box<QueuedLogger>
    where
        MF: WriteMessageFormatter + 'static,
    {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(
            self.build_service_with_formatter(formatter),
            max_retries,
            worker_count,
        )
    }

    /// Internal helper to construct the [`service::CoutWrite`] service with a custom formatter.
    pub fn build_service_with_formatter<MF>(self, formatter: MF) -> Box<service::CoutWrite<MF>>
    where
        MF: WriteMessageFormatter + 'static,
    {
        service::CoutWrite::with_formatter(formatter)
    }

    /// Internal helper to construct the [`service::CoutWrite`] service with the standard formatter.
    pub fn build_service(self) -> Box<service::CoutWrite<StandardWriteMessageFormatter>> {
        service::CoutWrite::new()
    }
}

impl Default for CoutWrite {
    /// Provides sensible defaults for console logging.
    ///
    /// - **max_retries**: `3`
    /// - **worker_count**: `1` (Preserves sequential terminal output).
    fn default() -> Self {
        Self {
            max_retries: 3,
            worker_count: 1,
        }
    }
}

/// A factory for creating loggers that write directly to the standard error (stderr).
///
/// `CerrWrite` provides a high-level builder for console logging. This is the
/// recommended choice for professional CLI applications, as it ensures that
/// diagnostic information (logs) does not interfere with the primary data
/// stream sent to stdout.
///
/// ### Default Configuration
/// - **Max Retries**: `3` (Resilience against transient I/O blocks or terminal pressure).
/// - **Worker Count**: `1` (Ensures logs appear in strict chronological order).
pub struct CerrWrite {
    max_retries: usize,
    worker_count: usize,
}

impl CerrWrite {
    /// Returns the currently configured maximum retry attempts.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Returns the currently configured background worker count.
    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    /// Updates the maximum number of retry attempts for the stderr service.
    ///
    /// Useful if the error stream is being redirected to a file or pipe that
    /// may experience intermittent congestion.
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Updates the background worker count for asynchronous error logging.
    ///
    /// **Note:** Using a worker count of 1 is strongly advised for stderr
    /// to prevent terminal interleaving and preserve message order.
    pub fn worker_count(self, worker_count: usize) -> Self {
        Self {
            worker_count,
            ..self
        }
    }

    /// Finalizes the logger using the [`StandardWriteMessageFormatter`] and a [`Concurrency`] strategy.
    pub fn build(self, concurrency: Concurrency) -> Logger {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct()),
            Concurrency::Async => Logger::new(self.build_impl_queued()),
        }
    }

    /// Finalizes the logger using a custom [`WriteMessageFormatter`] and a [`Concurrency`] strategy.
    ///
    /// This is frequently used to apply "error red" ANSI styling or specialized
    /// headers specifically for diagnostic output.
    pub fn build_with_formatter<MF>(self, concurrency: Concurrency, formatter: MF) -> Logger
    where
        MF: WriteMessageFormatter + 'static,
    {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct_with_formatter(formatter)),
            Concurrency::Async => Logger::new(self.build_impl_queued_with_formatter(formatter)),
        }
    }

    /// Builds the underlying [`DirectLogger`] implementation for stderr.
    pub fn build_impl_direct(self) -> Box<DirectLogger> {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service(), max_retries)
    }

    /// Builds the underlying [`QueuedLogger`] implementation for stderr.
    pub fn build_impl_queued(self) -> Box<QueuedLogger> {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(self.build_service(), max_retries, worker_count)
    }

    /// Builds a [`DirectLogger`] with a specific formatter for stderr.
    pub fn build_impl_direct_with_formatter<MF>(self, formatter: MF) -> Box<DirectLogger>
    where
        MF: WriteMessageFormatter + 'static,
    {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service_with_formatter(formatter), max_retries)
    }

    /// Builds a [`QueuedLogger`] with a specific formatter for stderr.
    pub fn build_impl_queued_with_formatter<MF>(self, formatter: MF) -> Box<QueuedLogger>
    where
        MF: WriteMessageFormatter + 'static,
    {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(
            self.build_service_with_formatter(formatter),
            max_retries,
            worker_count,
        )
    }

    /// Internal helper to construct the [`service::CerrWrite`] service with a custom formatter.
    pub fn build_service_with_formatter<MF>(self, formatter: MF) -> Box<service::CerrWrite<MF>>
    where
        MF: WriteMessageFormatter + 'static,
    {
        service::CerrWrite::with_formatter(formatter)
    }

    /// Internal helper to construct the [`service::CerrWrite`] service with the standard formatter.
    pub fn build_service(self) -> Box<service::CerrWrite<StandardWriteMessageFormatter>> {
        service::CerrWrite::new()
    }
}

impl Default for CerrWrite {
    /// Provides sensible defaults for error console logging.
    ///
    /// - **max_retries**: `3`
    /// - **worker_count**: `1` (Preserves sequential terminal output).
    fn default() -> Self {
        Self {
            max_retries: 3,
            worker_count: 1,
        }
    }
}
