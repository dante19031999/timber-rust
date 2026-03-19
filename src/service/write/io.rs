// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::ServiceError;
use crate::service::fallback::Fallback;
use crate::service::write::{StandardMessageFormatter, MessageFormatter};
use crate::{LoggerStatus, Message, Service};
use std::any::Any;
use std::sync::Mutex;

/// A private synchronization container for [`Io`].
///
/// This struct groups the writer and formatter into a single unit. This ensures
/// **atomicity**: the formatter state and writer output are synchronized.
/// By placing both in a single [`Mutex`], we guarantee that log interleaving
/// is impossible even if the formatter holds internal state.
struct IoData<W, F>
where
    W: std::io::Write + Send + Sync,
    F: MessageFormatter,
{
    /// The byte-oriented output destination.
    writer: W,
    /// The logic used to transform a [`Message`] into bytes.
    formatter: F,
}

/// A thread-safe [`Service`] for byte-stream logging destinations.
///
/// [`Io`] is the primary workhorse for file-based, socket-based, or
/// console-based logging. It implements the [`Service`] trait by wrapping its
/// internal data in a [`Mutex`].
///
/// ### Performance Note
/// This service does not explicitly call `flush()` after every write. If low-latency
/// is required with guaranteed persistence, wrap your writer in [`std::io::BufWriter`].
pub struct Io<W, F>
where
    W: std::io::Write + Send + Sync,
    F: MessageFormatter,
{
    /// The mutex-protected destination and formatting logic.
    writer: Mutex<IoData<W, F>>,
}

impl<W, F> Io<W, F>
where
    W: std::io::Write + Send + Sync,
    F: MessageFormatter,
{
    /// Creates a new [`Io`] on the heap.
    ///
    /// # Parameters
    /// - `writer`: A type implementing [`std::io::Write`].
    /// - `formatter`: The [`MessageFormatter`] implementation.
    pub fn new(writer: W) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(IoData {
                writer,
                formatter: Default::default(),
            }),
        })
    }

    /// Creates a new [`Io`] on the heap with a custom [formatter][`MessageFormatter`].
    ///
    /// # Parameters
    /// - `writer`: A type implementing [`std::io::Write`].
    /// - `formatter`: The [`MessageFormatter`] implementation.
    pub fn with_formatter(writer: W, formatter: F) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(IoData { writer, formatter }),
        })
    }
}

impl<W, F> Service for Io<W, F>
where
    W: std::io::Write + Send + Sync + 'static,
    F: MessageFormatter + 'static,
{
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Acquires the lock and streams the formatted message to the writer.
    ///
    /// # Errors
    /// - [`ServiceError::LockPoisoned`]: If the internal [`Mutex`] is poisoned.
    /// - [`ServiceError`]: If the formatter fails or the writer encounters an I/O error.
    fn work(&self, msg: &Message) -> Result<(), ServiceError> {
        let mut guard = self.writer.lock()?;

        // Destructuring allows simultaneous mutable access to both fields.
        let IoData {
            formatter, writer, ..
        } = &mut *guard;

        formatter.format_io(msg, writer)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<W, F> Fallback for Io<W, F>
where
    W: std::io::Write + Send + Sync + 'static,
    F: MessageFormatter + 'static,
{
    /// Best-effort fallback. Skips writing if the mutex is locked or poisoned
    /// to prevent cascading failures in the logging pipeline.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        if let Ok(mut guard) = self.writer.lock() {
            let mut out = std::io::stdout();
            let _ = guard.formatter.format_io(msg, &mut out);
            let _ = eprintln!("IoWriteService Fallback [Error: {}]", error);
        }
    }
}

/// A type alias for an [`Io`] service using a dynamic trait object.
///
/// This is particularly useful when you need to change the logging destination
/// at runtime (e.g., switching from a File to a Network stream).
///
/// **Bound Requirements:** The inner writer must be [`Send`] + [`Sync`] + `'static`.
#[allow(type_alias_bounds)]
pub type BoxedIo<F: MessageFormatter> = Io<Box<dyn std::io::Write + Send + Sync>, F>;

/// A type alias for an [`Io`] service writing specifically to a [`std::fs::File`].
#[allow(type_alias_bounds)]
pub type FileWrite<F: MessageFormatter> = Io<std::fs::File, F>;

/// A pre-configured [`BoxedIo`] service using the crate's [`StandardMessageFormatter`].
pub type StandardBoxedIo =
    Io<Box<dyn std::io::Write + Send + Sync>, StandardMessageFormatter>;

/// A pre-configured [`FileWrite`] using the crate's [`StandardMessageFormatter`].
pub type StandardFileWrite = Io<std::fs::File, StandardMessageFormatter>;
