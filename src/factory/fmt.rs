// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::{StandardWriteMessageFormatter, WriteMessageFormatter};
use crate::{service, Concurrency, DirectLogger, Logger, QueuedLogger};

/// A specialized factory for creating loggers that write to memory-oriented destinations.
///
/// The `FmtFactory` acts as the entry point for any output that implements [`std::fmt::Write`].
/// It allows you to configure global retry and threading policies before selecting a
/// specific output target (like a String writer).
///
/// ### Default Configuration
/// - **Max Retries**: 3 (Attempts to re-send if a write failure occurs).
/// - **Worker Count**: 1 (Single background thread to maintain message order).
pub struct FmtWrite {
    max_retries: usize,
    worker_count: usize,
}

/// A concrete builder state for a specific writer type `W`.
///
/// Once a writer is provided (via [`FmtWrite::string`], etc.),
/// this struct allows you to finalize the logger by choosing a [`Concurrency`] model.
pub struct TypedFmtWrite<W>
where
    W: std::fmt::Write + Send + Sync + 'static,
{
    writer: W,
    max_retries: usize,
    worker_count: usize,
}

/// A concrete builder state for a specific boxed writer type `W`.
///
/// Once a writer is provided (via [`FmtWrite::boxed`], etc.),
/// this struct allows you to finalize the logger by choosing a [`Concurrency`] model.
pub struct BoxedFmtWrite {
    writer: Box<dyn std::fmt::Write + Send + Sync>,
    max_retries: usize,
    worker_count: usize,
}

/// A pre-configured factory for logging directly to a fmt [`String`].
pub type StringFmt = TypedFmtWrite<String>;

impl FmtWrite {
    /// Specializes the factory to log directly to a [`String`].
    ///
    /// This is the most direct path for string logging. Each log entry is
    /// appended directly into an string.
    pub fn string(self) -> StringFmt {
        StringFmt {
            writer: String::with_capacity(1024),
            max_retries: self.max_retries,
            worker_count: self.worker_count,
        }
    }

    /// Specializes the factory to log directly to a [`String`].
    /// Allows to set a custom capacity.
    ///
    /// This is the most direct path for string logging. Each log entry is
    /// appended directly into an string.
    pub fn string_with_capacity(self, capacity: usize) -> StringFmt {
        StringFmt {
            writer: String::with_capacity(capacity),
            max_retries: self.max_retries,
            worker_count: self.worker_count,
        }
    }

    /// Specializes the factory to log directly to a [`String`].
    /// Allows to set an already existing string as the destiny.
    ///
    /// This is the most direct path for string logging. Each log entry is
    /// appended directly into an string.
    pub fn string_sullied(self, writer: String) -> StringFmt {
        StringFmt {
            writer,
            max_retries: self.max_retries,
            worker_count: self.worker_count,
        }
    }

    /// Specializes the factory for any generic type implementing [`std::fmt::Write`].
    pub fn writer<W>(self, writer: W) -> TypedFmtWrite<W>
    where
        W: std::fmt::Write + Send + Sync + 'static,
    {
        TypedFmtWrite {
            writer,
            max_retries: self.max_retries,
            worker_count: self.worker_count,
        }
    }

    /// Specializes the factory for any generic type implementing `Box<dyn std::fmt::Write + Send + Sync>`.
    pub fn boxed(self, writer: Box<dyn std::fmt::Write + Send + Sync>) -> BoxedFmtWrite {
        BoxedFmtWrite {
            writer,
            max_retries: self.max_retries,
            worker_count: self.worker_count,
        }
    }

    /// Configures the maximum number of retries for the resulting service.
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Configures the background worker thread count for asynchronous logging.
    pub fn worker_count(self, worker_count: usize) -> Self {
        Self {
            worker_count,
            ..self
        }
    }

