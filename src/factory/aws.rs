// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "awscout")))]

use crate::factory::awscout::CloudWatch;
use crate::{CloudWatchLogger, Logger, service};

/// A builder state for constructing a [`CloudWatchLogger`] from a concrete [`CloudWatchConfig`].
pub struct CloudWatchConfig {
    config: service::CloudWatchConfig,
}

/// A builder state for constructing a [`CloudWatchLogger`] using environment-based
/// AWS credentials and a specific log group.
pub struct CloudWatchEnv {
    log_group: String,
}

impl CloudWatch {
    /// Begins building a CloudWatch logger using a manual configuration.
    pub fn config(self, config: service::CloudWatchConfig) -> CloudWatchConfig {
        CloudWatchConfig { config }
    }

    /// Begins building a CloudWatch logger that pulls credentials from the environment.
    pub fn env<S>(self, log_group: S) -> CloudWatchEnv
    where
        S: Into<String>,
    {
        CloudWatchEnv {
            log_group: log_group.into(),
        }
    }
}

impl CloudWatchConfig {
    /// Creates a new config factory instance.
    pub fn new(config: service::CloudWatchConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the internal configuration.
    pub fn get_config(&self) -> &service::CloudWatchConfig {
        &self.config
    }

    /// Sets underlying configuration.
    pub fn config(self, config: service::CloudWatchConfig) -> CloudWatchConfig {
        Self { config, ..self }
    }

    /// Finalizes construction and returns a wrapped [`Logger`].
    pub fn build(self) -> Logger {
        Logger::new(self.build_impl())
    }

    /// Builds the underlying [`CloudWatchLogger`] implementation.
    pub fn build_impl(self) -> Box<CloudWatchLogger> {
        CloudWatchLogger::new(self.config)
    }
}

impl CloudWatchEnv {
    /// Creates a new environment-based factory instance.
    pub fn new<S>(log_group: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            log_group: log_group.into(),
        }
    }

    /// Returns the name of the target log group.
    pub fn get_log_group(&self) -> &str {
        self.log_group.as_str()
    }

    /// Sets or overrides the target log group.
    pub fn log_group<S>(&mut self, value: S) -> Self
    where
        S: Into<String>,
    {
        CloudWatchEnv {
            log_group: value.into(),
        }
    }

    /// Finalizes construction and returns a wrapped [`Logger`].
    pub fn build(self) -> Logger {
        Logger::new(self.build_impl())
    }

    /// Builds the underlying [`CloudWatchLogger`] using environment discovery.
    pub fn build_impl(self) -> Box<CloudWatchLogger> {
        CloudWatchLogger::from_env(self.log_group)
    }
}
