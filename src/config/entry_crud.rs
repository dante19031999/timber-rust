// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::Concurrency;
use crate::config::entry::Entry;
#[cfg(feature = "aws")]
use crate::service::CloudWatchConfig;
#[cfg(feature = "loki")]
use crate::service::LokiConfig;
#[cfg(feature = "aws")]
use std::time::SystemTime;

impl Entry {
    /// Returns the concurrency strategy if the variant supports one.
    ///
    /// Returns `None` for variants like `Silent` where execution strategy is irrelevant.
    pub fn get_concurrency(&self) -> Option<Concurrency> {
        match self {
            Entry::Silent { .. } => None,
            Entry::StdOut { concurrency, .. } => Some(*concurrency),
            Entry::StdErr { concurrency, .. } => Some(*concurrency),
            Entry::File { concurrency, .. } => Some(*concurrency),
            Entry::BufferedFile { concurrency, .. } => Some(*concurrency),
            Entry::String { concurrency, .. } => Some(*concurrency),
            Entry::Vector { concurrency, .. } => Some(*concurrency),
            _ => None,
        }
    }

    /// Returns the configured capacity if the variant supports pre-allocation.
    pub fn get_capacity(&self) -> Option<usize> {
        match self {
            Entry::String { capacity, .. } => capacity.clone(),
            Entry::Vector { capacity, .. } => capacity.clone(),
            _ => None,
        }
    }

    /// Returns the number of worker threads configured for this entry.
    pub fn get_worker_count(&self) -> Option<usize> {
        match self {
            Entry::StdOut { worker_count, .. } => worker_count.clone(),
            Entry::StdErr { worker_count, .. } => worker_count.clone(),
            Entry::File { worker_count, .. } => worker_count.clone(),
            Entry::BufferedFile { worker_count, .. } => worker_count.clone(),
            Entry::String { worker_count, .. } => worker_count.clone(),
            Entry::Vector { worker_count, .. } => worker_count.clone(),
            #[cfg(feature = "loki")]
            Entry::Loki { worker_count, .. } => Some(*worker_count),
            #[cfg(feature = "awscout")]
            Entry::CloudWatchCout { worker_count, .. } => worker_count.clone(),
            _ => None,
        }
    }

    /// Returns the maximum retry attempts configured for this entry.
    pub fn get_max_retries(&self) -> Option<usize> {
        match self {
            Entry::StdOut { max_retries, .. } => max_retries.clone(),
            Entry::StdErr { max_retries, .. } => max_retries.clone(),
            Entry::File { max_retries, .. } => max_retries.clone(),
            Entry::BufferedFile { max_retries, .. } => max_retries.clone(),
            Entry::String { max_retries, .. } => max_retries.clone(),
            Entry::Vector { max_retries, .. } => max_retries.clone(),
            #[cfg(feature = "loki")]
            Entry::Loki { max_retries, .. } => Some(*max_retries),
            #[cfg(feature = "awscout")]
            Entry::CloudWatchCout { max_retries, .. } => max_retries.clone(),
            _ => None,
        }
    }

    /// Sets the concurrency strategy for this entry.
    pub fn concurrency(mut self, concurrency: Concurrency) -> Self {
        match &mut self {
            Entry::StdOut {
                concurrency: conc, ..
            }
            | Entry::StdErr {
                concurrency: conc, ..
            }
            | Entry::File {
                concurrency: conc, ..
            }
            | Entry::BufferedFile {
                concurrency: conc, ..
            }
            | Entry::String {
                concurrency: conc, ..
            }
            | Entry::Vector {
                concurrency: conc, ..
            } => {
                *conc = concurrency;
            }
            #[cfg(feature = "awscout")]
            Entry::CloudWatchCout {
                concurrency: conc, ..
            } => {
                *conc = concurrency;
            }
            _ => {}
        }

        self
    }

    /// Sets the pre-allocation capacity for variants that support it (String, Vector).
    pub fn capacity(mut self, capacity: usize) -> Self {
        match &mut self {
            Entry::String { capacity: cap, .. } => *cap = Some(capacity),
            Entry::Vector { capacity: cap, .. } => *cap = Some(capacity),
            _ => {}
        }

        self
    }

    /// Sets the background worker count for this entry.
    pub fn worker_count(mut self, worker_count: usize) -> Self {
        match &mut self {
            Entry::StdOut {
                worker_count: wk, ..
            }
            | Entry::StdErr {
                worker_count: wk, ..
            }
            | Entry::File {
                worker_count: wk, ..
            }
            | Entry::BufferedFile {
                worker_count: wk, ..
            }
            | Entry::String {
                worker_count: wk, ..
            }
            | Entry::Vector {
                worker_count: wk, ..
            } => *wk = Some(worker_count),
            #[cfg(feature = "loki")]
            Entry::Loki {
                worker_count: wk, ..
            } => *wk = worker_count,
            #[cfg(feature = "awscout")]
            Entry::CloudWatchCout {
                worker_count: wk, ..
            } => *wk = Some(worker_count),
            _ => {}
        }
        self
    }

    /// Sets the maximum number of retry attempts for this entry.
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        match &mut self {
            Entry::StdOut {
                max_retries: ret, ..
            }
            | Entry::StdErr {
                max_retries: ret, ..
            }
            | Entry::File {
                max_retries: ret, ..
            }
            | Entry::BufferedFile {
                max_retries: ret, ..
            }
            | Entry::String {
                max_retries: ret, ..
            }
            | Entry::Vector {
                max_retries: ret, ..
            } => *ret = Some(max_retries),
            #[cfg(feature = "loki")]
            Entry::Loki {
                max_retries: wk, ..
            } => *wk = max_retries,
            #[cfg(feature = "awscout")]
            Entry::CloudWatchCout {
                max_retries: wk, ..
            } => *wk = Some(max_retries),
            _ => {}
        }
        self
    }

    #[cfg(feature = "loki")]
    #[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
    /// Attempts to extract and build a [`LokiConfig`] from this entry.
    pub fn build_loki_config(self) -> Option<LokiConfig> {
        match self {
            Entry::Loki {
                url,
                app,
                job,
                env,
                basic_auth,
                bearer_auth,
                connection_timeout,
                request_timeout,
                max_retries,
                worker_count,
            } => {
                let config = LokiConfig::new(url)
                    .app(app)
                    .job(job)
                    .env(env)
                    .basic_auth(basic_auth)
                    .bearer_auth(bearer_auth)
                    .connection_timeout(connection_timeout)
                    .request_timeout(request_timeout)
                    .max_retries(max_retries)
                    .worker_count(worker_count);
                Some(config)
            }
            _ => None,
        }
    }

    #[cfg(feature = "aws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
    /// Attempts to extract and build a [`CloudWatchConfig`] from this entry.
    pub fn build_cloudwatch_config(self) -> Option<CloudWatchConfig> {
        match self {
            Entry::CloudWatchConfig {
                access_key_id,
                access_key_secret,
                session_token,
                expires_in,
                log_group,
                region,
            } => {
                let config =
                    CloudWatchConfig::new(access_key_id, access_key_secret, log_group, region)
                        .session_token(session_token)
                        .expires_in(expires_in.map(|t| SystemTime::from(t)));
                Some(config)
            }
            _ => None,
        }
    }
}
