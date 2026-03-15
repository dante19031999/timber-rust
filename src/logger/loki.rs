// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "loki")]
#![cfg_attr(docsrs, doc(cfg(feature = "loki")))]

use crate::service::{DefaultLokiService, LokiConfig, LokiData, LokiMessage, LokiService, ServiceError};
use crate::{LoggerImpl, LoggerStatus, Message};
use reqwest::blocking::{Client, RequestBuilder, Response};
use std::any::Any;
use std::ops::AddAssign;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

/// A persistent, asynchronous logger implementation for Grafana Loki.
///
/// `LokiLogger` manages a dedicated background worker thread that handles batching,
/// retries, and HTTP transmission. It is designed to be high-performance, ensuring
/// that the main application thread is never blocked by network latency.
///
/// ### Important Design Notes
/// - **Monotonic Dating**: This logger uses an internal "highwater" marking system.
///   The original date of the `Message` is ignored in favor of a strictly increasing
///   timestamp generated at the moment of dispatch. This prevents Loki from rejecting
///   out-of-order logs during high-frequency bursts.
/// - **Worker Lifecycle**: A background thread is spawned on creation. Logs are
///   transmitted via an MPSC channel. Dropping the logger will signal the worker
///   to finish pending tasks before shutting down.
/// - **Performance**: Internal operations (batching, grouping by level) are performed
///   in-memory, assuming that network I/O is the primary bottleneck.
pub struct Loki {
    highwater: Mutex<SystemTime>,
    worker: Option<JoinHandle<()>>,
    sender: Option<Sender<LokiMessage>>,
    data: Arc<LokiData>,
    service: Arc<dyn LokiService + Send + Sync>,
}

impl Loki {
    /// Creates a new [`LokiLogger`][`Loki`] using the [`DefaultLokiService`].
    ///
    /// This is a convenience wrapper around [`Self::with_service`].
    pub fn new(config: LokiConfig) -> Box<Loki> {
        Self::with_service(config, Box::new(DefaultLokiService {}))
    }

    /// Primary constructor that initializes the logger with a custom [`LokiService`].
    ///
    /// This method:
    /// 1. Computes the full Loki Push API endpoint.
    /// 2. Wraps the provided service in an [`Arc`] for shared access between the logger and the worker.
    /// 3. Spawns a background worker thread to process the log queue.
    ///
    /// ### Dependency Injection
    /// By providing a custom [`LokiService`], you can override how batches are processed,
    /// enabling features like compression, custom filtering, or specialized error handling.
    ///
    /// # Threading
    /// Spawns a dedicated OS thread. The worker handles all network I/O and will
    /// gracefully shut down when the [`LokiLogger`][`Loki`] instance is dropped.
    ///
    /// # Panics
    /// Panics if the internal [`Client`][reqwest::blocking::Client] cannot be initialized.
    pub fn with_service(
        config: LokiConfig,
        service: Box<dyn LokiService + Send + Sync>,
    ) -> Box<Loki> {
        let mut post_url = config.url.clone();
        post_url.push_str("loki/api/v1/push");

        // Loki data
        let data = Arc::new(LokiData {
            client: Self::build_client(&config),
            config,
            post_url,
        });

        let work_data = data.clone();
        let service: Arc<dyn LokiService + Send + Sync> = Arc::from(service);
        let work_service = service.clone();
        let (sender, receiver) = std::sync::mpsc::channel::<LokiMessage>();

        Box::new(Loki {
            highwater: Mutex::new(SystemTime::now()),
            worker: Some(thread::spawn(move || {
                work_service.work(receiver, work_data)
            })),
            sender: Some(sender),
            data,
            service,
        })
    }

    /// Constructs a pre-configured HTTP client for the Loki service.
    ///
    /// This client is built with the specific connection and request timeouts
    /// defined in the [`LokiConfig`]. It is intended to be used within an [`Arc`]
    /// to share a connection pool across the worker's lifecycle.
    ///
    /// # Panics
    /// - Panics if the TLS backend cannot be initialized or if the system
    /// configuration prevents creating a secure socket.
    pub fn build_client(config: &LokiConfig) -> Client {
        Client::builder()
            .connect_timeout(config.connection_timeout)
            .timeout(config.request_timeout)
            .build()
            .expect("Failed to build reqwest client")
    }

