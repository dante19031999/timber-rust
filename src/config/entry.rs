// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#[cfg(feature = "loki")]
use crate::config::duration::FlexibleDuration;
#[cfg(feature = "aws")]
use crate::config::timestamp::Timestamp;
#[cfg(feature = "aws")]
use crate::service::CloudWatchConfig;
#[cfg(feature = "loki")]
use crate::service::LokiConfig;
#[cfg(feature = "loki")]
use crate::BasicAuth;
use crate::Concurrency;
use serde::{Deserialize, Serialize};

/// Represents the destination and configuration for a logging channel.
///
/// This enum defines where log entries are sent and how they are processed.
/// It supports various outputs ranging from standard streams to cloud-based
/// collectors like Grafana Loki.
///
/// See [`LogManager`][`crate::LogManager`] (represents a channel).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entry {
    /// A "black hole" destination.
    /// All logs sent to this channel are silently discarded.
    ///
    /// See: [`SilentLogger`][`crate::SilentLogger`].
    Silent {},

    /// Standard Output (stdout).
    /// Logs are printed directly to the terminal's standard output stream.
    ///
    /// - See: [`service::write::Cout`][`crate::service::write::Cout`]
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    StdOut {
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
    },

    /// Standard Error (stderr).
    /// Logs are printed to the terminal's standard error stream, typically
    /// used for diagnostics or errors.
    ///
    /// - See: [`service::write::Cerr`][`crate::service::write::Cerr`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    StdErr {
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
    },

    /// Unbuffered File Output.
    /// Logs are written directly to a file on disk. Each write is typically
    /// immediate, ensuring data integrity at the cost of higher I/O overhead.
    ///
    /// - See: [`service::write::File`][`crate::service::FileWrite`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    File {
        /// Path to the file where to dump the log.
        path: String,
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
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
    /// - See: [`service::write::BufferedFile`][`crate::service::write::BufferedFile`].
    /// - See: [`service::IoWrite`][`crate::service::IoWrite`].
    /// - See: [`std::fs::File`]
    /// - See: [`std::io::BufWriter`]
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    BufferedFile {
        /// Path to the file where to dump the log.
        path: String,
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
    },

    /// Memory Buffer.
    /// Captures logs into an internal string buffer, useful for testing
    /// or displaying logs within an application UI.
    ///
    /// - See: [`service::write::StringFmt`][`crate::service::write::StringFmt`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    String {
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
        /// String capacity
        capacity: Option<usize>,
    },

    /// Memory Buffer (vector).
    /// Captures logs into an internal vector, useful for testing.
    ///
    /// - See: [`service::Vector`][`crate::service::Vector`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    Vector {
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
        /// Vector capacity
        capacity: Option<usize>,
    },

    /// Grafana Loki Integration.
    /// Pushes logs to a remote Loki instance via HTTP/HTTPS.
    ///
    /// This variant is only available when the `loki` feature is enabled.
    ///
    /// - See: [`LokiLogger`][`crate::logger::Loki`].
    /// - See: [`LokiService`][`crate::service::Loki`].
    /// - See: [`LokiConfig`][`crate::service::LokiConfig`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    #[cfg(feature = "loki")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
    Loki {
        /// The full HTTP/HTTPS endpoint for the Loki push API
        /// (e.g., `https://logs-prod-us-central1.grafana.net/loki/api/v1/push`).
        url: String,
        /// The name of the application. This becomes a static label used for
        /// filtering logs in Grafana.
        app: String,
        /// The job name associated with the process. Useful for distinguishing
        /// between different instances of the same application.
        job: String,
        /// The deployment environment (e.g., "production", "staging", "development").
        /// Helps isolate logs across different stages of the lifecycle.
        env: String,
        /// Optional credentials for Basic Authentication.
        basic_auth: Option<BasicAuth>,
        /// Optional Bearer token for API authentication (e.g., for Grafana Cloud).
        bearer_auth: Option<String>,
        /// The maximum time allowed to establish a connection to the Loki server.
        connection_timeout: FlexibleDuration,
        /// The maximum time allowed for a single push request to complete.
        request_timeout: FlexibleDuration,
        /// How many times a failed push should be retried before falling back.
        /// This is handled by the internal dispatch logic.
        max_retries: usize,
        /// The number of background worker threads dedicated to pushing logs.
        /// Higher counts increase throughput but consume more network resources.
        worker_count: usize,
    },

    /// AWS Cloudwatch Integration.
    /// Pushes logs to a remote Cloudwatch Integration instance via AWS SDK.
    ///
    /// This variant is only available when the `aws` feature is enabled.
    ///
    /// - See: [`CloudwatchLogger`][`crate::logger::CloudWatch`].
    /// - See: [`CloudwatchService`][`crate::service::CloudWatch`].
    /// - See: [`CloudwatchConfig`][`crate::service::LokiConfig`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    CloudWatchEnv {
        /// Log group for the cloudwatch service. The config is loaded from ENV.
        log_group: String,
    },

    /// AWS Cloudwatch Integration.
    /// Pushes logs to a remote Cloudwatch Integration instance via AWS SDK.
    ///
    /// This variant is only available when the `aws` feature is enabled.
    ///
    /// - See: [`CloudwatchLogger`][`crate::logger::CloudWatch`].
    /// - See: [`CloudwatchService`][`crate::service::CloudWatch`].
    /// - See: [`CloudwatchConfig`][`crate::service::LokiConfig`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    CloudWatchConfig {
        /// AWS Access Key ID used for authentication.
        access_key_id: String,
        /// AWS Secret Access Key. This field is sensitive and hidden in Debug logs.
        access_key_secret: String,
        /// Optional session token for temporary credentials (STS).
        session_token: Option<String>,
        /// Optional expiration timestamp for the credentials in seconds.
        expires_in: Option<Timestamp>,
        /// The name of the CloudWatch Log Group where logs will be sent.
        log_group: String,
        /// The AWS Region (e.g., "us-east-1").
        region: String,
    },

    /// AWS CloudWatch (via Standard Output).
    ///
    /// Formats logs as single-line JSON objects and prints them to `stdout`.
    /// This is the preferred method for AWS Lambda, ECS (with `awslogs` driver),
    /// and Fargate, as it avoids the overhead of the AWS SDK while maintaining
    /// structured logs.
    ///
    /// - See: [`CloudWatchCout`][`crate::service::CloudWatchCout`].
    /// - See: [`CloudWatchCoutMessageFormatter`][`crate::service::CloudWatchCoutMessageFormatter`].
    /// - See: [`logger::Direct`][`crate::logger::Direct`]
    /// - See: [`logger::Queued`][`crate::`logger::Queued`]
    #[cfg(feature = "awscout")]
    #[cfg_attr(docsrs, doc(cfg(feature = "awscout")))]
    CloudWatchCout {
        /// The execution strategy: [`Concurrency::Sync`] for immediate writes,
        /// or [`Concurrency::Async`] for background-threaded logging.
        concurrency: Concurrency,
        /// Optional override for the maximum number of write retries.
        /// If `None`, the factory default (usually 3) is used.
        max_retries: Option<usize>,
        /// Optional override for the number of background worker threads.
        /// If `None`, the factory default (usually 1) is used.
        /// **Note:** A value of 1 is recommended to prevent interlaced output.
        worker_count: Option<usize>,
    },

    /// Placeholder for missing functionality.
    /// Used when a configuration specifies a model (like `loki`) but the
    /// required crate feature was not enabled at compile time.
    DisabledFeature {
        /// The name of the feature that is currently missing.
        feature: String,
    },
}

