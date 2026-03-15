// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#[cfg(feature = "loki")]
use crate::service::LokiConfig;
use crate::service::{FeatureDisabledError, FvnBuildHasher, ServiceError};
use crate::{LogManager, LoggerFactory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::BufWriter;

/// Defines the execution strategy for log processing and delivery.
///
/// This setting determines whether the logging operations will block the
/// current thread or run concurrently using an asynchronous runtime.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Concurrency {
    /// Synchronous execution.
    ///
    /// Logging operations are performed on the caller's thread. The program
    /// execution will wait until the log is processed/sent before continuing.
    /// Recommended for CLI tools or simple scripts where latency is not critical.
    Sync,

    /// Asynchronous execution.
    ///
    /// Logging operations are offloaded to an async task. This prevents
    /// blocking the main application flow, making it suitable for high-performance
    /// servers and applications using runtimes like `tokio` or `async-std`.
    Async,
}

impl Display for Concurrency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Sync => write!(f, "sync"),
            Self::Async => write!(f, "async"),
        }
    }
}

/// Represents the destination and configuration for a logging channel.
///
/// This enum defines where log entries are sent and how they are processed.
/// It supports various outputs ranging from standard streams to cloud-based
/// collectors like Grafana Loki.
///
/// See [`LogManager`] (represents a channel).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigEntry {
    /// A "black hole" destination.
    /// All logs sent to this channel are silently discarded.
    ///
    /// See: [`SilentLogger`][`crate::SilentLogger`].
    Null {},

    /// Standard Output (stdout).
    /// Logs are printed directly to the terminal's standard output stream.
    ///
    /// - See: [`CoutWriteService`][`crate::service::CoutWriteService`].
    /// - See:  [`DirectLogger`][`crate::DirectLogger`]
    /// - See:  [`QueuedLogger`][`crate::QueuedLogger`]
    StdOut {
        /// The execution strategy: "sync" (blocking) or "async" (non-blocking).
        concurrency: Concurrency,
    },

    /// Standard Error (stderr).
    /// Logs are printed to the terminal's standard error stream, typically
    /// used for diagnostics or errors.
    ///
    /// - See: [`CerrWriteService`][`crate::service::CerrWriteService`].
    /// - See:  [`DirectLogger`][`crate::DirectLogger`]
    /// - See:  [`QueuedLogger`][`crate::QueuedLogger`]
    StdErr {
        /// The execution strategy: "sync" (blocking) or "async" (non-blocking).
        concurrency: Concurrency,
    },

    /// Unbuffered File Output.
    /// Logs are written directly to a file on disk. Each write is typically
    /// immediate, ensuring data integrity at the cost of higher I/O overhead.
    ///
    /// - See: [`FileWriteService`][`crate::service::FileWriteService`].
    /// - See:  [`DirectLogger`][`crate::DirectLogger`]
    /// - See:  [`QueuedLogger`][`crate::QueuedLogger`]
    File {
        /// The execution strategy: "sync" (blocking) or "async" (non-blocking).
        concurrency: Concurrency,
        /// The system path where the log file will be created or appended to.
        path: String,
    },

    /// Performance-Optimized Buffered File Output.
    /// Logs are accumulated in a memory buffer before being flushed to disk.
    ///
    /// ### ⚠️ Warning
    /// Use with caution. Because logs are held in memory, a sudden application
    /// crash or panic may result in the loss of the most recent log entries.
    ///
    /// Unbuffered File Output.
    /// Logs are written directly to a file on disk. Each write is typically
    /// immediate, ensuring data integrity at the cost of higher I/O overhead.
    ///
    /// - See: [`IoWriteService`][`crate::service::IoWriteService`].
    /// - See: [`std::fs::File`]
    /// - See: [`std::io::BufWriter`]
    /// - See:  [`DirectLogger`][`crate::DirectLogger`]
    /// - See:  [`QueuedLogger`][`crate::QueuedLogger`]
    BufferedFile {
        /// The execution strategy: "sync" (blocking) or "async" (non-blocking).
        concurrency: Concurrency,
        /// The system path where the log file will be created or appended to.
        path: String,
    },

    /// Memory Buffer.
    /// Captures logs into an internal string buffer, useful for testing
    /// or displaying logs within an application UI.
    ///
    /// - See: [`StringWriteService`][`crate::service::StringWriteService`].
    /// - See:  [`DirectLogger`][`crate::DirectLogger`]
    /// - See:  [`QueuedLogger`][`crate::QueuedLogger`]
    String {
        /// The execution strategy: "sync" (blocking) or "async" (non-blocking).
        concurrency: Concurrency,
    },

    /// Grafana Loki Integration.
    /// Pushes logs to a remote Loki instance via HTTP/HTTPS.
    ///
    /// This variant is only available when the `loki` feature is enabled.
    ///
    /// - See: [`LokiService`][`crate::service::LokiService`].
    /// - See: [`LokiConfig`][`crate::service::LokiConfig`].
    /// - See:  [`DirectLogger`][`crate::DirectLogger`]
    /// - See:  [`QueuedLogger`][`crate::QueuedLogger`]
    #[cfg(feature = "loki")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
    Loki {
        /// Detailed connection and authentication settings for the Loki service.
        config: LokiConfig,
    },

    /// Placeholder for missing functionality.
    /// Used when a configuration specifies a model (like `loki`) but the
    /// required crate feature was not enabled at compile time.
    DisabledFeature {
        /// The execution strategy: "sync" (blocking) or "async" (non-blocking).
        concurrency: Concurrency,
        /// The name of the feature that is currently missing.
        feature: String,
    },
}

