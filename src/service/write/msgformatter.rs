// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::Message;
use crate::service::ServiceError;
use chrono::{DateTime, SecondsFormat, Utc};

/// Trait defining the behavior for formatting log messages.
///
/// Implementations are responsible for defining the layout (timestamp, level, content)
/// and writing the result to an I/O sink.
pub trait MessageFormatter: Send + Sync + Default {
    /// Formats and writes the message to the provided I/O sink.
    ///
    /// ### Implementation Requirements
    /// - **Atomicity**: To ensure log integrity in concurrent environments, implementations
    ///   should minimize the number of calls to the writer. Using a single `write!` macro
    ///   or a buffered approach is highly recommended.
    /// - **Thread Safety**: The `writer` is guaranteed to be `Send + Sync`. However,
    ///   some global sinks (like `std::io::stdout()`) may not support explicit locking
    ///   while maintaining these bounds.
    ///
    /// # Errors
    /// Returns [`ServiceError`] if formatting fails or the writer encounters an I/O error.
    fn format_io(
        &mut self,
        message: &Message,
        write: &mut (dyn std::io::Write + Send + Sync),
    ) -> Result<(), ServiceError>;

    /// Formats a message specifically for fmt-based destinations.
    fn format_fmt(
        &mut self,
        message: &Message,
        write: &mut (dyn std::fmt::Write + Send + Sync),
    ) -> Result<(), ServiceError>;
}

/// A high-performance, stateful formatter that produces RFC 3339 compliant logs.
///
/// ### Output Format
/// The formatter produces a single line per message using the following pattern:
/// `[Timestamp] [ [Level] ] [Message]`
///
/// * **Timestamp**: Generated in UTC using [RFC 3339](https://tools.ietf.org/html/rfc3339)
///   format with nanosecond precision (e.g., `2026-03-14T14:48:02.609225083+00:00`).
/// * **Level**: The log level is uppercase, padded with single spaces inside brackets
///   (e.g., `[ DEBUG ]`, `[ INFO  ]`, `[ ERROR ]`).
/// * **Message**: The raw message content followed by a newline (`\n`).
///
/// ### Example Output
/// ```text
/// 2026-03-14T15:30:00.123456789+00:00 [ INFO  ] Service started successfully
/// 2026-03-14T15:30:05.000000000+00:00 [ DEBUG ] Connecting to Loki at localhost:3100
/// ```
#[derive(Default)]
pub struct DefaultMessageFormatter {}

impl DefaultMessageFormatter {
    /// Creates a formatter with a default buffer capacity of 128 bytes.
    pub fn new() -> Self {
        DefaultMessageFormatter {}
    }
}

impl MessageFormatter for DefaultMessageFormatter {
    fn format_io(
        &mut self,
        message: &Message,
        write: &mut (dyn std::io::Write + Send + Sync),
    ) -> Result<(), ServiceError> {
        let instant: DateTime<Utc> = message.instant().into();
        write!(
            write,
            "{} [ {} ] {}\n",
            instant.to_rfc3339_opts(SecondsFormat::Nanos, true),
            message.level(),
            message.content()
        )?;
        Ok(())
    }

    fn format_fmt(
        &mut self,
        message: &Message,
        write: &mut (dyn std::fmt::Write + Send + Sync),
    ) -> Result<(), ServiceError> {
        let instant: DateTime<Utc> = message.instant().into();
        write!(
            write,
            "{} [ {} ] {}\n",
            instant.to_rfc3339_opts(SecondsFormat::Nanos, true),
            message.level(),
            message.content()
        )?;
        Ok(())
    }
}

/// A high-performance, stateful formatter that produces undated logs.
///
/// ### Output Format
/// The formatter produces a single line per message using the following pattern:
/// `[ [Level] ] [Message]`
///
/// * **Level**: The log level is uppercase, padded with single spaces inside brackets
///   (e.g., `[ DEBUG ]`, `[ INFO  ]`, `[ ERROR ]`).
/// * **Message**: The raw message content followed by a newline (`\n`).
///
/// ### Example Output
/// ```text
/// [ INFO  ] Service started successfully
/// [ DEBUG ] Connecting to Loki at localhost:3100
/// ```
#[derive(Default)]
pub struct AtemporalMessageFormatter {}

impl AtemporalMessageFormatter {
    /// Creates a formatter with a default buffer capacity of 128 bytes.
    pub fn new() -> Self {
        AtemporalMessageFormatter {}
    }
}

impl MessageFormatter for AtemporalMessageFormatter {
    fn format_io(
        &mut self,
        message: &Message,
        write: &mut (dyn std::io::Write + Send + Sync),
    ) -> Result<(), ServiceError> {
        write!(write, "[ {} ] {}\n", message.level(), message.content())?;
        Ok(())
    }

    fn format_fmt(
        &mut self,
        message: &Message,
        write: &mut (dyn std::fmt::Write + Send + Sync),
    ) -> Result<(), ServiceError> {
        write!(write, "[ {} ] {}\n", message.level(), message.content())?;
        Ok(())
    }
}