impl Entry {
    /// Creates a configuration for a silent logger that discards all input.
    ///
    /// Useful as a placeholder or for completely disabling output in specific environments.
    pub fn silent() -> Self {
        Entry::Silent {}
    }

    /// Configures logging to the Standard Output stream (stdout).
    ///
    /// This is the primary target for CLI applications and containerized services.
    pub fn stdout(concurrency: Concurrency) -> Self {
        Entry::StdOut {
            concurrency,
            max_retries: None,
            worker_count: None,
        }
    }

    /// Configures logging to the Standard Error stream (stderr).
    ///
    /// Recommended for diagnostic messages to keep the main stdout stream clean for data.
    pub fn stderr(concurrency: Concurrency) -> Self {
        Entry::StdErr {
            concurrency,
            max_retries: None,
            worker_count: None,
        }
    }

    /// Configures an in-memory string buffer for captured logs.
    ///
    /// Useful for small-scale log capturing where a full Vector of messages is not required.
    pub fn string(concurrency: Concurrency) -> Self {
        Entry::String {
            concurrency,
            max_retries: None,
            worker_count: None,
            capacity: None,
        }
    }

    /// Configures a raw file logger at the specified path.
    ///
    /// Opens the file in append mode. Ensure the process has appropriate write permissions.
    pub fn file(concurrency: Concurrency, path: String) -> Self {
        Entry::File {
            path,
            concurrency,
            max_retries: None,
            worker_count: None,
        }
    }

