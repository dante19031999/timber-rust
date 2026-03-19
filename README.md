[![Repository](https://img.shields.io/badge/repository-GitHub-lightgrey?style=for-the-badge&logo=github)](https://github.com/dante19031999/timber-rust)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue?style=for-the-badge&logo=github)](https://dante19031999.github.io/timber-rust/timber_rust/index.html)
[![Credits](https://img.shields.io/badge/credits-licenses-orange?style=for-the-badge&logo=creative-commons)](https://dante19031999.github.io/timber-rust/about/licenses.html)

# 🌲 Timber Rust

## 🚀 Overview

**Timber Rust** is designed to provide a seamless integration for system logs and telemetry. It focuses on performance
and type-safety, allowing developers to switch between different observability backends with minimal configuration.

## ✨ Key Features

- **Blazing Fast:** Zero-cost abstractions following the Rust philosophy.
- **Pluggable Backends:** Support for multiple services (e.g., Loki) via feature flags.
- **Secure by Default:** Automatic vulnerability scanning and license compliance.
- **Docs-First:** Comprehensive API documentation integrated with [GitHub Pages](https://dante19031999.github.io/timber-rust/timber_rust/index.html).

## 📖 Documentation

Full technical documentation, including API references and module usage, is available at our GitHub Pages site.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
timber-rust = { git = "[https://github.com/dante19031999/timber-rust](https://github.com/dante19031999/timber-rust)" }
```

## 🚀 How to get started

### Logging into stdout/stderr

```rust
use timber_rust::LoggerFactory;
use timber_rust::LogLevel;

// An async logger over stdout/stderr (1 worker)
let logger_stdout = LoggerFactory::queued_cout();
let logger_stderr = LoggerFactory::queued_cerr();

// Log something
logger_stdout.log(("INFO", "Hello world!"));
logger_stderr.log((LogLevel::Info, "Hello world!"));
```

- See: [`DefaultCoutWriteService`][`service::StandardCoutWriteService`]
- See: [`DefaultCerrWriteService`][`service::StandardCerrWriteService`]

### Logging into a file

```rust
use timber_rust::LoggerFactory;
use timber_rust::LogLevel;
use std::fs::OpenOptions;

let mut file = OpenOptions::new()
    .write(true)   // Write
    .append(true)  // Append (not squash previous logs)
    .create(true)  // Create if not exits
    .open("logs.txt").expect("Could not open file!");

// An async logger over a file (1 worker)
let logger = LoggerFactory::queued_file(file);

// Log something
logger.log(("INFO", "Hello world!"));
logger.log((LogLevel::Info, "Hello world!"));

# let _ = std::fs::remove_file("logs.txt");
```

- See: [`DefaultFileWriteService`][`crate::service::StandardFileWriteService`]

### Logging into loki

```rust
# #[cfg(feature = "loki")]
use timber_rust::service::LokiConfig;
use timber_rust::LoggerFactory;
use timber_rust::LogLevel;

# #[cfg(feature = "loki")]
let mut config = LokiConfig::new("localhost::3001");

// An async batched loki logger
# #[cfg(feature = "loki")]
let logger = LoggerFactory::loki(file);

// Log something
# #[cfg(feature = "loki")]
logger.log(("INFO", "Hello world!"));
# #[cfg(feature = "loki")]
logger.log((LogLevel::Info, "Hello world!"));

```

- See: [`LokiLogger`]
- See: [`LokiConfig`][`service::LokiConfig`]

### Creating your own custom loggers

The library uses two main logger models:
- [`DirectLogger`]: A sync logger. Blocks until the process is finished.
- [`QueuedLogger`]: An async logger. Uses a crossbeam internal queue to dispatch logs.
- Both options use a [`Service`] as a backend. You may check or inbuilt [services][`service`]. You may build your own implementing the trait [`Service`][`crate::Service`].
- [`LokiLogger`]: An specific batched logger to loki. Can be customized using a custom [`LokiService`][`crate::service::LokiService`].
- Don't forget to check first our inbuilt [`LoggerFactory`]!

### Mutltichanel logging

Our logging system supports multichannel logging through [`LogManager`].

It is possible to create presets through [`Config`]. Though it is limited to what can be deduced at runtime.

Our [`Config`][`crate::Config`] implements [`serde::Serialize`] and [`serde::Deserialize`] for total freedom in congiguration storage.

The examples where built using JSON (the most common), but [`serde`] allows for any model.

## ⚖️ Credits & Licensing

This project relies on several open-source crates. You can find the full list of dependencies and their respective
licenses in our [Credits Report](https://dante19031999.github.io/timber-rust/about/licenses.html).
