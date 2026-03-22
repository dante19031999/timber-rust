// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "aws")))]

use crate::logger::Status;
use crate::service::aws::{Config, Data, StandardMessageFormatter, MessageFormatter};
use crate::service::{CloudWatchMessage, ServiceError};
use crate::{Fallback, Message};
use aws_config::{BehaviorVersion, Region};
use aws_sdk_cloudwatchlogs::types::InputLogEvent;
use chrono::{SecondsFormat, Utc};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio::runtime::Builder;

/// Trait defining the behavior for CloudWatch log providers.
///
/// This abstraction allows swapping log service providers (e.g., for testing or
/// migrating to other cloud providers) without changing the core application logic.
///
/// It implements `Sync + Send` to ensure the service can be safely shared across
/// multiple threads in a concurrent environment.
pub trait CloudWatch: Fallback + Sync + Send {
    fn status(&self) -> Status;

    fn work(&self, receiver: Receiver<CloudWatchMessage>);
}

/// Standard implementation of the CloudWatch service using the AWS SDK.
///
/// # Design Considerations
/// AWS SDK for Rust is natively asynchronous. This implementation wraps the
/// asynchronous client and manages an internal **Tokio [`Runtime`][`tokio::runtime::Runtime`]** to provide
/// a synchronous (blocking) API.
///
/// ### Performance Note
/// Initializing this struct has a high overhead due to the creation of a
/// multi-threaded Tokio [`Runtime`][`tokio::runtime::Runtime`]. It is recommended to initialize this service
/// once and reuse the instance throughout the application's lifecycle.
///
/// ### Alternatives
/// If high-performance logging is required without the overhead of an internal runtime, consider:
/// - Logging to [`stdout`][`crate::service::CoutWrite`] and using the **CloudWatch Agent**.
/// - Logging to `stdout` with a [`CloudWatchCout`][`crate::service::CloudWatchCout`] service and using the **CloudWatch Agent**.
/// - Using **AWS Lambda** integrated logging.
pub struct SimpleCloudWatch {
    /// Service configuration
    data: Arc<Data>,
}

impl SimpleCloudWatch {
    /// Initializes a new `SimpleCloudWatch` service using the [`StandardMessageFormatter`].
    ///
    /// This is a convenience wrapper around [`Self::new_formatted`]. It creates a
    /// multi-threaded Tokio runtime to manage the asynchronous AWS SDK
    /// synchronously under the hood.
    ///
    /// # Arguments
    /// * `config` - A [`Config`] object containing AWS credentials and log group.
    ///
    /// # Panics
    /// Panics if the Tokio [`Runtime`][`tokio::runtime::Runtime`] fails to initialize or the AWS region is invalid.
    ///
    /// # Example
    /// ```rust
    /// # use timber_rust::service::aws::Config;
    /// # use timber_rust::service::SimpleCloudWatch;
    /// let config = Config::new("access_key", "secret", "my-group", "us-east-1");
    /// let service = SimpleCloudWatch::new(config);
    /// ```
    pub fn new(config: Config) -> Box<dyn CloudWatch + Send + Sync> {
        Self::new_formatted(config, StandardMessageFormatter {})
    }

    /// Initializes a new `SimpleCloudWatch` service with a custom [`MessageFormatter`].
    ///
    /// Use this method if you need to customize how logs are structured (e.g., adding
    /// extra fields, changing JSON keys, or using a non-JSON format).
    ///
    /// # Arguments
    /// * `config` - AWS configuration and credentials.
    /// * `formatter` - An implementation of [`MessageFormatter`].
    ///
    /// # Type Parameters
    /// * `F` - The specific type of the formatter, which must be `Send + Sync + 'static`.
    pub fn new_formatted<F>(config: Config, formatter: F) -> Box<dyn CloudWatch + Send + Sync>
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        let rt = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let access_key_id = config.get_access_key_id().to_string();
        let access_key_secret = config.get_access_key_secret().to_string();
        let session_token = config.get_session_token().map(|t| t.to_string());
        let expires_in = config.get_expires_in();
        let provider = config.get_provider();
        let region = config.get_region().to_string();