    /// Finalizes the logger using a specific writer and a [`Concurrency`] strategy.
    /// This uses the default [`StandardWriteMessageFormatter`].
    pub fn build<W>(self, concurrency: Concurrency, writer: W) -> Logger
    where
        W: std::fmt::Write + Send + Sync + 'static,
    {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct(writer)),
            Concurrency::Async => Logger::new(self.build_impl_queued(writer)),
        }
    }

    /// Builds a [`DirectLogger`] implementation wrapped in a [`Box`].
    pub fn build_impl_direct<W>(self, writer: W) -> Box<DirectLogger>
    where
        W: std::fmt::Write + Send + Sync + 'static,
    {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service(writer), max_retries)
    }

    /// Builds a [`QueuedLogger`] implementation wrapped in a [`Box`].
    pub fn build_impl_queued<W>(self, writer: W) -> Box<QueuedLogger>
    where
        W: std::fmt::Write + Send + Sync + 'static,
    {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(self.build_service(writer), max_retries, worker_count)
    }

    /// Internal helper to construct the [`service::FmtWrite`] service with the standard formatter.
    pub fn build_service<W>(
        self,
        writer: W,
    ) -> Box<service::FmtWrite<W, StandardWriteMessageFormatter>>
    where
        W: std::fmt::Write + Send + Sync + 'static,
    {
        service::FmtWrite::new(writer)
    }

    /// Finalizes the logger using a custom formatter and a [`Concurrency`] strategy.
    pub fn build_with_formatter<W, MF>(
        self,
        concurrency: Concurrency,
        writer: W,
        formatter: MF,
    ) -> Logger
    where
        MF: WriteMessageFormatter + 'static,
        W: std::fmt::Write + Send + Sync + 'static,
    {
        match concurrency {
            Concurrency::Sync => {
                Logger::new(self.build_impl_direct_with_formatter(writer, formatter))
            }
            Concurrency::Async => {
                Logger::new(self.build_impl_queued_with_formatter(writer, formatter))
            }
        }
    }

    /// Builds a [`DirectLogger`] with a custom formatter.
    pub fn build_impl_direct_with_formatter<W, MF>(
        self,
        writer: W,
        formatter: MF,
    ) -> Box<DirectLogger>
    where
        MF: WriteMessageFormatter + 'static,
        W: std::fmt::Write + Send + Sync + 'static,
    {
        let max_retries = self.max_retries;
        DirectLogger::new(
            self.build_service_with_formatter(writer, formatter),
            max_retries,
        )
    }

    /// Builds a [`QueuedLogger`] with a custom formatter.
    pub fn build_impl_queued_with_formatter<W, MF>(
        self,
        writer: W,
        formatter: MF,
    ) -> Box<QueuedLogger>
    where
        MF: WriteMessageFormatter + 'static,
        W: std::fmt::Write + Send + Sync + 'static,
    {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(
            self.build_service_with_formatter(writer, formatter),
            max_retries,
            worker_count,
        )
    }

    /// Internal helper to construct the [`service::FmtWrite`] service with a custom formatter.
    pub fn build_service_with_formatter<W, MF>(
        self,
        writer: W,
        formatter: MF,
    ) -> Box<service::FmtWrite<W, MF>>
    where
        MF: WriteMessageFormatter + 'static,
        W: std::fmt::Write + Send + Sync + 'static,
    {
        service::FmtWrite::with_formatter(writer, formatter)
    }
}

impl Default for FmtWrite {
    /// Provides sensible defaults for byte-oriented logging.
    ///
    /// - **max_retries**: `3` (Standard resilience against transient I/O issues).
    /// - **worker_count**: `1` (Ensures sequential log ordering in asynchronous mode).
    fn default() -> Self {
        Self {
            max_retries: 3,
            worker_count: 1,
        }
    }
}

impl<W> TypedFmtWrite<W>
where
    W: std::fmt::Write + Send + Sync + 'static,
{
    /// Creates a new [`TypedFmtWrite`] with a specific writer and default policies.
    ///
    /// Defaults to 3 retries and 1 worker thread.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            max_retries: 3,
            worker_count: 1,
        }
    }

    /// Returns a reference to the underlying writer.
    pub fn get_writer(&self) -> &W {
        &self.writer
    }

    /// Returns the currently configured maximum retry attempts.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Returns the currently configured background worker count.
    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    /// Replaces the current writer while keeping existing retry and worker configurations.
    pub fn writer(self, writer: W) -> Self {
        Self { writer, ..self }
    }

    /// Updates the maximum number of retry attempts for this specific writer.
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Updates the background worker count for this specific writer.
    pub fn worker_count(self, worker_count: usize) -> Self {
        Self {
            worker_count,
            ..self
        }
    }

    /// Finalizes the builder and returns a high-level [`Logger`].
    ///
    /// This uses the default [`StandardWriteMessageFormatter`].
    pub fn build(self, concurrency: Concurrency) -> Logger {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct()),
            Concurrency::Async => Logger::new(self.build_impl_queued()),
        }
    }

    /// Builds the underlying [`DirectLogger`] implementation for this writer.
    ///
    /// Useful if you need to bypass the [`Logger`] wrapper and manage the
    /// synchronous driver manually.
    pub fn build_impl_direct(self) -> Box<DirectLogger> {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service(), max_retries)
    }

    /// Builds the underlying [`QueuedLogger`] implementation for this writer.
    ///
    /// Useful if you need to bypass the [`Logger`] wrapper and manage the
    /// asynchronous worker pool manually.
    pub fn build_impl_queued(self) -> Box<QueuedLogger> {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(self.build_service(), max_retries, worker_count)
    }

    /// Internal helper to construct the [`service::FmtWrite`] service for this specific writer
    /// using the standard formatter.
    pub fn build_service(self) -> Box<service::FmtWrite<W, StandardWriteMessageFormatter>> {
        service::FmtWrite::new(self.writer)
    }

    /// Finalizes the builder using a custom [`WriteMessageFormatter`].
    ///
    /// This allows you to define exactly how messages are serialized (e.g., JSON,
    /// custom text headers) before being sent to the writer.
    pub fn build_with_formatter<MF>(self, concurrency: Concurrency, formatter: MF) -> Logger
    where
        MF: WriteMessageFormatter + 'static,
    {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct_with_formatter(formatter)),
            Concurrency::Async => Logger::new(self.build_impl_queued_with_formatter(formatter)),
        }
    }

    /// Builds a [`DirectLogger`] with a specific formatter for this writer.
    pub fn build_impl_direct_with_formatter<MF>(self, formatter: MF) -> Box<DirectLogger>
    where
        MF: WriteMessageFormatter + 'static,
    {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service_with_formatter(formatter), max_retries)
    }

    /// Builds a [`QueuedLogger`] with a specific formatter for this writer.
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

    /// Internal helper to construct the [`service::FmtWrite`] service for this specific writer
    /// using a custom formatter.
    pub fn build_service_with_formatter<MF>(self, formatter: MF) -> Box<service::FmtWrite<W, MF>>
    where
        MF: WriteMessageFormatter + 'static,
    {
        service::FmtWrite::with_formatter(self.writer, formatter)
    }
}