    /// Configures a buffered file logger at the specified path.
    ///
    /// Wraps the file in a buffer to reduce the frequency of underlying system calls,
    /// significantly improving performance during high-volume bursts.
    pub fn buffered_file(concurrency: Concurrency, path: String) -> Self {
        Entry::BufferedFile {
            path,
            concurrency,
            max_retries: None,
            worker_count: None,
        }
    }

    /// Configures a logger that captures structured `Message` objects in a `Vec`.
    ///
    /// This is the "gold standard" for unit testing, allowing for precise assertions
    /// on log levels and contents.
    pub fn vector(concurrency: Concurrency) -> Self {
        Entry::Vector {
            concurrency,
            max_retries: None,
            worker_count: None,
            capacity: None,
        }
    }

    /// Integrates a Grafana Loki configuration into the entry list.
    ///
    /// This converts a high-level [`LokiConfig`] into the flat enum representation.
    #[cfg(feature = "loki")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
    pub fn loki(config: LokiConfig) -> Self {
        Entry::Loki {
            url: config.url,
            app: config.app,
            job: config.job,
            env: config.env,
            basic_auth: config.basic_auth,
            bearer_auth: config.bearer_auth,
            connection_timeout: config.connection_timeout.into(),
            request_timeout: config.request_timeout.into(),
            max_retries: config.max_retries,
            worker_count: config.worker_count,
        }
    }

    /// Configures Amazon CloudWatch using explicit credentials.
    ///
    /// Use this when credentials are provided manually or via a secret manager.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn cloudwatch_config(config: CloudWatchConfig) -> Self {
        Entry::CloudWatchConfig {
            access_key_id: config.access_key_id,
            access_key_secret: config.access_key_secret,
            session_token: config.session_token,
            expires_in: config.expires_in.map(|t| Timestamp::from(t)),
            log_group: config.log_group,
            region: config.region,
        }
    }

    /// Configures Amazon CloudWatch using credentials sourced from the environment.
    ///
    /// Standard AWS environment variables (AWS_ACCESS_KEY_ID, etc.) will be used automatically.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn cloudwatch_env(log_group: String) -> Self {
        Entry::CloudWatchEnv { log_group }
    }

    /// Configures a CloudWatch-formatted logger that writes to stdout.
    ///
    /// This allows logs to be formatted for AWS CloudWatch even if they are
    /// being emitted to a local terminal or captured by a separate log agent.
    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    pub fn cloudwatch_cout(concurrency: Concurrency) -> Self {
        Entry::CloudWatchCout {
            concurrency,
            worker_count: None,
            max_retries: None,
        }
    }
}