/// Represents a configuration profile for a [`LogManager`].
/// Allows to preconfigure a [`LogManager`] within a given freedom range.
/// For indexing custom user tools, a composition pattern will be necessary.
///
/// # Serialization and deserialization
///
/// Serialization and deseraitliaziton are done thought [serde](https://docs.rs/crate/serde/latest).
/// This class implements [`serde::Serialize`] and [`serde::Deserialize`].
///
/// # Configuration Schema
///
/// The root configuration is an object containing a list of logging channels.
/// In this documentation we use JSON as example, but supports basic serde serialization/deserialization.
///
/// ## Root Structure
/// ```json
/// {
///   "main": { "model": "stdout", "concurrency": "sync" },
///   "audit_log": { "model": "file", "path": "audit.log", "concurrency": "async" }
/// }
/// ```
///
/// ## Channel Models
///
/// | Model | Additional Fields | Description |
/// | :--- | :--- | :--- |
/// | `null` | None | Discards all log entries. |
/// | `stdout` | None | Prints logs to standard output. |
/// | `stderr` | None | Prints logs to standard error. |
/// | `string` | None | Captures logs into an internal string buffer. |
/// | `file` | `path`: String | Writes logs to the specified file path. |
/// | `buffered_file`| `path`: String | Writes logs to a file using a memory buffer for performance. Please be careful with potential log losses on panic. |
/// | `loki` | `config`: Object | Pushes logs to a Grafana Loki instance (Requires `loki` feature). |
///
/// ---
///
/// ## Loki Configuration ([config][`LokiConfig`] object)
///
/// When `model` is set to `"loki"`, the `config` field must be an object with these fields:
///
/// ### Required Fields
/// - **`url`**: (String) The full endpoint for Loki (e.g., `https://loki.example.com`).
///
/// ### Optional Fields
/// - **`app`**: (String) Application label. Default: [`LOKI_DEFAULT_APP`][`crate::service::LOKI_DEFAULT_APP`].
/// - **`job`**: (String) Job label. Default: [`LOKI_DEFAULT_JOB`][`crate::service::LOKI_DEFAULT_JOB`].
/// - **`env`**: (String) Environment label (e.g., `"prod"`, `"dev"`). Default: [`LOKI_DEFAULT_ENV`][`crate::service::LOKI_DEFAULT_ENV`].
/// - **`max_retries`**: (Integer) Number of retry attempts for failed requests. Default: [`LOKI_DEFAULT_RETRIES`][`crate::service::LOKI_DEFAULT_RETRIES`].
/// - **`workers`**: (Integer) Number of concurrent background workers. Default: [`LOKI_DEFAULT_WORKERS`][`crate::service::LOKI_DEFAULT_WORKERS`].
/// - **`connection_timeout`**: (Duration) Timeout for the HTTP connection. Default: [`LOKI_DEFAULT_CONNECTION_TIMEOUT`][`crate::service::LOKI_DEFAULT_CONNECTION_TIMEOUT`].
/// - **`request_timeout`**:(Duration) Timeout for the HTTP request response. Default: [`LOKI_DEFAULT_REQUEST_TIMEOUT`][`crate::service::LOKI_DEFAULT_REQUEST_TIMEOUT`].
///
/// ### Authentication (Choose one)
/// - **`basic_auth`**: (Object) `{ "username": "...", "password": "..." }`. Can be null or absent. See [`BasicAuth`][`crate::service::BasicAuth`].
/// - **`bearer_auth`**: (String) An API token or JWT. Can be null or absent.
///
/// ### Timeouts (optional)
///   - Can be a **Number** (seconds, e.g., `5.5`).
///   - Can be an **Object**: `{ "secs": 30, "nsecs": 0 }`.
///
/// ---
///
/// ## Complete Example
/// ```json
/// {
///   "channels": [
///     "production-loki": {
///       "model": "loki",
///       "config": {
///         "url": "[https://logs.internal.net/loki/api/v1/push](https://logs.internal.net/loki/api/v1/push)",
///         "app": "auth-service",
///         "env": "production",
///         "bearer_auth": "secret-token-123",
///         "request_timeout": 10.0
///       }
///     },
///     "local-file": {
///       "model": "file",
///       "path": "./logs/latest.log" // Careful with relative paths!
///     }
///   ]
/// }
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    entries: HashMap<String, ConfigEntry, FvnBuildHasher>,
}