    /// Performs a synchronous POST request to the Loki push API.
    ///
    /// This method handles the low-level HTTP transmission, including setting
    /// the `Content-Type` header and attaching authentication credentials.
    ///
    /// # Errors
    /// Returns a [`ServiceError::Network`] if the server is unreachable,
    /// DNS resolution fails, or the connection times out.
    pub fn request_post(payload: String, data: &Arc<LokiData>) -> Result<Response, ServiceError> {
        let mut request = data
            .client
            .post(data.post_url.as_str())
            .header("Content-Type", "application/json")
            .body(payload); // payload is already a String, no need for .to_string()

        request = Loki::request_auth(request, &data);

        request.send().map_err(ServiceError::Network)
    }

    /// Decorates a [`RequestBuilder`] with the configured authentication method.
    ///
    /// Supports both **Basic Auth** (username/password) and **Bearer Token** /// authentication. If both are configured, they will both be applied to
    /// the request (though Loki usually expects only one).
    ///
    /// # Parameters
    /// - `request`: The initial request builder to decorate.
    /// - `data`: The shared data containing authentication credentials.
    pub fn request_auth(mut request: RequestBuilder, data: &Arc<LokiData>) -> RequestBuilder {
        if let Some(auth) = &data.config.basic_auth {
            request = request.basic_auth(&auth.username, auth.password.as_deref());
        }
        if let Some(token) = &data.config.bearer_auth {
            request = request.bearer_auth(token);
        }
        request
    }
}

impl LoggerImpl for Loki {
    /// Returns the current operational status of the Loki service.
    ///
    /// This method performs a live health check by hitting the `/loki/status` endpoint.
    /// It uses a functional pipeline to transform the network result into a [`LoggerStatus`].
    ///
    /// ### Performance Note:
    /// This call is **blocking**. If the network is slow or the Loki server is hanging,
    /// this method will block the calling thread until the default timeout is reached.
    ///
    /// # Returns
    /// * [`LoggerStatus::Running`] - Server is reachable and returned a success code.
    /// * [`LoggerStatus::Broken`] - Any failure occurred (DNS, Connection Refused, 404, 500, etc.).
    fn status(&self) -> LoggerStatus {
        // Fetch url
        let mut url = self.data.config.url.clone();
        url.push_str("ready");

        // Build request
        let mut request = self.data.client.get(&url);
        request = Self::request_auth(request, &self.data);

        // Perform request
        request
            .send()
            .ok()
            .map(|response| match response.status().is_success() {
                true => LoggerStatus::Running,
                false => LoggerStatus::Broken,
            })
            .unwrap_or(LoggerStatus::Broken)
    }

    /// Enqueues a [`Message`] for asynchronous processing and delivery.
    ///
    /// This is the primary entry point for recording logs. It performs two critical tasks:
    /// 1. **Monotonic Timestamping**: Uses a "highwater" Mutex to ensure every message
    ///    has a strictly increasing timestamp, preventing Loki out-of-order errors.
    /// 2. **Asynchronous Dispatch**: Sends the message through an MPSC channel to the
    ///    background worker.
    ///
    /// ### Thread Safety & Performance
    /// This method is non-blocking (except for a very brief lock on the highwater
    /// timestamp). If the background worker is overloaded or the channel is
    /// disconnected, it triggers the [`LokiFallback`][`crate::service::LokiFallback`] immediately to avoid data loss.
    ///
    /// # Parameters
    /// - `message`: The log entry containing level, target, and content.
    fn log(&self, message: Message) {
        // Process highwater
        let mut timestamp = SystemTime::now();
        if let Ok(mut highwater) = self.highwater.lock() {
            if timestamp <= *highwater {
                highwater.add_assign(Duration::new(0, 1));
                timestamp = *highwater
            }
        }

        // Add to queue
        if let Some(sender) = &self.sender {
            if let Err(error) = sender.send(LokiMessage { timestamp, message }) {
                let message = error.0;
                self.service
                    .fallback(&ServiceError::LockPoisoned, &message.message);
            }
        } else {
            self.service.fallback(&ServiceError::LockPoisoned, &message);
        }
    }

    /// Returns a reference to the underlying type as [Any] for downcasting.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Ensures a graceful shutdown of the logging pipeline.
///
/// When the `LokiLogger` goes out of scope, the following sequence occurs:
/// 1. **Channel Closure**: The `sender` is dropped (`None`). This signals the background
///    worker that no more messages will be sent.
/// 2. **Worker Drain**: The worker's `receiver.recv()` will return an error once
///    the channel is empty, allowing its loop to terminate naturally.
/// 3. **Thread Join**: The main thread blocks until the worker thread has finished
///    processing and sending the final batch of logs.
///
/// This mechanism prevents data loss during application shutdown or restarts.
impl Drop for Loki {
    fn drop(&mut self) {
        // Drop the sender first to close the MPSC channel
        self.sender = None;

        // Wait for the worker thread to finish its last batch
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
