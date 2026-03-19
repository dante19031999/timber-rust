use crate::service::ServiceError;
use crate::service::write::{StandardMessageFormatter, MessageFormatter};
use crate::{Fallback, LoggerStatus, Message, Service};
use std::any::Any;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

/// A private synchronization container for heap-allocated string writers.
///
/// Unlike [`crate::service::write::fmt::FmtWriteServiceData`], this struct explicitly holds a trait object
/// ([`Arc<dyn std::fmt::Write>`]). This separation allows us to handle the unique
/// borrowing requirements of shared trait objects within the [`Service`] implementation.
///
/// Not that this structure causes double lock. For the service and for the Arc.
/// This was built for testing purposes, not for high performance.
struct ArcedFmtServiceData<F>
where
    F: MessageFormatter,
{
    /// A heap-allocated, dynamically dispatched writer implementing [`std::fmt::Write`].
    /// Must be [`Send`] + [`Sync`] to allow the [`Service`] to move between threads.
    writer: Arc<Mutex<dyn std::fmt::Write + Send + Sync>>,
    /// The strategy used to format the [`Message`].
    formatter: F,
}

/// A specialized [`Service`] for dynamically dispatched string-based logging.
///
/// ### Why this exists (The "Orphan Rule" Workaround)
/// In Rust, [`std::fmt::Write`] is not implemented for `Arc<dyn std::fmt::Write>`.
/// Using a type alias would require implementing a foreign trait on
/// a foreign type, which is forbidden by Rust's "Orphan Rules."
///
/// [`ArcedFmt`] service solves this by providing a concrete struct that "wraps"
/// the shared trait object, allowing us to manually dispatch the write calls in the
/// [`work`](Self::work) method.
///
/// Not that this structure causes double lock. For the service and for the Arc.
/// This was built for testing purposes, not for high performance.
pub struct ArcedFmt<F>
where
    F: MessageFormatter,
{
    /// Mutex-protected storage for the shared writer and formatter.
    writer: Mutex<ArcedFmtServiceData<F>>,
}

impl<F> ArcedFmt<F>
where
    F: MessageFormatter,
{
    /// Creates a new [`ArcedFmt`] service on the heap.
    ///
    /// # Parameters
    /// - `writer`: A shared trait object. This is useful when the exact type of
    ///   the string writer (e.g., a custom UI buffer vs. a standard [`String`])
    ///   is not known at compile time.
    /// - `formatter`: The [`MessageFormatter`] implementation.
    pub fn new(writer: Arc<Mutex<dyn std::fmt::Write + Send + Sync>>) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(ArcedFmtServiceData {
                writer,
                formatter: Default::default(),
            }),
        })
    }

    /// Creates a new [`ArcedFmt`] service on the heap with a custom [formatter][`MessageFormatter`].
    ///
    /// # Parameters
    /// - `writer`: A shared trait object. This is useful when the exact type of
    ///   the string writer (e.g., a custom UI buffer vs. a standard [`String`])
    ///   is not known at compile time.
    /// - `formatter`: The [`MessageFormatter`] implementation.
    pub fn with_formatter(
        writer: Arc<Mutex<dyn std::fmt::Write + Send + Sync>>,
        formatter: F,
    ) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(ArcedFmtServiceData { writer, formatter }),
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
    pub fn inspect_writer<R>(
        &self,
        f: impl FnOnce(&Arc<Mutex<dyn std::fmt::Write + Send + Sync>>) -> R,
    ) -> Option<R> {
        self.writer.lock().ok().map(|data| f(&data.writer))
    }

    /// Destroys the [`Service`] and reclaims ownership of the underlying buffer or writer.
    ///
    /// Use this at the end of a program, a test case, or a lifecycle stage to extract
    /// all recorded logs and free up the resources used by the [`Service`].
    ///
    /// ### Ownership & Lifecycle
    /// This method consumes `self`, meaning the [`ArcedFmt`] service can no longer be
    /// used after this call.
    ///
    /// ### Guarantees
    /// Because this takes ownership of the [`Service`], it is compile-time guaranteed
    /// that no other threads can be writing to the buffer when this is called.
    /// Destroys the service and returns the shared writer.
    ///
    /// ### Warning
    /// Unlike other services, this does NOT return the underlying buffer,
    /// but the shared [`Arc`] handle. Other parts of the system may still
    /// hold references to this writer.
    pub fn recover_writer(
        self,
    ) -> Result<Arc<Mutex<dyn std::fmt::Write + Send + Sync>>, ServiceError> {
        match self.writer.into_inner() {
            Ok(data) => Ok(data.writer),
            Err(_) => Err(ServiceError::LockPoisoned),
        }
    }
}

impl<F> Service for ArcedFmt<F>
where
    F: MessageFormatter + 'static,
{
    /// Returns the current operational status.
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Acquires the lock and dispatches the write to the shared trait object.
    ///
    /// # Internal Mechanics
    /// Since `Arc<dyn std::fmt::Write>` doesn't implement [`std::fmt::Write`], this method uses
    /// `Arc<Mutex>` to obtain a mutable reference to the underlying
    /// trait object before passing it to the formatter.
    ///
    /// # Errors
    /// [`ServiceError::LockPoisoned`] if the mutex is poisoned
    /// - Forwards any [`ServiceError`] returned by the formatter.
    fn work(&self, msg: &Message) -> Result<(), ServiceError> {
        let mut guard = self.writer.lock()?;

        // Destructure the internal data
        let ArcedFmtServiceData {
            formatter, writer, ..
        } = &mut *guard;

        // DOBLE LOCK
        // Manual dispatch: conversion from Arc<dyn Write> to &mut dyn Write
        let mut writer_guard = writer.lock().map_err(|_| ServiceError::LockPoisoned)?;
        formatter.format_fmt(msg, writer_guard.deref_mut())?;
        Ok(())
    }

    /// Returns a reference to the underlying type as [Any] for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<F> Fallback for ArcedFmt<F>
where
    F: MessageFormatter + 'static,
{
    /// Emergency fallback that redirects output to `stdout`.
    ///
    /// If the primary shared writer is inaccessible or failing, the message
    /// is formatted using the standard I/O fallback path.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        if let Ok(mut guard) = self.writer.lock() {
            let mut out = std::io::stdout();
            let _ = guard.formatter.format_io(msg, &mut out);
            let _ = eprintln!("ArcedFmtWriteService Fallback [Error: {}]", error);
        }
    }
}

/// A [`ArcedFmt`] service pre-configured with the [`StandardMessageFormatter`].
///
/// This type is commonly used as a catch-all for string-based logging where
/// maximum flexibility is required for the output destination.
pub type StandardArcedFmt = ArcedFmt<StandardMessageFormatter>;
