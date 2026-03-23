// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "loki")]
#![cfg_attr(docsrs, doc(cfg(feature = "loki")))]

use crate::{Logger, LokiLogger, service};

/// A factory for creating Grafana Loki loggers using a fluent builder pattern.
///
/// This factory provides two paths for construction:
/// 1. **Standard**: Uses the default network-based Loki service.
/// 2. **Injected**: Allows providing a custom implementation of the [`Loki`][service::Loki] trait.
pub struct Loki {}

/// A middle-state builder that holds the Loki configuration.
///
/// From here, you can either build a standard logger or transition to
/// a service-injected state.
pub struct LokiConfig {
    config: service::LokiConfig,
}

/// A final-state builder that holds both configuration and a specific service implementation.
///
/// This state is typically used in unit tests to inject a mock Loki service
/// or in specialized environments where a custom HTTP client is required.
pub struct LokiFactoryService {
    config: service::LokiConfig,
    service: Box<dyn service::Loki + Send + Sync>,
}

impl Loki {
    /// Begins the building process by providing a [`LokiConfig`].
    pub fn config(self, config: service::LokiConfig) -> LokiConfig {
        LokiConfig { config }
    }

    /// Shortcut to create a factory state with a pre-defined configuration and service.
    pub fn service(
        self,
        config: service::LokiConfig,
        service: Box<dyn service::Loki + Send + Sync>,
    ) -> LokiFactoryService {
        LokiFactoryService { config, service }
    }
}

impl LokiConfig {
    /// Creates a new LokiFactoryConfig
    pub fn new(config: service::LokiConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the internal configuration.
    pub fn get_config(&self) -> &service::LokiConfig {
        &self.config
    }

    /// Sets underlying configuration.
    pub fn config(self, config: service::LokiConfig) -> Self {
        Self { config, ..self }
    }

    /// Finalizes the builder and returns a high-level [`Logger`].
    ///
    /// This will initialize the default background worker and HTTP client.
    pub fn build(self) -> Logger {
        Logger::new(self.build_impl())
    }

    /// Builds the underlying [`LokiLogger`] implementation without the generic wrapper.
    pub fn build_impl(self) -> Box<LokiLogger> {
        LokiLogger::new(self.config)
    }

    /// Injects a custom service into the current building process.
    ///
    /// Use this to override the default networking logic.
    pub fn service(self, service: Box<dyn service::Loki + Send + Sync>) -> LokiFactoryService {
        LokiFactoryService {
            config: self.config,
            service,
        }
    }
}

impl LokiFactoryService {
    /// Creates a new LokiFactoryService
    pub fn new(config: service::LokiConfig, service: Box<dyn service::Loki + Send + Sync>) -> Self {
        Self { config, service }
    }

    /// Returns a reference to the internal configuration.
    pub fn get_config(&self) -> &service::LokiConfig {
        &self.config
    }

    /// Returns a reference to the internal service.
    pub fn get_service(&self) -> &(dyn service::Loki + Send + Sync) {
        self.service.as_ref()
    }

    /// Sets underlying configuration.
    pub fn config(self, config: service::LokiConfig) -> Self {
        Self { config, ..self }
    }

    /// Sets the underlying service.
    pub fn service(self, service: Box<dyn service::Loki + Send + Sync>) -> Self {
        Self { service, ..self }
    }

    /// Finalizes the builder using the provided custom service and returns a [`Logger`].
    pub fn build(self) -> Logger {
        Logger::new(self.build_impl())
    }

    /// Builds the underlying [`LokiLogger`] using the custom service.
    pub fn build_impl(self) -> Box<LokiLogger> {
        LokiLogger::with_service(self.config, self.service)
    }
}
