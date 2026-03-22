// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::*;
use crate::{DirectLogger, Logger, QueuedLogger, Service, SilentLogger};
use std::fs::File;

#[cfg(feature = "aws")]
use crate::CloudWatchLogger;
#[cfg(feature = "loki")]
use crate::LokiLogger;
use crate::service::aws::CloudWatchCout;
use crate::service::write::MessageFormatter;

/// A centralized factory for constructing various [`Logger`] implementations.
///
/// [`LoggerFactory`] provides a clean, high-level API for instantiating both synchronous
/// and asynchronous loggers. It abstracts the complexity of manual service and
/// worker pool configuration by providing pre-configured sensible defaults for
/// common outputs like `stdout`, `stderr`, files, and remote service like Loki.
///
/// # Architecture Overview
///
/// The factory produces loggers based on two primary execution models:
///
/// 1. **Directhronous ([`DirectLogger`])**: Log calls block the current thread until processing
///    is complete. Best for CLI tools, debugging, or critical failure paths.
/// 2. **Queuedhronous ([`QueuedLogger`])**: Log calls offload messages to a background
///    worker pool. Best for high-performance service where logging latency must be minimized.
///
///
///
/// # Categories of Loggers
///
/// - **Standard Streams**: Quick access to `stdout` ([`direct_cout`](Self::direct_cout)) and `stderr`.
/// - **File I/O**: Direct file logging with sequential write guarantees ([`queued_file`](Self::queued_file)).
/// - **Trait Objects**: Wrap existing [[`std::io::Write`]] or [[`std::fmt::Write`]] implementations.
/// - **Remote Services**: Integration with external platforms like Grafana Loki (requires feature flags).
/// - **Sentinel**: A "No-Op" logger ([`SilentLogger`]) for testing or optional logging scenarios.
///
/// # Performance Considerations
///
/// When using `queued_` methods, the factory configures a background worker pool. For
/// local I/O (files, strings), a single worker is typically used to ensure message
/// ordering. For network-heavy service like Loki, multiple workers can be used to
/// increase throughput.
pub struct LoggerFactory {}

impl LoggerFactory {
    /// Returns a sentinel ["Silent" logger][`SilentLogger`].
    ///
    /// All log messages sent to this instance are silently dropped. This is
    /// ideal for unit testing or as a default placeholder to avoid [`Option<Logger>`][`Logger`] overhead.
    pub fn silent() -> Logger {
        Logger::new(SilentLogger::new())
    }

    /***********************************************************************
     * Pure service
     ***********************************************************************/

    /// Promotes a pre-configured [`DirectLogger`] into the standard [`Logger`] interface.
    ///
    /// Use this when you have already manually initialized a synchronous logger
    /// and need to wrap it for use in the broader application.
    pub fn direct_impl(logger: Box<DirectLogger>) -> Logger {
        Logger::new(logger)
    }

    /// Promotes a pre-configured [`QueuedLogger`] into the standard [`Logger`] interface.
    ///
    /// Use this when you have a custom-configured [asynchronous logger][`QueuedLogger`](e.g., specific
    /// channel sizes or thread settings) and need to wrap it for the application.
    pub fn queued_impl(logger: Box<QueuedLogger>) -> Logger {
        Logger::new(logger)
    }

    /// Constructs a [synchronous logger][`DirectLogger`] backed by a concrete [`Service`].
    ///
    /// **Behavior:**
    /// - **Blocking:** Log calls will block the current thread until the [`Service`]
    ///   successfully processes the message or exhausts all retries.
    /// - **Reliability:** Guarantees order and completion before moving to the next line of code.
    /// - `max_retries`: The number of times the service will attempt to re-send a failed log entry.
    pub fn direct_service(service: Box<dyn Service + Send + Sync>, max_retries: usize) -> Logger {
        Logger::new(DirectLogger::new(service, max_retries))
    }

