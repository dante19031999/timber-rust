// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::service::FvnBuildHasher;
use crate::{logger::Loggable, Logger, LoggerFactory};
use std::collections::HashMap;

/// [`LogManager`] acts as a registry and dispatcher for multiple logging channels.
///
/// It maintains a `default_logger` for general use and a [`HashMap`] of specialized
/// channels, using a high-performance FNV hasher for fast lookups.
///
/// ### Key Features
/// * **Default Channel**: The default channels is always set. By default, an implementation of [`crate::SilentLogger`].
/// * **Channel Routing**: Dispatch logs to specific destinations via string identifiers.
/// * **Fluent Interface**: Methods like [`log()`][LogManager::log()] return `&Self` to allow for chaining.
/// * **FVN-1 Hash**: Uses FVN-1 hash to store the logs. Be careful about collisions!
pub struct LogManager {
    default_logger: Logger,
    /// Uses a non-cryptographic FVN hasher for O(1) lookups on channel names.
    channel_loggers: HashMap<String, Logger, FvnBuildHasher>,
}

impl LogManager {
    /// Creates a new [`LogManager`] with a [`SilentLogger`][crate::SilentLogger] implementation as the default.
    /// Logs sent to a new manager will be discarded until a default is set.
    pub fn new() -> Self {
        LogManager {
            default_logger: LoggerFactory::silent(),
            channel_loggers: HashMap::default(),
        }
    }

    /// Creates a manager with a pre-configured default [`Logger`].
    pub fn new_default(default_logger: Logger) -> Self {
        LogManager {
            default_logger,
            channel_loggers: HashMap::default(),
        }
    }

    /// Retrieves a [`Logger`] for a specific channel.
    ///
    /// If the channel name is `"default"`, it returns the default [`Logger`].
    /// Returns [`None`] if the requested channel has not been registered.
    ///
    /// # Arguments
    /// * `channel` - A string slice or type that can be referenced as a string.
    pub fn get_logger<Q>(&self, channel: &Q) -> Option<Logger>
    where
        Q: AsRef<str> + ?Sized,
    {
        let channel = channel.as_ref();
        if channel == "default" {
            return Some(self.default_logger.clone());
        }
        self.channel_loggers.get(channel).cloned() // Standard way to clone the inner Logger
    }

    /// Registers or updates a [`Logger`] for a specific channel name.
    ///
    /// Using the name `"default"` will overwrite the `default_logger`.
    pub fn set_logger<S>(&mut self, channel: S, logger: Logger)
    where
        S: Into<String>,
    {
        let channel = channel.into();
        if channel == "default" {
            self.default_logger = logger;
        } else {
            self.channel_loggers.insert(channel, logger);
        }
    }

    /// Registers or updates a [`Logger`] for a specific channel name.
    ///
    /// Using the name `"default"` will do nothing and return [`None`].
    pub fn remove_logger<Q>(&mut self, channel: &Q) -> Option<Logger>
    where
        Q: AsRef<str> + ?Sized,
    {
        let channel = channel.as_ref();
        if channel != "default" {
            return self.channel_loggers.remove(channel);
        }
        None
    }

    /// Returns a cloned instance of the current primary (default) [`Logger`].
    pub fn get_default_logger(&self) -> Logger {
        self.default_logger.clone()
    }

    /// Directly updates the default [`Logger`], bypassing the channel map.
    pub fn set_default_logger(&mut self, default_logger: Logger) {
        self.default_logger = default_logger;
    }

    /// Dispatches a message to the default [`Logger`].
    ///
    /// # Example
    /// ```
    /// # use timber_rust::{LogLevel, LogManager};
    /// let log_manager = LogManager::new();
    /// log_manager.log(("INFO","System started"))
    ///     .log((LogLevel::Info,"Initializing..."));
    /// ```
    pub fn log<T: Loggable>(&self, message: T) -> &Self {
        let message = message.to_message();
        self.default_logger.log(message);
        self
    }

    /// Dispatches a message to a specific named channel.
    ///
    /// **Warning**: If the channel does not exist, the log will be silently dropped.
    /// Use [`get_logger()`][LogManager::get_logger()] beforehand if you need to guarantee delivery.
    ///
    /// # Example
    /// ```
    /// # use timber_rust::{LogLevel, LogManager};
    /// let log_manager = LogManager::new();
    /// log_manager.log_channel("default", ("INFO","System started"))
    ///     .log_channel("default", (LogLevel::Info,"Initializing..."));
    /// ```
    pub fn log_channel<Q, L>(&self, channel: &Q, message: L) -> &Self
    where
        Q: AsRef<str> + ?Sized,
        L: Loggable,
    {
        let message = message.to_message();
        if let Some(logger) = self.get_logger(channel) {
            logger.log(message);
        }
        self
    }
}
