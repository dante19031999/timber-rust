// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "loki")]
#![cfg_attr(docsrs, doc(cfg(feature = "loki")))]

use crate::service::{HttpError, LokiData, LokiMessage, ServiceError};
use crate::{Fallback, LokiLogger, Message};
use chrono::{SecondsFormat, Utc};
use reqwest::blocking::Response;
use serde_json::json;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::UNIX_EPOCH;

/// The core logic provider for Loki interactions.
///
/// This trait defines how batches of logs are collected, formatted, and pushed.
/// By implementing this trait, you can customize the batching strategy or
/// adding custom logic (like GZIP compression) before sending.
///
/// ### Required Hierarchy
/// Must implement [`LokiFallback`][`Fallback`] to handle delivery failures.
pub trait Loki: Fallback {
    /// Orchestrates the background worker loop.
    ///
    /// This method is the entry point for the worker thread. It implements a
    /// **double-buffer/drain** strategy:
    /// 1. It blocks on `receiver.recv()` until at least one message is available,
    ///    ensuring zero CPU usage during idle periods.
    /// 2. Once a message arrives, it performs a non-blocking `try_recv` loop to
    ///    "drain" the channel and form a batch.
    /// 3. It groups messages by their log level into a `BTreeMap` to maintain
    ///    consistent stream ordering before calling [`work_batch`][`Self::work_batch()`].
    ///
    /// This design assumes that network I/O is the bottleneck, allowing the
    /// in-memory collection of logs to be extremely fast.
    fn work(&self, receiver: Receiver<LokiMessage>, data: Arc<LokiData>) {
        let mut messages = std::collections::BTreeMap::<String, Vec<LokiMessage>>::new();

        // Get first message
        while let Ok(message) = receiver.recv() {
            // Deal with message
            let level = message.message.level().to_string();
            messages.entry(level).or_insert_with(Vec::new).push(message);

            // Get batch
            while let Ok(message) = receiver.try_recv() {
                let level = message.message.level().to_string();
                messages.entry(level).or_insert_with(Vec::new).push(message);
            }

            // Process batch
            for (level, batch) in &mut messages {
                if !batch.is_empty() {
                    let _ = self.work_batch(level, batch, &data);
                }
            } // Process batch
        } // Receive message
    }

    /// Handles the transformation and transmission of a specific log batch.
    ///
    /// This method performs the following steps:
    /// 1. **Serialization**: Converts the batch of [`LokiMessage`] into a
    ///    JSON payload compatible with the Loki Push API.
    /// 2. **Transmission**: Sends the payload via a POST request.
    /// 3. **Retry Logic**: If the request fails or returns a non-success status,
    ///    it enters a retry loop up to the `max_retries` limit defined in [`LokiConfig`][`crate::service::LokiConfig`].
    /// 4. **Safety Net**: If all retries fail, it triggers the [`LokiFallback`][`Fallback`]
    ///    mechanism for every message in the batch.
    /// 5. **Cleanup**: Clears the batch vector to prepare it for the next iteration,
    ///    reusing the allocated memory capacity.
    ///
    /// # Returns
    ///
    /// Returns the last [`Response`] received from the server, or a [`ServiceError`]
    /// if the transmission was impossible.
    fn work_batch(
        &self,
        level: &str,
        batch: &mut Vec<LokiMessage>,
        data: &Arc<LokiData>,
    ) -> Result<Response, ServiceError> {
        // Prepare messages for loki
        let payload_batch = batch
            .iter()
            .map(|message| {
                let timestamp = message
                    .timestamp
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_nanos();
                json!([timestamp.to_string(), message.message.content().to_string()])
            })
            .collect::<Vec<_>>();

        // Payload
        let payload = json!({
            "streams": [{
                "stream": {
                    "app": data.config.get_app(),
                    "job": data.config.get_job(),
                    "env": data.config.get_env(),
                    "level": level,
                },
                "values": payload_batch
            }]
        });

        // First attempt
        let txt_payload = payload.to_string();
        let mut response = LokiLogger::request_post(txt_payload, &data);

        // Check if retry
        let needs_retry = match &response {
            Ok(response) => !response.status().is_success(),
            Err(_) => true,
        };

        // Attempt until end of lives
        if needs_retry {
            let mut i = 0usize;
            let payload = payload.to_string();

            response = loop {
                let current_response = LokiLogger::request_post(payload.clone(), &data);

                // Check if retry
                let is_success = match &current_response {
                    Ok(current_response) => current_response.status().is_success(),
                    Err(_) => false,
                };

                // End if retry
                if is_success {
                    break current_response;
                }

                if i >= data.config.get_max_retries() {
                    match current_response {
                        Ok(current_response) => {
                            break Err(ServiceError::Http(HttpError::new(
                                current_response.status().as_u16(),
                            )));
                        }
                        Err(_) => {
                            break current_response;
                        }
                    }
                }

                i += 1;
            } // Loop
        } // If not first OK

        // Last fallback
        if let Err(error) = &response {
            for message in &*batch {
                self.fallback(error, &message.message);
            }
        }

        // Clear message trail
        batch.clear();

        response
    }
}

/// The standard implementation of the Loki transport logic.
///
/// [`StandardLokiService`][`StandardLoki`] provides the baseline behavior for the logging pipeline,
/// using the standard batching and retry mechanisms defined in the [`LokiService`][`Loki`]
/// default trait methods.
///
/// ### Behavior
/// - **Transport**: Uses the `reqwest` blocking client to push JSON payloads.
/// - **Fallback**: Inherits the default [`LokiFallback`][`Fallback`] behavior, which redirects
///   failed logs to the system's standard error and output streams.
///
/// This struct is stateless, acting primarily as a marker to satisfy the trait
/// requirements of the [`LokiLogger`].
pub struct StandardLoki {}

impl Loki for StandardLoki {}

impl Fallback for StandardLoki {
    /// Handles the ultimate delivery failure for a log message.
    ///
    /// This method is the "safety net" of the logging pipeline. It is invoked when
    /// a message cannot be sent to Loki after exhausting all retry attempts or
    /// encountering a non-recoverable error (like a 400 Bad Request).
    ///
    /// ### Default Behavior
    /// The default implementation performs a **Best-Effort** recovery:
    /// 1. Formats the failure reason (HTTP status or Network error) to `stderr`.
    /// 2. Prints the original log content to `stdout` with its timestamp,
    ///    level, and payload.
    ///
    /// This ensures that even in a total network collapse, the logs are
    /// captured by the system's standard output streams (useful for Docker/K8s
    /// logs collectors).
    ///
    /// ### Thread Safety
    /// This is called from the background worker thread. Any custom implementation
    /// must be non-blocking and thread-safe to avoid stalling the entire
    /// logging pipeline.
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        let now: chrono::DateTime<Utc> = msg.instant().into();
        let now = now.to_rfc3339_opts(SecondsFormat::Nanos, true);

        match error {
            ServiceError::Http(e) => eprintln!("Loki rejected log: Status {}", e.status_code()),
            ServiceError::Network(e) => eprintln!("Loki network error: {}", e),
            _ => eprintln!("Loki service failure: {}", error),
        }

        println!("{} [{}] | {}", now, msg.level(), msg.content());
    }
}