    /// Constructs an [asynchronous, multi-threaded logger][`QueuedLogger`] backed by a concrete [`Service`].
    ///
    /// **Behavior:**
    /// - **Non-blocking:** Log calls return immediately after handing the message to an
    ///   internal buffer, minimizing latency in the hot path.
    /// - **Parallelism:** Spawns a pool of `worker_count` threads to process logs in the background.
    /// - `max_retries`: Retries are handled per-worker without stalling the main application thread.
    /// - `worker_count`: Controls the number of concurrent background threads processing the log queue.
    pub fn queued_service(
        service: Box<dyn Service + Send + Sync>,
        max_retries: usize,
        worker_count: usize,
    ) -> Logger {
        Logger::new(QueuedLogger::new(service, max_retries, worker_count))
    }

    /***********************************************************************
     * StdWriteService
     ***********************************************************************/

    /// Creates a [synchronous logger][`DirectLogger`] for `stdout`.
    ///
    /// **Hardcoded Config:**
    /// - **Retries:** 0 (Fails immediately if the stream is broken).
    /// - **Execution:** Blocking; the caller waits for the write to finish.
    pub fn direct_cout() -> Logger {
        Logger::new(DirectLogger::new(
            StandardCoutWrite::new(),
            0, // <--- Retry count set to 0
        ))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for `stdout`.
    ///
    /// **Hardcoded Config:**
    /// - **Retries:** 0.
    /// - **Workers:** 1 (Single background thread to maintain message order).
    pub fn queued_cout() -> Logger {
        Logger::new(QueuedLogger::new(
            StandardCoutWrite::new(),
            0, // <--- Retry count
            1, // <--- Worker count
        ))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for `stdout` with a [custom formatter][`MessageFormatter`].
    ///
    /// **Hardcoded Config:** 0 retries.
    pub fn direct_cout_formatted<F>(formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(DirectLogger::new(CoutWrite::with_formatter(formatter), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for `stdout` with a [custom formatter][`MessageFormatter`].
    ///
    /// **Hardcoded Config:** 0 retries, 1 background worker.
    pub fn queued_cout_formatted<F>(formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(QueuedLogger::new(
            CoutWrite::with_formatter(formatter),
            0,
            1,
        ))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for `stderr`.
    ///
    /// **Hardcoded Config:** 0 retries.
    pub fn direct_cerr() -> Logger {
        Logger::new(DirectLogger::new(StandardCerrWrite::new(), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for `stderr`.
    ///
    /// **Hardcoded Config:** 0 retries, 1 background worker.
    pub fn queued_cerr() -> Logger {
        Logger::new(QueuedLogger::new(StandardCerrWrite::new(), 0, 1))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for `stderr` with a [custom formatter][`MessageFormatter`].
    ///
    /// **Hardcoded Config:** 0 retries.
    pub fn direct_cerr_formatted<F>(formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(DirectLogger::new(CerrWrite::with_formatter(formatter), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for `stderr` with a [custom formatter][`MessageFormatter`].
    ///
    /// **Hardcoded Config:** 0 retries, 1 background worker.
    pub fn queued_cerr_formatted<F>(formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(QueuedLogger::new(
            CerrWrite::with_formatter(formatter),
            0,
            1,
        ))
    }

    /***********************************************************************
     * IoWriteService
     ***********************************************************************/

    /***********************************************************************
     * FileWriteService
     ***********************************************************************/

    /// Creates a [synchronous logger][`DirectLogger`] that writes to a filesystem [`File`].
    ///
    /// **Configuration:**
    /// - **Formatter:** Uses the [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0 (Fails immediately if a disk error occurs).
    /// - **Execution:** Blocking; the calling thread waits for the file I/O to complete.
    pub fn direct_file(file: File) -> Logger {
        Logger::new(DirectLogger::new(StandardFileWrite::new(file), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] that writes to a filesystem [`File`].
    ///
    /// **Configuration:**
    /// - **Formatter:** Uses the [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0.
    /// - **Workers:** 1 (A single background thread ensures log lines are written in the order they were called).
    /// - **Execution:** Non-blocking; logs are queued for the background worker.
    pub fn queued_file(file: File) -> Logger {
        Logger::new(QueuedLogger::new(StandardFileWrite::new(file), 0, 1))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for a [`File`] using a [custom message formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Execution:** Blocking.
    pub fn direct_file_formatted<F>(file: File, formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(DirectLogger::new(
            FileWrite::<F>::with_formatter(file, formatter),
            0,
        ))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for a [`File`] using a [custom message formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Workers:** 1.
    /// - **Execution:** Non-blocking.
    pub fn queued_file_formatted<F>(file: File, formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(QueuedLogger::new(
            FileWrite::<F>::with_formatter(file, formatter),
            0,
            1,
        ))
    }

    /***********************************************************************
     * BoxedIoWriteService
     ***********************************************************************/

    /// Creates a [synchronous logger][`DirectLogger`] that writes to a boxed [`std::io::Write`] trait object.
    ///
    /// **Configuration:**
    /// - **Formatter:** [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0.
    /// - **Execution:** Blocking; calls to the logger wait for the underlying [`Write`][`std::io::Write`]
    ///   implementation to return.
    pub fn direct_boxed_io(boxed_io: Box<dyn std::io::Write + Send + Sync>) -> Logger {
        Logger::new(DirectLogger::new(StandardBoxedIoWrite::new(boxed_io), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] that writes to a boxed [`std::io::Write`] trait object.
    ///
    /// **Configuration:**
    /// - **Formatter:** [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0.
    /// - **Workers:** 1 (Ensures logs are written sequentially even if dispatched from multiple threads).
    /// - **Execution:** Non-blocking.
    pub fn queued_boxed_io(boxed_io: Box<dyn std::io::Write + Send + Sync>) -> Logger {
        Logger::new(QueuedLogger::new(StandardBoxedIoWrite::new(boxed_io), 0, 1))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for a boxed [`Write`][`std::io::Write`] object using a [custom formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Execution:** Blocking.
    pub fn direct_boxed_io_formatted<F>(
        boxed_io: Box<dyn std::io::Write + Send + Sync>,
        formatter: F,
    ) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(DirectLogger::new(
            BoxedIoWrite::<F>::with_formatter(boxed_io, formatter),
            0,
        ))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for a boxed [`Write`][`std::io::Write`] object using a [custom formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Workers:** 1.
    /// - **Execution:** Non-blocking.
    pub fn queued_boxed_io_formatted<F>(
        boxed_io: Box<dyn std::io::Write + Send + Sync>,
        formatter: F,
    ) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(QueuedLogger::new(
            BoxedIoWrite::<F>::with_formatter(boxed_io, formatter),
            0,
            1,
        ))
    }

    /***********************************************************************
     * FmtWriteService
     ***********************************************************************/

    /***********************************************************************
     * StringWriteService
     ***********************************************************************/

    /// Creates a [synchronous logger][`DirectLogger`] that appends log entries to an in-memory [`String`].
    ///
    /// **Configuration:**
    /// - **Formatter:** [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0.
    /// - **Execution:** Blocking; the [`String`] is modified immediately on the current thread.
    pub fn direct_string(string: String) -> Logger {
        Logger::new(DirectLogger::new(StandardStringFmtWrite::new(string), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] that appends log entries to an in-memory [`String`].
    ///
    /// **Configuration:**
    /// - **Formatter:** [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0.
    /// - **Workers:** 1 (Single background thread to ensure sequential string growth).
    /// - **Execution:** Non-blocking; the application continues while the worker appends to the buffer.
    pub fn queued_string(string: String) -> Logger {
        Logger::new(QueuedLogger::new(StandardStringFmtWrite::new(string), 0, 1))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for a [`String`] buffer using a [custom formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Execution:** Blocking.
    pub fn direct_string_formatted<F>(string: String, formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(DirectLogger::new(
            StringFmtWrite::<F>::with_formatter(string, formatter),
            0,
        ))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for a [`String`] buffer using a [custom formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Workers:** 1.
    /// - **Execution:** Non-blocking.
    pub fn queued_string_formatted<F>(string: String, formatter: F) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(QueuedLogger::new(
            StringFmtWrite::<F>::with_formatter(string, formatter),
            0,
            1,
        ))
    }

    /***********************************************************************
     * BoxedFmtWriteService
     ***********************************************************************/

    /// Creates a [synchronous logger][`DirectLogger`] that writes to a boxed [`std::fmt::Write`] trait object.
    ///
    /// **Configuration:**
    /// - **Formatter:** [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0 (Immediate failure if formatting fails).
    /// - **Execution:** Blocking; the caller waits for the string formatting to complete.
    pub fn direct_boxed_fmt(boxed_fmt: Box<dyn std::fmt::Write + Send + Sync>) -> Logger {
        Logger::new(DirectLogger::new(StandardBoxedFmtWrite::new(boxed_fmt), 0))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] that writes to a boxed [`std::fmt::Write`] trait object.
    ///
    /// **Configuration:**
    /// - **Formatter:** [`StandardMessageFormatter`][`crate::service::write::StandardMessageFormatter`].
    /// - **Retries:** 0.
    /// - **Workers:** 1 (Ensures logs are formatted and written in call order).
    /// - **Execution:** Non-blocking; offloads string formatting to a background thread.
    pub fn queued_boxed_fmt(boxed_fmt: Box<dyn std::fmt::Write + Send + Sync>) -> Logger {
        Logger::new(QueuedLogger::new(
            StandardBoxedFmtWrite::new(boxed_fmt),
            0,
            1,
        ))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for a boxed [`std::fmt::Write`] object using a [custom formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Execution:** Blocking.
    pub fn direct_boxed_fmt_formatted<F>(
        boxed_fmt: Box<dyn std::fmt::Write + Send + Sync>,
        formatter: F,
    ) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(DirectLogger::new(
            BoxedFmtWrite::<F>::with_formatter(boxed_fmt, formatter),
            0,
        ))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for a boxed [`std::fmt::Write`] object using a [custom formatter][`MessageFormatter`].
    ///
    /// **Configuration:**
    /// - **Retries:** 0.
    /// - **Workers:** 1.
    /// - **Execution:** Non-blocking.
    pub fn queued_boxed_fmt_formatted<F>(
        boxed_fmt: Box<dyn std::fmt::Write + Send + Sync>,
        formatter: F,
    ) -> Logger
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Logger::new(QueuedLogger::new(
            BoxedFmtWrite::<F>::with_formatter(boxed_fmt, formatter),
            0,
            1,
        ))
    }

    /***********************************************************************
     * LokiService
     ***********************************************************************/

    /// Creates a new [`LokiLogger`] wrapped in a [`Logger`] container.
    ///
    /// This function initializes a background worker thread that buffers and batches logs
    /// before pushing them to a Grafana Loki instance.
    ///
    /// ### Behavior
    /// - **Queuedhronous**: Log calls are non-blocking; they are sent to a channel and
    ///   processed by a dedicated worker.
    /// - **Batching**: The worker automatically groups messages by level and time
    ///   to optimize HTTP pressure.
    /// - **Resilience**: If the Loki server is down, the logger will attempt to
    ///   retry based on the `max_retries` defined in [`LokiConfig`].
    ///
    /// ### Fallback
    /// If all retry attempts fail (or the network is unreachable), the logger will
    /// trigger a fallback mechanism (printing to `stderr/stdout`) to ensure no data loss.
    ///
    /// ### Feature Gate
    /// Requires the **`loki`** feature to be enabled.
    ///
    /// # Panics
    /// - if the underlying HTTP client (reqwest) cannot be initialized due to
    /// invalid system configuration or TLS issues.
    #[cfg(feature = "loki")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
    pub fn loki(config: LokiConfig) -> Logger {
        Logger::new(LokiLogger::new(config))
    }

    /// Creates a new [`CloudWatchLogger`] wrapped in a [`Logger`] container.
    ///
    /// This function initializes a background worker thread that buffers and batches logs
    /// before pushing them to a AWS Cloudwatch instance.
    ///
    /// ### Behavior
    /// - **Queuedhronous**: Log calls are non-blocking; they are sent to a channel and
    ///   processed by a dedicated worker.
    /// - **Batching**: The worker automatically groups messages by level and time
    ///   to optimize HTTP pressure.
    ///
    /// ### Fallback
    /// If all retry attempts fail (or the network is unreachable), the logger will
    /// trigger a fallback mechanism (printing to `stderr/stdout`) to ensure no data loss.
    ///
    /// ### Feature Gate
    /// Requires the **`aws`** feature to be enabled.
    ///
    /// # Panics
    /// - if the AWS client cannot be initialized due to
    /// invalid system configuration or TLS issues.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn cloudwatch_cfg(config: CloudWatchConfig) -> Logger {
        Logger::new(CloudWatchLogger::new(config))
    }

    /// Creates a new [`CloudWatchLogger`] wrapped in a [`Logger`] container.
    ///
    /// This function initializes a background worker thread that buffers and batches logs
    /// before pushing them to a AWS Cloudwatch instance.
    ///
    /// Config is loaded from ENV.
    ///
    /// ### Behavior
    /// - **Queuedhronous**: Log calls are non-blocking; they are sent to a channel and
    ///   processed by a dedicated worker.
    /// - **Batching**: The worker automatically groups messages by level and time
    ///   to optimize HTTP pressure.
    ///
    /// ### Fallback
    /// If all retry attempts fail (or the network is unreachable), the logger will
    /// trigger a fallback mechanism (printing to `stderr/stdout`) to ensure no data loss.
    ///
    /// ### Feature Gate
    /// Requires the **`aws`** feature to be enabled.
    ///
    /// # Panics
    /// - if the AWS client cannot be initialized due to
    /// invalid system configuration or TLS issues.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn cloudwatch_env(log_group: String) -> Logger {
        Logger::new(CloudWatchLogger::from_env(log_group))
    }

    /// Creates a [synchronous logger][`DirectLogger`] for AWS CloudWatch via `stdout`.
    ///
    /// This logger uses the [`CloudWatchCoutMessageFormatter`] to transform log messages
    /// into single-line JSON objects, making them compatible with CloudWatch Logs
    /// subscription filters and insights.
    ///
    /// **Configuration:**
    /// - **Format:** `{"level":"...","msg":"..."}`
    /// - **Retries:** 3 (Attempts to re-write if the standard output stream is temporarily busy or interrupted).
    /// - **Execution:** Blocking; ideal for Lambda functions where the runtime environment
    ///   freezes the CPU after the main handler returns.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn direct_cloudwatch_cout() -> Logger {
        Logger::new(DirectLogger::new(CloudWatchCout::new(), 3))
    }

    /// Creates an [asynchronous logger][`QueuedLogger`] for AWS CloudWatch via `stdout`.
    ///
    /// This logger is optimized for long-running containerized applications (ECS/Fargate)
    /// where logging throughput is high and you want to minimize the impact on the
    /// application's primary execution path.
    ///
    /// **Configuration:**
    /// - **Format:** Single-line JSON.
    /// - **Retries:** 3.
    /// - **Workers:** 1 (Ensures logs remain in chronological order within the CloudWatch stream).
    /// - **Execution:** Non-blocking; logs are offloaded to a background worker thread.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn queued_cloudwatch_cout() -> Logger {
        Logger::new(QueuedLogger::new(CloudWatchCout::new(), 3, 1))
    }}
