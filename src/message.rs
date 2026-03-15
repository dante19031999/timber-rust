// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#[cfg(feature = "json")]
use serde_json::Value;
use std::any::Any;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;
use std::time::SystemTime;

/// A trait for [`Message`] implementations that can be formatted and downcast.
///
/// This trait is the core of the `rust_timber` extensibility. It allows the logger
/// to handle standard text, JSON, or complex Error objects through type erasure.
pub trait MessageImpl: Send + Sync + Any {
    /// Returns the log level (e.g., "INFO", "DEBUG") as a displayable object.
    /// Using `&dyn Display` ensures zero-copy for static string levels.
    fn level(&self) -> &dyn Display;

    /// Returns the message body content.
    /// This allows for deferred formatting of complex types like JSON or Error objects.
    fn content(&self) -> &dyn Display;

    /// Returns the exact [SystemTime] when the message was created.
    /// Crucial for maintaining chronological order in asynchronous logging.
    fn instant(&self) -> SystemTime;

    /// Returns a reference to `self` as a [`dyn Any`][std::any::Any] for downcasting purposes.
    /// This allows services to recover the original concrete type if specialized processing is needed.
    fn as_any(&self) -> &dyn Any;
}

/// A standard text-based log message.
///
/// Uses [`Cow<'static, str>`] to avoid heap allocations when using static string literals.
pub struct StringMessageImpl {
    level: Cow<'static, str>,
    content: Cow<'static, str>,
    instant: SystemTime,
}

impl MessageImpl for StringMessageImpl {
    fn level(&self) -> &dyn Display {
        &self.level
    }

    fn content(&self) -> &dyn Display {
        &self.content
    }

    fn instant(&self) -> SystemTime {
        self.instant
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl StringMessageImpl {
    /// Returns a reference to the underlying string content.
    pub fn get_string(&self) -> &str {
        &self.content
    }
}

/// A message implementation that stores structured JSON data.
///
/// This is used when you want to pass raw data to a logger that supports
/// structured output (like a database or an ELK stack).
///
/// # Feature Requirement
/// Only available when the `json` feature is enabled.
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub struct JsonMessageImpl {
    pub(crate) level: Cow<'static, str>,
    pub(crate) content: Value,
    instant: SystemTime,
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
impl MessageImpl for JsonMessageImpl {
    fn level(&self) -> &dyn Display {
        &self.level
    }

    fn content(&self) -> &dyn Display {
        &self.content
    }

    fn instant(&self) -> SystemTime {
        self.instant
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
impl JsonMessageImpl {
    /// Returns a reference to the underlying JSON value.
    pub fn json(&self) -> &Value {
        &self.content
    }
}

/// A message containing a boxed Error object.
///
/// This allows capturing full stack traces and error chains while still
/// satisfying the `Display` requirements of a logger.
pub struct ErrorMessageImpl {
    level: Cow<'static, str>,
    content: Box<dyn Error + Send + Sync>,
    instant: SystemTime,
}

impl MessageImpl for ErrorMessageImpl {
    fn level(&self) -> &dyn Display {
        &self.level
    }

    fn content(&self) -> &dyn Display {
        &self.content
    }

    fn instant(&self) -> SystemTime {
        self.instant
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ErrorMessageImpl {
    /// Returns a reference to the inner trait object for detailed error inspection.
    pub fn error(&self) -> &dyn Error {
        &*self.content
    }
}

/// The public-facing container for any log message.
///
/// This struct wraps a [`Box<dyn MessageImpl>`][MessageImpl], providing a uniform API
/// regardless of the underlying data type.
pub struct Message {
    m_impl: Box<dyn MessageImpl + Send + Sync>,
}

impl Message {
    /// Creates a new message from a concrete implementation.
    /// The implementation is moved into a Box and becomes owned by the Message.
    pub fn new(m_impl: Box<dyn MessageImpl + Send + Sync>) -> Self {
        Message { m_impl }
    }

    /// Accesses the log level (e.g., "INFO").
    /// Returns a dynamic reference to an object implementing Display.
    pub fn level(&self) -> &dyn Display {
        self.m_impl.level()
    }

    /// Accesses the message content.
    /// The formatting is deferred until the Display trait is actually invoked.
    pub fn content(&self) -> &dyn Display {
        self.m_impl.content()
    }

    /// Returns the creation timestamp.
    pub fn instant(&self) -> SystemTime {
        self.m_impl.instant()
    }

    /// Accesses the underlying implementation for downcasting purposes.
    /// This returns a reference to the trait object itself.
    pub fn implementation(&self) -> &dyn MessageImpl {
        &*self.m_impl
    }

    /// Unwraps the message and returns the implementation.
    /// This returns the trait object itself.
    pub fn unwrap(self) -> Box<dyn MessageImpl + Send + Sync> {
        self.m_impl
    }
}

/// The primary entry point for creating log messages.
///
/// Methods are designed to be "allocation-aware," using [`Cow`] to keep
/// static string logging as fast as possible.
pub struct MessageFactory {}

impl MessageFactory {
    /// Creates a [text-based message][`StringMessageImpl`].
    ///
    /// If passed `&'static str`, no heap allocation occurs for the strings.
    pub fn string_msg<S1, S2>(level: S1, content: S2) -> Message
    where
        S1: Into<Cow<'static, str>>,
        S2: Into<Cow<'static, str>>,
    {
        Message {
            m_impl: Box::new(StringMessageImpl {
                level: level.into(),
                content: content.into(),
                instant: SystemTime::now(),
            }),
        }
    }

    /// Creates a [json-based message][`JsonMessageImpl`] from a json `Value`.
    ///
    /// # Feature Requirement
    /// Only available when the `json` feature is enabled.
    #[cfg(feature = "json")]
    pub fn json_msg<S>(level: S, content: Value) -> Message
    where
        S: Into<Cow<'static, str>>,
    {
        Message {
            m_impl: Box::new(JsonMessageImpl {
                level: level.into(),
                content: content.into(),
                instant: SystemTime::now(),
            }),
        }
    }

    /// Creates a [error-based message][`ErrorMessageImpl`] from a boxed `Error`.
    pub fn error_msg<S>(level: S, content: Box<dyn Error + Send + Sync>) -> Message
    where
        S: Into<Cow<'static, str>>,
    {
        Message {
            m_impl: Box::new(ErrorMessageImpl {
                level: level.into(),
                content,
                instant: SystemTime::now(),
            }),
        }
    }
}