impl BoxedFmtWrite {
    /// Creates a new [`TypedFmtWrite`] with a specific writer and default policies.
    ///
    /// Defaults to 3 retries and 1 worker thread.
    pub fn new(writer: Box<dyn std::fmt::Write + Send + Sync>) -> Self {
        Self {
            writer,
            max_retries: 3,
            worker_count: 1,
        }
    }

    /// Returns a reference to the underlying writer.
    pub fn get_writer(&self) -> &(dyn std::fmt::Write + Send + Sync) {
        self.writer.as_ref()
    }

    /// Returns the currently configured maximum retry attempts.
    pub fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    /// Returns the currently configured background worker count.
    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    /// Replaces the current writer while keeping existing retry and worker configurations.
    pub fn writer(self, writer: Box<dyn std::fmt::Write + Send + Sync>) -> Self {
        Self { writer, ..self }
    }

    /// Updates the maximum number of retry attempts for this specific writer.
    pub fn max_retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Updates the background worker count for this specific writer.
    pub fn worker_count(self, worker_count: usize) -> Self {
        Self {
            worker_count,
            ..self
        }
    }

    /// Finalizes the builder and returns a high-level [`Logger`].
    ///
    /// This uses the default [`StandardWriteMessageFormatter`].
    pub fn build(self, concurrency: Concurrency) -> Logger {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct()),
            Concurrency::Async => Logger::new(self.build_impl_queued()),
        }
    }

    /// Builds the underlying [`DirectLogger`] implementation for this writer.
    ///
    /// Useful if you need to bypass the [`Logger`] wrapper and manage the
    /// synchronous driver manually.
    pub fn build_impl_direct(self) -> Box<DirectLogger> {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service(), max_retries)
    }

    /// Builds the underlying [`QueuedLogger`] implementation for this writer.
    ///
    /// Useful if you need to bypass the [`Logger`] wrapper and manage the
    /// asynchronous worker pool manually.
    pub fn build_impl_queued(self) -> Box<QueuedLogger> {
        let max_retries = self.max_retries;
        let worker_count = self.worker_count;
        QueuedLogger::new(self.build_service(), max_retries, worker_count)
    }

    /// Internal helper to construct the [`service::BoxedFmtWrite`] service for this specific writer
    /// using the standard formatter.
    pub fn build_service(self) -> Box<service::BoxedFmtWrite<StandardWriteMessageFormatter>> {
        service::BoxedFmtWrite::new(self.writer)
    }

    /// Finalizes the builder using a custom [`WriteMessageFormatter`].
    ///
    /// This allows you to define exactly how messages are serialized (e.g., JSON,
    /// custom text headers) before being sent to the writer.
    pub fn build_with_formatter<MF>(self, concurrency: Concurrency, formatter: MF) -> Logger
    where
        MF: WriteMessageFormatter + 'static,
    {
        match concurrency {
            Concurrency::Sync => Logger::new(self.build_impl_direct_with_formatter(formatter)),
            Concurrency::Async => Logger::new(self.build_impl_queued_with_formatter(formatter)),
        }
    }

    /// Builds a [`DirectLogger`] with a specific formatter for this writer.
    pub fn build_impl_direct_with_formatter<MF>(self, formatter: MF) -> Box<DirectLogger>
    where
        MF: WriteMessageFormatter + 'static,
    {
        let max_retries = self.max_retries;
        DirectLogger::new(self.build_service_with_formatter(formatter), max_retries)
    }

    /// Builds a [`QueuedLogger`] with a specific formatter for this writer.
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

    /// Internal helper to construct the [`service::BoxedFmtWrite`] service for this specific writer
    /// using a custom formatter.
    pub fn build_service_with_formatter<MF>(self, formatter: MF) -> Box<service::BoxedFmtWrite<MF>>
    where
        MF: WriteMessageFormatter + 'static,
    {
        service::BoxedFmtWrite::with_formatter(self.writer, formatter)
    }
}
