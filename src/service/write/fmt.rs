// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::ServiceError;
use crate::service::fallback::Fallback;
use crate::service::write::{StandardMessageFormatter, MessageFormatter};
use crate::{LoggerStatus, Message, Service};
use std::any::Any;
use std::sync::Mutex;

/// An internal wrapper for data protected by the [`FmtWrite`] service mutex.
///
/// This structure ensures that both the [`MessageFormatter`] and the [`std::fmt::Write`]
/// destination are kept together, allowing them to be borrowed mutably as a single unit
/// once the lock is acquired.
struct FmtData<W, F>
where
    W: std::fmt::Write + Send + Sync,
    F: MessageFormatter,
{
    /// The destination for formatted string data.
    /// Common types include [`String`] or a custom buffer.
    writer: W,
    /// The formatting strategy used to transform a [`Message`] into a string.
    formatter: F,
}

/// A thread-safe logging [`Service`] for string-based output destinations.
///
/// [`Fmt`] service implements the [`Service`] trait for types that satisfy [`std::fmt::Write`].
/// It is ideal for in-memory logging, testing, or targets that do not use byte-oriented I/O.
///
/// ### Thread Safety
/// The internal `FmtWriteData` is wrapped in a [`Mutex`]. This allows the [`Service`]
/// to be shared across threads ([`Send`] + [`Sync`]), while ensuring that only one thread
/// can perform a [`work`](Self::work) operation at a time.
pub struct Fmt<W, F>
where
    W: std::fmt::Write + Send + Sync,
    F: MessageFormatter,
{
    /// A mutex-protected container holding the writer and formatter.
    /// Access is managed via [`unlock_guard`].
    writer: Mutex<FmtData<W, F>>,
}

impl<W, F> Fmt<W, F>
where
    W: std::fmt::Write + Send + Sync,
    F: MessageFormatter + Default,
{
    /// Creates a new, heap-allocated [`Fmt`] service.
    ///
    /// # Parameters
    /// - `writer`: An object implementing [`std::fmt::Write`].
    /// - `formatter`: An object implementing [`MessageFormatter`].
    ///
    /// # Example
    /// ```
    /// # use timber_rust::{QueuedLogger, Logger};
    /// # use timber_rust::service::FmtWrite;
    /// # use timber_rust::service::write::StandardMessageFormatter;
    /// let service = FmtWrite::<String, StandardMessageFormatter>::new(String::new());
    /// let logger = QueuedLogger::new(service, 3, 4); // 3 retries, 4 worker threads
    /// let logger = Logger::new(logger);
    /// ```
    pub fn new(writer: W) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(FmtData {
                writer,
                formatter: F::default(),
            }),
        })
    }

    /// Creates a new, heap-allocated [`Fmt`] service with a custom [formatter][`MessageFormatter`].
    ///
    /// # Parameters
    /// - `writer`: An object implementing [`std::fmt::Write`].
    /// - `formatter`: An object implementing [`MessageFormatter`].
    ///
    /// # Example
    /// ```
    /// # use timber_rust::{QueuedLogger, Logger};
    /// # use timber_rust::service::write::StandardMessageFormatter;
    /// # use timber_rust::service::FmtWrite;
    /// let service = FmtWrite::<String, StandardMessageFormatter>::new(String::new());
    /// let logger = QueuedLogger::new(service, 3, 4); // 3 retries, 4 worker threads
    /// let logger = Logger::new(logger);
    /// ```
    pub fn with_formatter(writer: W, formatter: F) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(FmtData { writer, formatter }),
        })
    }

    /// Allows safe, read-only access to the internal buffer without stopping the logger.
    ///
    /// Use this to "peek" at logs while the application is still running—perfect for
    /// health-check endpoints that expose recent logs or for verifying state in tests.
    ///
    /// ### Thread Safety
    /// This method acquires a mutex lock. While the closure `f` is executing, any
    /// incoming logs from other threads will **block** until the closure returns.
    /// Keep the logic inside the closure as fast as possible.
    ///
    /// ### Returns
    /// - [`Some(R)`][`Some`]: The result of your closure if the lock was acquired.
    /// - [`None`]: If the internal lock was poisoned by a previous panic.
    pub fn inspect_writer<R>(&self, f: impl FnOnce(&W) -> R) -> Option<R> {
        self.writer.lock().ok().map(|data| f(&data.writer))
    }

    /// Destroys the [`Service`] and reclaims ownership of the underlying buffer or writer.
    ///
    /// Use this at the end of a program, a test case, or a lifecycle stage to extract
    /// all recorded logs and free up the resources used by the [`Service`].
    ///
    /// ### Ownership & Lifecycle
    /// This method consumes `self`, meaning the [`Fmt`] service can no longer be
    /// used after this call. This is the only way to get full, non-cloned ownership
    /// of the internal writer (e.g., a [`String`] or [`Vec<u8>`]).
    ///
    /// ### Guarantees
    /// Because this takes ownership of the [`Service`], it is compile-time guaranteed
    /// that no other threads can be writing to the buffer when this is called.
    pub fn recover_writer(self) -> Result<W, ServiceError> {
        let data = self.writer.into_inner();
        match data {
            Ok(data) => Ok(data.writer),
            Err(_) => Err(ServiceError::LockPoisoned),
        }
    }

    /// Clears the underlying writer if the type supports it (e.g., String).
    /// Useful for reusing the [`Service`] in benchmarks or test suites.
    pub fn clear_writer(&self)
    where
        W: Default,
    {
        if let Ok(mut data) = self.writer.lock() {
            data.writer = W::default();
        }
    }
}

