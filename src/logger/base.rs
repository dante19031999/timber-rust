// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::{LoggerStatus, Message};
use crate::logger::loggable::Loggable;
use std::any::Any;
use std::sync::Arc;

/// The core interface for logging backends (e.g., File, Console, Network).
///
/// Implementors must be [`Send`] and [`Sync`] to allow the [Logger] to be shared
/// across multiple threads. The [`Any`] bound enables runtime type introspection
/// via the `as_any` method.
pub trait LoggerImpl: Send + Sync + Any {
    /// Returns the current [LoggerStatus] of the logging backend.
    ///
    /// This provides a quick check to see if the logging service is [`Running`][`LoggerStatus::Running`]
    /// or has become [`Broken`][`LoggerStatus::Broken`] (e.g., due to a disk failure or network disconnection).
    /// The specific behavior is determined by the implementation.
    fn status(&self) -> LoggerStatus;

    /// Processes a single [Message]. The specific behavior (writing to disk,
    /// printing, etc.) is determined by the implementation.
    fn log(&self, message: Message);

    /// Returns a reference to the underlying type as [Any] for downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// A thread-safe, high-level wrapper around a logging implementation.
///
/// [`Logger`] acts as the primary entry point for the application. It uses
/// an [`Arc`] internally to allow cheap cloning and safe sharing across
/// asynchronous tasks or threads.
///
/// Loggers can be easily built using [`LoggerFactory`][`crate::LoggerFactory`].
///
/// Because it uses an [`Arc`] internally, cloning a [Logger] is extremely cheap—
/// it only increments a reference count. This allows you to pass the logger
/// into multiple threads, closures, or asynchronous tasks easily.
#[derive(Clone)]
pub struct Logger {
    m_impl: Arc<dyn LoggerImpl + Send + Sync>,
}

impl Logger {
    /// Creates a new [`Logger`] instance with the specified [backend implementation][`crate::LoggerImpl`].
    ///
    /// # Example
    /// ```
    /// # use timber_rust::Logger;
    /// # use timber_rust::SilentLogger;
    /// let logger = Logger::new(SilentLogger::new());
    /// ```
    pub fn new(implementation: Box<dyn LoggerImpl + Send + Sync>) -> Logger {
        Logger {
            m_impl: Arc::from(implementation),
        }
    }

    /// Returns the current [LoggerStatus] of the underlying logging backend.
    ///
    /// This provides a quick check to see if the logging service is [`Running`][`LoggerStatus::Running`]
    /// or has become [`Broken`][`LoggerStatus::Broken`] (e.g., due to a disk failure or network disconnection).
    pub fn status(&self) -> LoggerStatus {
        self.m_impl.status()
    }

    /// Logs an entry that implements the [Loggable] trait.
    ///
    /// This method uses static dispatch (monomorphization) to resolve the
    /// input `T` into a [Message] before passing it to the internal implementation.
    ///
    /// The enumeration [`LogLevel`][`crate::LogLevel`] implements [`Display`][`std::fmt::Display`] and can be used as levels.
    ///
    /// # Returns
    /// Returns a reference to `self` to enable **Method Chaining** (Fluent API).
    ///
    /// # Example
    /// ```
    /// # use timber_rust::Logger;
    /// # use timber_rust::SilentLogger;
    /// # use timber_rust::LogLevel;
    /// # let logger = Logger::new(SilentLogger::new());
    /// logger.log((LogLevel::Info, "Step 1"))
    ///       .log((LogLevel::Info, "Step 2"));
    /// ```
    ///
    /// # Arguments
    /// * `message` - Anything that can be converted into a log message,
    ///   typically a tuple such as `(Level::Info, "Hello World")`.
    pub fn log<T: Loggable>(&self, message: T) -> &Logger {
        self.m_impl.log(message.to_message());
        self
    }

    /// Provides access to the underlying [LoggerImpl].
    ///
    /// This is useful for accessing backend-specific methods or performing
    /// downcasts via [Any].
    pub fn get_implementation(&self) -> &dyn LoggerImpl {
        &*self.m_impl
    }

}
