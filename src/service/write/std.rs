// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::ServiceError;
use crate::service::fallback::Fallback;
use crate::service::write::{StandardMessageFormatter, MessageFormatter};
use crate::{LoggerStatus, Message, Service};
use std::any::Any;
use std::sync::Mutex;
// =======================================================================
// Cout / Cerr Services
// =======================================================================

/// A logging [`Service`] that targets the standard output stream ([`std::io::stdout`]).
///
/// Unlike other [`Service`]s, [`Cout`] service does not own its writer. Instead, it
/// acquires a handle to the global process `stdout` during every [`work`](Self::work) call.
///
/// ### Why the Mutex?
/// Even though `stdout` is globally available, the [`MessageFormatter`] (field `formatter`)
/// may hold internal state (like line counters or timing data) that is not thread-safe.
/// Wrapping the formatter in a [`Mutex`] ensures that the formatting logic remains
/// atomic and synchronized across threads.
pub struct Cout<F>
where
    F: MessageFormatter,
{
    /// Thread-safe access to the formatting logic.
    formatter: Mutex<F>,
}

impl<F> Cout<F>
where
    F: MessageFormatter,
{
    /// Creates a new [`Cout`] service on the heap.
    pub fn new() -> Box<Self> {
        Box::new(Self {
            formatter: Mutex::new(Default::default()),
        })
    }

    /// Creates a new [`Cout`] service on the heap  with a custom [formatter][`MessageFormatter`].
    pub fn with_formatter(formatter: F) -> Box<Self> {
        Box::new(Self {
            formatter: Mutex::new(formatter),
        })
    }
}

impl<F> Service for Cout<F>
where
    F: MessageFormatter + 'static,
{
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Acquires the formatter lock and streams the formatted message to `stdout`.
    ///
    /// This method locks the global `stdout` stream for the duration of the formatting
    /// process. This prevents "line interleaving" where parts of different log
    /// messages appear mixed in the console.
    ///
    /// # Errors
    /// Returns [`ServiceError::LockPoisoned`] if the internal formatter mutex is poisoned.
    fn work(&self, msg: &Message) -> Result<(), ServiceError> {
        let mut formatter_guard = self.formatter.lock()?;
        let mut out = std::io::stdout();
        formatter_guard.format_io(msg, &mut out)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<F> Fallback for Cout<F>
where
    F: MessageFormatter + 'static,
{
    /// Emergency fallback that attempts to log the error back to `stdout`.
    /// If the formatter is locked or poisoned, the fallback is aborted to avoid deadlocks.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        if let Ok(mut guard) = self.formatter.lock() {
            let mut out = std::io::stdout();
            let _ = guard.format_io(msg, &mut out);
            let _ = eprintln!("CoutWriteService Error: {}", error);
        }
    }
}

/// A [`Cout`] service pre-configured with the [`StandardMessageFormatter`].
pub type StandardCout = Cout<StandardMessageFormatter>;

/// A logging [`Service`] that targets the standard error stream ([`std::io::stderr`]).
///
/// [`Cerr`] service is typically used for high-priority alerts or diagnostic
/// information that should remain visible even if `stdout` is redirected to a file.
pub struct Cerr<F>
where
    F: MessageFormatter,
{
    /// Thread-safe access to the formatting logic.
    formatter: Mutex<F>,
}

impl<F> Cerr<F>
where
    F: MessageFormatter,
{
    /// Creates a new [`Cerr`] on the heap.
    pub fn new() -> Box<Self> {
        Box::new(Self {
            formatter: Mutex::new(Default::default()),
        })
    }

    /// Creates a new [`Cerr`] service on the heap with a custom [formatter][`MessageFormatter`].
    pub fn with_formatter(formatter: F) -> Box<Self> {
        Box::new(Self {
            formatter: Mutex::new(formatter),
        })
    }
}

impl<F> Service for Cerr<F>
where
    F: MessageFormatter + 'static,
{
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Acquires the formatter lock and writes to the global `stderr`.
    ///
    /// # Errors
    /// Returns [`ServiceError::LockPoisoned`] if the internal formatter mutex is poisoned.
    fn work(&self, msg: &Message) -> Result<(), ServiceError> {
        let mut formatter_guard = self.formatter.lock()?;
        let mut out = std::io::stderr();
        formatter_guard.format_io(msg, &mut out)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<F> Fallback for Cerr<F>
where
    F: MessageFormatter + 'static,
{
    /// Fallback for `stderr` failures. Paradoxically attempts to log the
    /// failure to `stdout` as a last-resort communication channel.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        if let Ok(mut guard) = self.formatter.lock() {
            let mut out = std::io::stdout();
            let _ = guard.format_io(msg, &mut out);
            let _ = println!("CerrWriteService Error: {}", error);
        }
    }
}

/// A [`Cerr`] service pre-configured with the [`StandardMessageFormatter`].
pub type StandardCerr = Cerr<StandardMessageFormatter>;