impl<W, F> Service for Fmt<W, F>
where
    W: std::fmt::Write + Send + Sync + 'static,
    F: MessageFormatter + 'static,
{
    /// Returns the current operational status.
    /// Currently always returns [`LoggerStatus::Running`].
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Acquires a lock and writes a formatted [`Message`] to the internal writer.
    ///
    /// This method uses [`MessageFormatter::format_fmt`] to perform the write.
    ///
    /// # Errors
    /// - Returns [`ServiceError::LockPoisoned`] if the internal mutex is poisoned.
    /// - Forwards any [`ServiceError`] returned by the formatter.
    fn work(&self, msg: &Message) -> Result<(), ServiceError> {
        let mut guard = self.writer.lock()?;
        // Destructure the guard to get mutable access to fields
        let FmtData {
            formatter, writer, ..
        } = &mut *guard;
        formatter.format_fmt(msg, writer)?;
        Ok(())
    }

    /// Returns a reference to the underlying type as [Any] for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl<W, F> Fallback for Fmt<W, F>
where
    W: std::fmt::Write + Send + Sync + 'static,
    F: MessageFormatter + 'static,
{
    /// Attempts to log an error to `stdout` if the primary [`work`](Self::work) call fails.
    ///
    /// This method performs a best-effort write. If the mutex is locked by a hanging
    /// thread, the fallback will be skipped to avoid a deadlock.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        if let Ok(mut guard) = self.writer.lock() {
            let mut out = std::io::stdout();
            let _ = guard.formatter.format_io(msg, &mut out);
            let _ = eprintln!("FmtWriteService Error: {}", error);
        }
    }
}

/// A specialized alias for logging directly into a [`String`].
///
/// This is commonly used for unit testing or collecting logs for display in a UI.
///
/// **Note:** Trait bounds on `F` are not enforced at definition time but are
/// checked during instantiation.
#[allow(type_alias_bounds)]
pub type StringFmt<F: MessageFormatter> = Fmt<String, F>;

/// A [`StringFmt`] service pre-configured with the [`StandardMessageFormatter`].
///
/// This provides a zero-configuration path for in-memory string logging.
pub type StandardStringFmt = Fmt<String, StandardMessageFormatter>;
