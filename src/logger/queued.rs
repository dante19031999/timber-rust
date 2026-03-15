// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use crate::{LoggerImpl, LoggerStatus, Message, Service};
use crossbeam::channel::Sender;
use std::any::Any;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

/// Wrapper for messages in flight.
///
/// This struct implements the "Traveling Sender" pattern, where the message
/// carries its own means of re-queuing itself. This decouples the channel's
/// lifetime from the workers and ties it to the completion of the work.
struct PerishableMessage {
    /// The actual log payload.
    pub message: Message,
    /// Number of retry attempts remaining.
    pub lives: usize,
    /// A clone of the producer-side channel handle used for retries.
    pub sender: Sender<PerishableMessage>,
}

/// An asynchronous, thread-pooled logger with guaranteed completion.
///
/// `AsyncLogger` acts as an orchestration layer that offloads logging side-effects
/// to a background pool of workers. It is designed for high-throughput scenarios
/// where log generation should not block the main application execution.
///
/// ### Architecture: The "Traveling Sender" Pattern
/// This logger implements a resilient dispatch system using [`crossbeam`] channels.
/// Each [`Message`] is wrapped in a `PerishableMessage` which carries its own
/// [`Sender`] clone. This ensures that:
/// 1. **Decoupled Retries**: Workers can re-queue failed tasks without needing
///    access to the parent logger's state.
/// 2. **Lifespan Tracking**: Messages have a finite number of `lives`, preventing
///    infinite loops in case of persistent service failure.
///
/// ### Graceful Shutdown & Persistence
/// The `AsyncLogger` guarantees that no log is lost during program termination:
/// - **Drain Policy**: On `drop()`, the primary sender is destroyed. Workers will
///   continue processing until the internal buffer is empty and all re-queue
///   attempts (lives) are exhausted.
/// - **Thread Synchronization**: The logger waits (`join`) for all worker threads
///   to finish their tasks before allowing the process to exit.
///
/// ### Important Design Considerations
/// - **Ordering**: Because this logger uses multiple concurrent workers (MPMC),
///   **strict chronological ordering is not guaranteed**. While each `Message`
///   retains its original timestamp, the order in which they reach the final
///   destination may vary due to thread scheduling and retry logic.
/// - **Compatibility**: This pattern is ideal for I/O-bound services like local
///   files or standard streams. It is **not recommended** for services requiring
///   strict sequential consistency (e.g., Loki or Aws).
///
/// ### Example
/// ```rust
/// # use std::fs::File;
/// # use timber_rust::{LogLevel, Logger};
/// # use timber_rust::Message;
/// # use timber_rust::MessageFactory;
/// # use timber_rust::QueuedLogger;
/// # use timber_rust::service::{DefaultCoutWriteService, DefaultMessageFormatter};
/// let service = DefaultCoutWriteService::new(DefaultMessageFormatter{});
/// let logger = QueuedLogger::new(service, 3, 4); // 3 retries, 4 worker threads
/// let logger = Logger::new(logger);
/// logger.log((LogLevel::Info,"System started"));
/// ```
pub struct Queued {
    /// The underlying service used to perform the actual logging/work.
    service: Arc<dyn Service + Send + Sync>,
    /// Default number of retries for new messages.
    max_retries: usize,
    /// Handles to the background worker threads.
    workers: Vec<JoinHandle<()>>,
    /// The primary entry point for the queue. Wrapped in an `Option` to allow
    /// the `Drop` implementation to signal shutdown by destroying the sender.
    sender: Option<Sender<PerishableMessage>>,
}

impl Queued {
    /// Creates a new [`QueuedLogger`][`Queued`] and initializes the worker pool.
    ///
    /// # Arguments
    /// * `service` - The logging service implementation.
    /// * `max_retries` - How many times a failed log should be re-queued.
    /// * `worker_count` - The number of background threads to spawn.
    pub fn new(
        service: Box<dyn Service + Send + Sync>,
        max_retries: usize,
        worker_count: usize,
    ) -> Box<Self> {
        let service: Arc<dyn Service + Send + Sync> = Arc::from(service);
        let (sender, receiver) = crossbeam::channel::unbounded::<PerishableMessage>();
        let mut workers = Vec::with_capacity(worker_count);

        for _ in 0..worker_count {
            let worker_receiver = receiver.clone();
            let worker_service = service.clone();

            workers.push(thread::spawn(move || {
                while let Ok( mut message) = worker_receiver.recv() {

                    // Attempt to log
                    let result = worker_service.work(&message.message);
                    if result.is_ok() {
                        continue;
                    }

                    // Treat error
                    let error = result.unwrap_err();
                    // Discard if lives reaches 0
                    if message.lives == 0 {
                        worker_service.fallback(&error, &message.message);
                        continue;
                    }

                    // Retry
                    message.lives -= 1;
                    let sender = message.sender.clone();
                    let result = sender.send(message);
                    if let Err(err) = result {
                        worker_service.fallback(&error, &err.0.message);
                    }
                }
            }));
        }

        Box::new(Queued {
            service,
            max_retries,
            workers,
            sender: Some(sender),
        })
    }

    /// Returns the base service
    pub fn get_service(&self) -> &dyn Service {
        self.service.as_ref()
    }
}

impl LoggerImpl for Queued {
    /// Checks the health status of the underlying service.
    fn status(&self) -> LoggerStatus {
        self.service.status()
    }

    /// Dispatches a message to the background worker pool.
    ///
    /// Each message is bundled with a clone of the sender to enable
    /// self-contained retry logic within the worker threads.
    fn log(&self, message: Message) {
        // Safe to expect: Borrow checker prevents log() during drop().
        let sender = self
            .sender
            .as_ref()
            .expect("AsyncLogger integrity violation: log() called after drop() initialization.");

        let _ = sender.send(PerishableMessage {
            message,
            lives: self.max_retries,
            sender: sender.clone(),
        });
    }

    /// Downcasts the logger to `Any` for reflection-like capabilities.
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for Queued {
    /// Orchestrates a graceful shutdown of the logger.
    ///
    /// 1. The primary [`Sender`] is dropped (set to `None`).
    /// 2. This does not immediately close the channel if messages are in the queue,
    ///    as those messages carry their own [`Sender`] clones.
    /// 3. Once all messages are processed/exhausted, the last [`Sender`] dies.
    /// 4. `worker_receiver.recv()` returns `Err`, and threads exit.
    /// 5. [`join()`][std::thread::JoinHandle::join()] ensures all threads have finished before the program proceeds.
    fn drop(&mut self) {
        // Pull the plug on the primary entry point.
        self.sender = None;

        // Wait for workers to drain the existing queue and self-terminate.
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