impl Config {
    /// Builds the [`LogManager`] by borrowing the configuration.
    ///
    /// This method is ideal consumes the [`Config`] object. Normally the configuration
    /// object will be loaded from a file or elsewhere in order to build the [`LogManager`]
    /// and then discarded. Note that [`Config`] implements [`Clone`] in case a persistent
    /// copy is needed.
    ///
    /// ### Behavior
    /// - Takes **ownership** of `self`.
    /// - **Moves** internal data (like strings and configs) into the loggers.
    ///
    /// ### Errors
    /// Returns [`ServiceError`] if:
    /// - A file cannot be opened with "write + append" permissions.
    /// - A channel requires a feature that was not compiled in.
    /// - Other nondescript errors.
    pub fn build(self) -> Result<LogManager, ServiceError> {
        let mut manager = LogManager::new();
        for (channel, entry) in self.entries {
            let logger = match entry {
                ConfigEntry::Null {} => LoggerFactory::silent(),
                ConfigEntry::StdOut { concurrency } => match concurrency {
                    Concurrency::Sync => LoggerFactory::direct_cout(),
                    Concurrency::Async => LoggerFactory::queued_cout(),
                },
                ConfigEntry::StdErr { concurrency } => match concurrency {
                    Concurrency::Sync => LoggerFactory::direct_cerr(),
                    Concurrency::Async => LoggerFactory::queued_cerr(),
                },
                ConfigEntry::File { concurrency, path } => {
                    let file = OpenOptions::new().write(true).append(true).open(path)?;
                    match concurrency {
                        Concurrency::Sync => LoggerFactory::direct_file(file),
                        Concurrency::Async => LoggerFactory::queued_file(file),
                    }
                }
                ConfigEntry::BufferedFile { concurrency, path } => {
                    let file = OpenOptions::new().write(true).append(true).open(path)?;
                    let writer = Box::new(BufWriter::new(file));
                    match concurrency {
                        Concurrency::Sync => LoggerFactory::direct_boxed_io(writer),
                        Concurrency::Async => LoggerFactory::queued_boxed_io(writer),
                    }
                }
                ConfigEntry::String { concurrency } => match concurrency {
                    Concurrency::Sync => LoggerFactory::direct_string(String::new()),
                    Concurrency::Async => LoggerFactory::queued_string(String::new()),
                },
                #[cfg(feature = "loki")]
                ConfigEntry::Loki { config } => LoggerFactory::loki(config),
                ConfigEntry::DisabledFeature {
                    concurrency: _concurrency,
                    feature,
                } => {
                    return Err(ServiceError::FeatureDisabled(FeatureDisabledError::new(
                        feature,
                    )));
                }
            };

            // Register logger
            manager.set_logger(channel, logger);
        }

        Ok(manager)
    }

