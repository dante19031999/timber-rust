// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#[cfg(feature = "json")]
use serde_json::Value;
use crate::{Message, MessageFactory};
use std::borrow::Cow;
use std::error::Error;

/// The [`Loggable`] trait acts as a compile-time dispatcher for the [`Logger`][`crate::Logger`].
///
/// It utilizes Rust's monomorphization (static dispatch) to provide a
/// pseudo-overloaded API. Each implementation handles a specific "Log Shape,"
/// ensuring high performance and clear type separation without runtime overhead.
pub trait Loggable {
    /// Converts the implementing type into a unified [`Message`] object.
    fn to_message(self) -> Message;
}

/// Identity implementation: Allows pre-constructed [`Message`]s to be logged directly.
impl Loggable for Message {
    fn to_message(self) -> Message {
        self
    }
}

/// Implementation for static string slices.
/// This is the "Hot Path"—zero allocation is required for the message content.
impl<S: Into<Cow<'static, str>>> Loggable for (S, &'static str) {
    fn to_message(self) -> Message {
        MessageFactory::string_msg(self.0, self.1)
    }
}

/// Implementation for owned [`String`]s.
/// Takes ownership of the string, moving it into the Message without copying.
impl<S: Into<Cow<'static, str>>> Loggable for (S, String) {
    fn to_message(self) -> Message {
        MessageFactory::string_msg(self.0, self.1)
    }
}

/// Implementation for Cow (Copy-on-Write) strings.
/// Handles both borrowed and owned data efficiently.
impl<S: Into<Cow<'static, str>>> Loggable for (S, Cow<'static, str>) {
    fn to_message(self) -> Message {
        MessageFactory::string_msg(self.0, self.1)
    }
}

/// Implementation for String references.
/// Performs a `.clone()` to satisfy the `'static` requirement of the [`Logger`][`crate::Logger`].
impl<S: Into<Cow<'static, str>>> Loggable for (S, &String) {
    fn to_message(self) -> Message {
        MessageFactory::string_msg(self.0, self.1.clone())
    }
}

/// Implementation for structured JSON data.
/// **Requires** feature `json`.
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
impl<S: Into<Cow<'static, str>>> Loggable for (S, Value) {
    fn to_message(self) -> Message {
        MessageFactory::json_msg(self.0, self.1)
    }
}

/// Implementation for Standard Library Errors.
/// Wraps the error in a [`Box`] for rich exception logging.
impl<S: Into<Cow<'static, str>>> Loggable for (S, Box<dyn Error + Send + Sync>) {
    fn to_message(self) -> Message {
        MessageFactory::error_msg(self.0, self.1)
    }
}