// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#[cfg(feature = "awscout")]
use crate::factory::awscout::CloudWatch;
use crate::factory::fmt::FmtWrite;
use crate::factory::io::IoWrite;
#[cfg(feature = "loki")]
use crate::factory::loki::Loki;
use crate::factory::std::CerrWrite;
use crate::factory::std::CoutWrite;
use crate::factory::vec::Vector;
use crate::{Logger, SilentLogger};

/// The primary entry point for the Timber logging system.
///
/// `LoggerFactory` provides a centralized interface for creating specialized 
/// loggers. It uses a fluent builder pattern to allow for granular configuration 
/// of retry policies, concurrency models, and destination-specific settings.
///
/// ### Example: Console Logging
/// ```rust
/// # use timber_rust::LoggerFactory;
/// # use timber_rust::Concurrency;
/// let logger = LoggerFactory::cout()
///     .max_retries(5)
///     .build(Concurrency::Async);
/// ```
///
/// ### Example: Capturing Logs for Testing
/// ```rust
/// # use timber_rust::LoggerFactory;
/// # use timber_rust::Concurrency;
/// let logger = LoggerFactory::vector()
///     .capacity(500)
///     .build(Concurrency::Sync);
/// ```
pub struct LoggerFactory {}

impl LoggerFactory {
    /// Returns a "No-Op" logger that discards all messages.
    ///
    /// Useful for silencing output in production environments or as a 
    /// default placeholder in library configuration.
    pub fn silent() -> Logger {
        Logger::new(SilentLogger::new())
    }

    /// Creates a builder for Grafana Loki.
    ///
    /// Requires the `loki` feature to be enabled.
    #[cfg(feature = "loki")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
    pub fn loki() -> Loki {
        Loki {}
    }

    /// Creates a builder for Amazon CloudWatch.
    ///
    /// Requires the `aws` feature to be enabled.
    #[cfg(feature = "awscout")]
    #[cfg_attr(docsrs, doc(cfg(feature = "awscout")))]
    pub fn cloudwatch() -> CloudWatch {
        CloudWatch {}
    }

    /// Creates a builder for byte-oriented output targets (Files, TCP streams).
    pub fn io() -> IoWrite {
        IoWrite::default()
    }

    /// Creates a builder for string-oriented output targets ([`String`], in-memory buffers).
    pub fn fmt() -> FmtWrite {
        FmtWrite::default()
    }

    /// Creates a builder for capturing structured [`Message`][crate::Message] data in a [`Vec`].
    ///
    /// Ideal for unit testing and programmatic analysis.
    pub fn vector() -> Vector {
        Vector::default()
    }

    /// Creates a builder for logging to the Standard Output (stdout).
    pub fn cout() -> CoutWrite {
        CoutWrite::default()
    }

    /// Creates a builder for logging to the Standard Error (stderr).
    pub fn cerr() -> CerrWrite {
        CerrWrite::default()
    }
}