        let client = rt.block_on(async {
            let creds = aws_sdk_cloudwatchlogs::config::Credentials::new(
                access_key_id,
                access_key_secret,
                session_token,
                expires_in,
                provider,
            );

            let sdk_config = aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(region))
                .credentials_provider(creds)
                .load()
                .await; // Need tokio to fix this

            aws_sdk_cloudwatchlogs::Client::new(&sdk_config)
        });

        Box::new(SimpleCloudWatch {
            data: Arc::new(Data {
                client,
                rt,
                log_group: config.get_log_group().to_string(),
                formatter: Box::new(formatter),
            }),
        })
    }

    /// Initializes the service using AWS standard environment variables.
    ///
    /// It looks for `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_REGION`.
    ///
    /// # Arguments
    /// * `log_group` - The name of the CloudWatch Log Group to use.
    ///
    /// # Panics
    /// Panics if the internal Tokio [`Runtime`][`tokio::runtime::Runtime`] fails to initialize.
    ///
    /// # Example
    /// ```rust
    /// // Assumes AWS_REGION and credentials are set in the environment
    /// # use timber_rust::service::SimpleCloudWatch;
    /// let cw_service = SimpleCloudWatch::from_env("my-app-logs".to_string());
    /// ```
    pub fn from_env(log_group: String) -> Box<dyn CloudWatch + Send + Sync> {
        Self::from_env_formatted(log_group, StandardMessageFormatter {})
    }

    pub fn from_env_formatted<F>(
        log_group: String,
        formatter: F,
    ) -> Box<dyn CloudWatch + Send + Sync>
    where
        F: MessageFormatter + Send + Sync + 'static,
    {
        let rt = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let client = rt.block_on(async {
            let sdk_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
            aws_sdk_cloudwatchlogs::Client::new(&sdk_config)
        });

        Box::new(SimpleCloudWatch {
            data: Arc::new(Data {
                client,
                rt,
                log_group,
                formatter: Box::new(formatter),
            }),
        })
    }

    /// Returns the configured Log Group name.
    pub fn log_group(&self) -> &str {
        &self.data.log_group
    }

    /// Returns the internal data
    pub fn get_data(&self) -> Arc<Data> {
        self.data.clone()
    }
}

impl CloudWatch for SimpleCloudWatch {
    /// Checks if the service is operational.
    ///
    /// This performs a synchronous check against AWS to verify:
    /// 1. Connectivity and Credentials.
    /// 2. Existence of the target Log Group.
    fn status(&self) -> Status {
        let result = self.data.rt.block_on(async {
            self.data
                .client
                .describe_log_groups()
                .log_group_name_prefix(&self.data.log_group)
                .send()
                .await
        });

        match result {
            Ok(output) => {
                // Check if our specific log group is in the returned list
                let exists = output
                    .log_groups()
                    .iter()
                    .any(|g| g.log_group_name() == Some(&self.data.log_group));

                if exists {
                    Status::Running
                } else {
                    // If the group doesn't exist, PutLogEvents will fail.
                    Status::Broken
                }
            }
            Err(_) => {
                // Network error, 403 Forbidden, or expired tokens.
                Status::Broken
            }
        }
    }

    /// This method runs in a dedicated thread and implements a **greedy-drain** /// batching strategy to optimize network I/O:
    ///
    /// 1. **Idle Efficiency**: It uses a blocking `receiver.recv()` to wait for the
    ///    first message, ensuring the thread consumes zero CPU cycles when there is
    ///    no logging activity.
    /// 2. **Batch Formation**: Once the first message is received, it performs
    ///    non-blocking `try_recv()` calls to "drain" all currently pending messages
    ///    in the channel into a single batch.
    /// 3. **Synchronous Bridge**: It utilizes the internal Tokio [`Runtime`][`tokio::runtime::Runtime`] via
    ///    `block_on` to execute the asynchronous AWS SDK `PutLogEvents` call
    ///    synchronously within the worker thread.
    ///
    /// # Thread Safety
    /// This loop is designed to run indefinitely until the `receiver` is disconnected
    /// (usually when the application starts its shutdown sequence).
    fn work(&self, receiver: Receiver<CloudWatchMessage>) {
        let mut messages = Vec::with_capacity(128);

        // Get first message
        while let Ok(message) = receiver.recv() {
            // Clear buffer
            messages.clear();

            // Deal with message
            messages.push(message);

            // Get batch
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }

            let data = &self.data;
            data.rt.block_on(async {
                let mut log_events = Vec::with_capacity(messages.len());

                for msg in &messages {
                    let event = InputLogEvent::builder()
                        .message(data.formatter.format(&msg.message))
                        .timestamp(msg.timestamp) // El i64 que definimos antes
                        .build()
                        .expect("Failed to build log event");
                    log_events.push(event);
                }

                let _ = data
                    .client
                    .put_log_events()
                    .log_group_name(&data.log_group)
                    .log_stream_name("my-stream-name") // ¡OJO! Esto debe existir
                    .set_log_events(Some(log_events))
                    .send()
                    .await;
            });
        } // Receive message
    }
}

impl Fallback for SimpleCloudWatch {
    fn fallback(&self, error: &ServiceError, msg: &Message) {
        let now: chrono::DateTime<Utc> = msg.instant().into();
        let now = now.to_rfc3339_opts(SecondsFormat::Nanos, true);

        match error {
            ServiceError::Http(e) => {
                eprintln!("Cloudwatch rejected log: Status {}", e.status_code())
            }
            ServiceError::Network(e) => eprintln!("Cloudwatch network error: {}", e),
            _ => eprintln!("Cloudwatch service failure: {}", error),
        }

        println!("{} [{}] | {}", now, msg.level(), msg.content());
    }
}
