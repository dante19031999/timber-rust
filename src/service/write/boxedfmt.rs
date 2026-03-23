use crate::{Fallback, LoggerStatus, Message, Service};
use std::any::Any;
use std::sync::Mutex;
use crate::service::ServiceError;
use crate::service::write::{StandardMessageFormatter, MessageFormatter};

/// A private synchronization container for heap-allocated string writers.
///
/// Unlike [`crate::service::write::fmt::FmtWriteServiceData`], this struct explicitly holds a trait object
/// ([`Box<dyn std::fmt::Write>`]). This separation allows us to handle the unique
/// borrowing requirements of boxed trait objects within the [`Service`] implementation.
struct BoxedFmtServiceData<F>
where
    F: MessageFormatter,
{
    /// A heap-allocated, dynamically dispatched writer implementing [`std::fmt::Write`].
    /// Must be [`Send`] + [`Sync`] to allow the [`Service`] to move between threads.
    writer: Box<dyn std::fmt::Write + Send + Sync>,
    /// The strategy used to format the [`Message`].
    formatter: F,
}

/// A specialized [`Service`] for dynamically dispatched string-based logging.
///
/// ### Why this exists (The "Orphan Rule" Workaround)
/// In Rust, [`std::fmt::Write`] is not implemented for `Box<dyn std::fmt::Write>`.
/// While we could use a type alias for byte-based writers ([`BoxedFmt`] service),
/// doing so for string-based writers would require implementing a foreign trait on
/// a foreign type, which is forbidden by Rust's "Orphan Rules."
///
/// [`BoxedFmt`] service solves this by providing a concrete struct that "wraps"
/// the boxed trait object, allowing us to manually dispatch the write calls in the
/// [`work`](Self::work) method.
///
///
pub struct BoxedFmt<F>
where
    F: MessageFormatter,
{
    /// Mutex-protected storage for the boxed writer and formatter.
    writer: Mutex<BoxedFmtServiceData<F>>,
}

impl<F> BoxedFmt<F>
where
    F: MessageFormatter,
{
    /// Creates a new [`BoxedFmt`] service on the heap.
    ///
    /// # Parameters
    /// - `writer`: A boxed trait object. This is useful when the exact type of
    ///   the string writer (e.g., a custom UI buffer vs. a standard [`String`])
    ///   is not known at compile time.
    /// - `formatter`: The [`MessageFormatter`] implementation.
    pub fn new(writer: Box<dyn std::fmt::Write + Send + Sync>) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(BoxedFmtServiceData {
                writer,
                formatter: Default::default(),
            }),
        })
    }

    /// Creates a new [`BoxedFmt`] service on the heap with a custom [formatter][`MessageFormatter`].
    ///
    /// # Parameters
    /// - `writer`: A boxed trait object. This is useful when the exact type of
    ///   the string writer (e.g., a custom UI buffer vs. a standard [`String`])
    ///   is not known at compile time.
    /// - `formatter`: The [`MessageFormatter`] implementation.
    pub fn with_formatter(
        writer: Box<dyn std::fmt::Write + Send + Sync>,
        formatter: F,
    ) -> Box<Self> {
        Box::new(Self {
            writer: Mutex::new(BoxedFmtServiceData { writer, formatter }),
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
        f: impl FnOnce(&Box<dyn std::fmt::Write + Send + Sync>) -> R,
    ) -> Option<R> {
        self.writer.lock().ok().map(|data| f(&data.writer))
    }

    /// Destroys the [`Service`] and reclaims ownership of the underlying buffer or writer.
    ///
    /// Use this at the end of a program, a test case, or a lifecycle stage to extract
    /// all recorded logs and free up the resources used by the [`Service`].
    ///
    /// ### Ownership & Lifecycle
    /// This method consumes `self`, meaning the [`BoxedFmt`] service can no longer be
    /// used after this call. This is the only way to get full, non-cloned ownership
    /// of the internal writer (e.g., a [`String`] or [`Vec<u8>`]).
    ///
    /// ### Guarantees
    /// Because this takes ownership of the [`Service`], it is compile-time guaranteed
    /// that no other threads can be writing to the buffer when this is called.
    pub fn take_writer(self) -> Result<Box<dyn std::fmt::Write + Send + Sync>, ServiceError> {
        let data = self.writer.into_inner();
        match data {
            Ok(data) => Ok(data.writer),
            Err(_) => Err(ServiceError::LockPoisoned),
        }
    }
}

impl<F> Service for BoxedFmt<F>
where
    F: MessageFormatter + 'static,
{
    /// Returns the current operational status.
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Acquires the lock and dispatches the write to the boxed trait object.
    ///
    /// # Internal Mechanics
    /// Since `Box<dyn std::fmt::Write>` doesn't implement [`std::fmt::Write`], this method uses
    /// [`Box::as_mut()`] to obtain a mutable reference to the underlying
    /// trait object before passing it to the formatter.
    ///
    /// # Errors
    /// - [`ServiceError::LockPoisoned`] if the mutex is poisoned
    /// - Forwards any [`ServiceError`] returned by the formatter.
    fn work(&self, msg: &Message) -> Result<(), ServiceError> {
        let mut guard = self.writer.lock()?;

        // Destructure the internal data
        let BoxedFmtServiceData {
            formatter, writer, ..
        } = &mut *guard;

        // Manual dispatch: conversion from Box<dyn Write> to &mut dyn Write
        formatter.format_fmt(msg, writer.as_mut())?;
        Ok(())
    }

    /// Returns a reference to the underlying type as [Any] for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<F> Fallback for BoxedFmt<F>
where
    F: MessageFormatter + 'static,
{
    /// Emergency fallback that redirects output to `stdout`.
    ///
    /// If the primary boxed writer is inaccessible or failing, the message
    /// is formatted using the standard I/O fallback path.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        if let Ok(mut guard) = self.writer.lock() {
            let mut out = std::io::stdout();
            let _ = guard.formatter.format_io(msg, &mut out);
            let _ = eprintln!("BoxedFmtWriteService Fallback [Error: {}]", error);
        }
    }
}

/// A [`BoxedFmt`] service pre-configured with the [`StandardMessageFormatter`].
///
/// This type is commonly used as a catch-all for string-based logging where
/// maximum flexibility is required for the output destination.
pub type StandardBoxedFmt = BoxedFmt<StandardMessageFormatter>;
