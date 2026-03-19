// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "aws")))]

use crate::service::aws::{MessageFormatter, SimpleCloudWatch};
use crate::service::{CloudWatchConfig, CloudWatchMessage, ServiceError};
use crate::{LoggerImpl, LoggerStatus, Message, service};
use std::any::Any;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

/// A high-level, thread-safe logger implementation for AWS CloudWatch.
///
/// This structure acts as a bridge between the application's logging calls and the
/// asynchronous AWS SDK. It implements a **producer-consumer pattern** using a
/// dedicated background worker thread to ensure that logging operations do not
/// block the main application execution.
///
/// # Architecture
/// 1. **Producer ([`log`][Self::log])**: Receives [`Message`] objects, timestamps them,
///    and pushes them into an internal MPSC (Multi-Producer, Single-Consumer) channel.
/// 2. **Consumer (`worker`)**: A dedicated background thread that
///    drains the channel using a **greedy-drain** strategy, batching logs to
///    optimize network throughput to AWS.
/// 3. **Service Layer**: Handles the actual communication with the CloudWatch API
///    via the internal Tokio runtime.
///
/// # Performance & Thread Safety
/// - **Non-blocking**: The `log` method is essentially non-blocking, as it only
///   performs a channel send operation.
/// - **Graceful Shutdown**: Implements [`Drop`] to ensure that the channel is closed
///   and all pending logs are flushed to AWS before the thread joins and the
///   application terminates.
///
/// # Example
/// ```rust
/// use timber_rust::Logger;
/// use timber_rust::service::CloudWatch;
/// use timber_rust::service::aws::Config;
///
/// let config = Config::new("access", "secret", "my-group", "us-east-1");
/// let logger = CloudWatch::new(config);
/// let logger = Logger::new(logger);
/// ```
pub struct CloudWatch {
    /// Handle to the background worker thread. Taken during [`Drop`].
    worker: Option<JoinHandle<()>>,
    /// Sending end of the log pipeline. Dropped during [`Drop`] to signal shutdown.
    sender: Option<Sender<CloudWatchMessage>>,
    /// Shared reference to the underlying CloudWatch service provider.
    service: Arc<dyn service::aws::CloudWatch + Send + Sync>,
}

impl CloudWatch {
    pub fn new(config: CloudWatchConfig) -> Box<CloudWatch> {
        Self::with_service(SimpleCloudWatch::new(config))
    }

    pub fn new_formatted<F>(config: CloudWatchConfig, formatter: F) -> Box<CloudWatch>
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        Self::with_service(SimpleCloudWatch::new_formatted(config, formatter))
    }

    pub fn from_env<S>(log_group: S) -> Box<CloudWatch>
    where
        S: Into<String>,
    {
        Self::with_service(SimpleCloudWatch::from_env(log_group.into()))
    }

    pub fn from_env_formatted<S, F>(log_group: S, formatter: F) -> Box<CloudWatch>
    where
        S: Into<String>,
        F: MessageFormatter + Send + Sync + 'static,
    {
        Self::with_service(SimpleCloudWatch::from_env_formatted(
            log_group.into(),
            formatter,
        ))
    }

    pub fn with_service(
        service: Box<dyn service::aws::CloudWatch + Send + Sync>,
    ) -> Box<CloudWatch> {
        let service: Arc<dyn service::aws::CloudWatch + Send + Sync> = Arc::from(service);
        let work_service = service.clone();
        let (sender, receiver) = std::sync::mpsc::channel::<CloudWatchMessage>();

        Box::new(Self {
            worker: Some(thread::spawn(move || work_service.work(receiver))),
            sender: Some(sender),
            service,
        })
    }
}

impl LoggerImpl for CloudWatch {
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
        self.service.status()
    }

    /// Enqueues a [`Message`] for asynchronous processing and delivery.
    ///
    /// This is the primary entry point for recording logs. It performs two critical tasks:
    /// 1. **Timestamping**: Uses a timestamp generated at the log moment.
    /// 2. **Asynchronous Dispatch**: Sends the message through an MPSC channel to the
    ///    background worker.
    ///
    /// ### Thread Safety & Performance
    /// This method is non-blocking (except for a very brief lock on the highwater
    /// timestamp). If the background worker is overloaded or the channel is
    /// disconnected, it triggers the [`Fallback`][`crate::service::Fallback`] immediately to avoid data loss.
    ///
    /// # Parameters
    /// - `message`: The log entry containing level, target, and content.
    fn log(&self, message: Message) {
        // Timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;

        // Add to queue
        if let Some(sender) = &self.sender {
            if let Err(error) = sender.send(CloudWatchMessage { message, timestamp }) {
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
/// When the [`CloudwatchLogger`][`CloudWatch`] goes out of scope, the following sequence occurs:
/// 1. **Channel Closure**: The `sender` is dropped (`None`). This signals the background
///    worker that no more messages will be sent.
/// 2. **Worker Drain**: The worker's `receiver.recv()` will return an error once
///    the channel is empty, allowing its loop to terminate naturally.
/// 3. **Thread Join**: The main thread blocks until the worker thread has finished
///    processing and sending the final batch of logs.
///
/// This mechanism prevents data loss during application shutdown or restarts.
impl Drop for CloudWatch {
    fn drop(&mut self) {
        // Drop the sender first to close the MPSC channel
        self.sender = None;

        // Wait for the worker thread to finish its last batch
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
