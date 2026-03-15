// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::{LoggerImpl, Message, LoggerStatus};
use std::any::Any;

/// A no-op (no-operation) implementation of [`LoggerImpl`].
///
/// [`SilentLogger`][`Silent`] satisfies the logger interface while effectively discarding all
/// log messages. This is useful for:
/// - **Testing**: Disabling log output during unit tests.
/// - **Defaults**: Providing a safe, "do-nothing" fallback logger.
/// - **Performance**: Avoiding conditional "if logger.is_some()" checks in hot paths.
pub struct Silent;

impl LoggerImpl for Silent {
    /// Always returns [`LoggerStatus::Running`].
    ///
    /// Since the [`SilentLogger`][`Silent`] cannot fail in its mission to do nothing, it
    /// is perpetually in a healthy state.
    fn status(&self) -> LoggerStatus {
        LoggerStatus::Running
    }

    /// Discards the incoming [`Message`] immediately.
    ///
    /// This method is essentially a "black hole." The underscore in `_message`
    /// ensures the compiler does not warn about the unused variable while
    /// the message is dropped at the end of the scope.
    fn log(&self, _message: Message) {}

    /// Returns the [`SilentLogger`][`Silent`] instance as a [dyn Any].
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Silent {
    /// Creates a new, heap-allocated instance of [`SilentLogger`][`Silent`].
    ///
    /// This is a convenience constructor that returns the logger inside a [`Box`],
    /// making it directly compatible with [`Logger::new()`][crate::Logger::new()].
    ///
    /// # Example
    /// ```
    /// # use timber_rust::Logger;
    /// # use timber_rust::SilentLogger;
    /// let logger = Logger::new(SilentLogger::new());
    /// ```
    pub fn new() -> Box<Self> {
        Box::new(Silent {})
    }
}
