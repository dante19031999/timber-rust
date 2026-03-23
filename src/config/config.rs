use crate::config::entry::Entry;
use crate::service::{FeatureDisabledError, FvnBuildHasher, ServiceError};
use crate::{LogManager, LoggerFactory};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::Path;

pub struct Config {
    entries: HashMap<String, Entry, FvnBuildHasher>,
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
                Entry::Silent {} => LoggerFactory::silent(),
                Entry::StdOut {
                    concurrency,
                    max_retries,
                    worker_count,
                } => {
                    let mut factory = LoggerFactory::cout();
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    factory.build(concurrency)
                }
                Entry::StdErr {
                    concurrency,
                    max_retries,
                    worker_count,
                } => {
                    let mut factory = LoggerFactory::cout();
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    factory.build(concurrency)
                }
                Entry::File {
                    path,
                    concurrency,
                    max_retries,
                    worker_count,
                } => {
                    // Create the directory structure if it doesn't exist
                    if let Some(parent) = Path::new(path.as_str()).parent() {
                        std::fs::create_dir_all(parent).map_err(|e| ServiceError::Io(e))?;
                    }
                    // Build the File with Append and Create flags
                    let file: File = OpenOptions::new()
                        .append(true) // High-level: ensures all writes go to the end
                        .create(true) // High-level: creates the file if missing
                        .write(true) // Required to enable writing
                        .open(path)
                        .map_err(|e| ServiceError::Io(e))?;
                    // Build the logger
                    let mut factory = LoggerFactory::io().file(file);
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    factory.build(concurrency)
                }
                Entry::BufferedFile {
                    path,
                    concurrency,
                    max_retries,
                    worker_count,
                } => {
                    // Create the directory structure if it doesn't exist
                    if let Some(parent) = Path::new(path.as_str()).parent() {
                        std::fs::create_dir_all(parent).map_err(|e| ServiceError::Io(e))?;
                    }
                    // Build the File with Append and Create flags
                    let file: File = OpenOptions::new()
                        .append(true) // High-level: ensures all writes go to the end
                        .create(true) // High-level: creates the file if missing
                        .write(true) // Required to enable writing
                        .open(path)
                        .map_err(|e| ServiceError::Io(e))?;
                    // Build the logger
                    let mut factory = LoggerFactory::io().buffered_file(BufWriter::new(file));
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    factory.build(concurrency)
                }
                Entry::String {
                    concurrency,
                    max_retries,
                    worker_count,
                    capacity,
                } => {
                    // Build the logger
                    let mut factory = LoggerFactory::fmt().string();
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    if let Some(capacity) = capacity {
                        factory = factory.max_retries(capacity);
                    }
                    factory.build(concurrency)
                }
                Entry::Vector {
                    concurrency,
                    max_retries,
                    worker_count,
                    capacity,
                } => {
                    let mut factory = LoggerFactory::vector();
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    if let Some(capacity) = capacity {
                        factory = factory.max_retries(capacity);
                    }
                    factory.build(concurrency)
                }
                #[cfg(feature = "loki")]
                Entry::Loki { .. } => {
                    let config = entry.build_loki_config().expect("Corrupted memory");
                    LoggerFactory::loki().config(config).build()
                }
                #[cfg(feature = "aws")]
                Entry::CloudWatchConfig { .. } => {
                    let config = entry.build_cloudwatch_config().expect("Corrupted memory");
                    LoggerFactory::cloudwatch().config(config).build()
                }
                #[cfg(feature = "aws")]
                Entry::CloudWatchEnv { log_group } => {
                    LoggerFactory::cloudwatch().env(log_group).build()
                }
                #[cfg(feature = "awscout")]
                Entry::CloudWatchCout {
                    concurrency,
                    max_retries,
                    worker_count,
                } => {
                    let mut factory = LoggerFactory::cout();
                    if let Some(max_retries) = max_retries {
                        factory = factory.max_retries(max_retries);
                    }
                    if let Some(worker_count) = worker_count {
                        factory = factory.max_retries(worker_count);
                    }
                    factory.build(concurrency)
                }
                Entry::DisabledFeature { feature } => {
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

    /// Returns a reference to the [configuration entry][`Entry`] for the specified channel.
    ///
    /// Accepts any type that can be referenced as a string (e.g., `&str` or `String`).
    pub fn get_entry<Q>(&self, channel: &Q) -> Option<&Entry>
    where
        Q: AsRef<str> + ?Sized,
    {
        self.entries.get(channel.as_ref())
    }

    /// Returns a mutable reference to the [configuration entry][`Entry`] for the specified channel.
    pub fn get_entry_mut<Q>(&mut self, channel: &Q) -> Option<&mut Entry>
    where
        Q: AsRef<str> + ?Sized,
    {
        self.entries.get_mut(channel.as_ref())
    }

    /// Removes a channel from the configuration and returns its [entry][`Entry`], if it existed.
    pub fn remove_entry<Q>(&mut self, channel: &Q) -> Option<Entry>
    where
        Q: AsRef<str> + ?Sized,
    {
        self.entries.remove(channel.as_ref())
    }

    /// Inserts a raw [configuration entry][`Entry`] for a specific channel.
    /// If the channel already exists, the old configuration is overwritten.
    pub fn insert_entry<S>(&mut self, channel: S, entry: Entry)
    where
        S: Into<String>,
    {
        self.entries.insert(channel.into(), entry);
    }
}