    /// Returns a reference to the [configuration entry][`ConfigEntry`] for the specified channel.
    ///
    /// Accepts any type that can be referenced as a string (e.g., `&str` or `String`).
    pub fn get_entry<Q>(&self, channel: &Q) -> Option<&ConfigEntry>
    where
        Q: AsRef<str> + ?Sized,
    {
        self.entries.get(channel.as_ref())
    }

    /// Returns a mutable reference to the [configuration entry][`ConfigEntry`] for the specified channel.
    pub fn get_entry_mut<Q>(&mut self, channel: &Q) -> Option<&mut ConfigEntry>
    where
        Q: AsRef<str> + ?Sized,
    {
        self.entries.get_mut(channel.as_ref())
    }

    /// Removes a channel from the configuration and returns its [entry][`ConfigEntry`], if it existed.
    pub fn remove_entry<Q>(&mut self, channel: &Q) -> Option<ConfigEntry>
    where
        Q: AsRef<str> + ?Sized,
    {
        self.entries.remove(channel.as_ref())
    }

    /// Inserts a raw [configuration entry][`ConfigEntry`] for a specific channel.
    /// If the channel already exists, the old configuration is overwritten.
    pub fn insert_entry<S>(&mut self, channel: S, entry: ConfigEntry)
    where
        S: Into<String>,
    {
        self.entries.insert(channel.into(), entry);
    }

    /// Adds a "Black Hole" (Null) logger to the specified channel.
    ///
    /// See: [`ConfigEntry::Null`]
    pub fn insert_entry_null<S>(&mut self, channel: S)
    where
        S: Into<String>,
    {
        self.insert_entry(channel, ConfigEntry::Null {});
    }

    /// Adds a Standard Output (stdout) logger to the specified channel.
    ///
    /// See: [`ConfigEntry::StdOut`]
    pub fn insert_entry_stdout<S>(&mut self, channel: S, concurrency: Concurrency)
    where
        S: Into<String>,
    {
        self.insert_entry(channel, ConfigEntry::StdOut { concurrency });
    }

    /// Adds a Standard Error (stderr) logger to the specified channel.
    ///
    /// See: [`ConfigEntry::StdErr`]
    pub fn insert_entry_stderr<S>(&mut self, channel: S, concurrency: Concurrency)
    where
        S: Into<String>,
    {
        self.insert_entry(channel, ConfigEntry::StdErr { concurrency });
    }

    /// Adds an In-Memory String logger to the specified channel.
    ///
    /// See: [`ConfigEntry::String`]
    pub fn insert_entry_string<S>(&mut self, channel: S, concurrency: Concurrency)
    where
        S: Into<String>,
    {
        self.insert_entry(channel, ConfigEntry::String { concurrency });
    }

    /// Adds a File logger to the specified channel.
    ///
    /// See: [`ConfigEntry::File`]
    pub fn insert_entry_file<S1, S2>(&mut self, channel: S1, concurrency: Concurrency, path: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.insert_entry(
            channel,
            ConfigEntry::File {
                concurrency,
                path: path.into(),
            },
        );
    }

    /// Adds a Buffered File logger to the specified channel.
    ///
    /// See: [`ConfigEntry::BufferedFile`]
    ///
    /// ### ⚠️ Warning: Data Loss
    /// Because this logger uses an internal memory buffer, some log entries may be lost
    /// if the application panics or is killed before the buffer can be flushed to disk.
    pub fn insert_entry_buffered_file<S1, S2>(
        &mut self,
        channel: S1,
        concurrency: Concurrency,
        path: S2,
    ) where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.insert_entry(
            channel,
            ConfigEntry::BufferedFile {
                concurrency,
                path: path.into(),
            },
        );
    }

    /// Adds a Grafana Loki logger to the specified channel.
    /// Only available when the `loki` feature is enabled.
    ///
    /// See: [`ConfigEntry::Loki`]
    #[cfg(feature = "loki")]
    pub fn insert_entry_loki<S>(&mut self, channel: S, config: LokiConfig)
    where
        S: Into<String>,
    {
        self.insert_entry(channel, ConfigEntry::Loki { config });
    }
}
