// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "aws")]
#![cfg_attr(docsrs, doc(cfg(feature = "aws")))]

use crate::Message;
use crate::service::write::MessageFormatter;
use crate::service::{CoutWrite, ServiceError};
use serde_json::json;
use std::io::Write;

/// A [`MessageFormatter`] that serializes log messages into a single-line JSON format.
///
/// This formatter is specifically designed for AWS CloudWatch Logs when running in
/// environments like AWS Lambda, ECS, or Fargate, where `stdout` is automatically
/// captured and ingested as structured logs.
///
/// ### Output Format
/// ```json
/// {"level":"INFO","msg":"Your log message here"}
/// ```
#[derive(Default)]
pub struct CloudWatchCoutMessageFormatter {}

impl CloudWatchCoutMessageFormatter {
    /// Creates a new instance of [`CloudWatchCoutMessageFormatter`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl MessageFormatter for CloudWatchCoutMessageFormatter {
    /// Formats the [`Message`] as a JSON string and writes it to an IO stream.
    ///
    /// # Errors
    /// Returns [`ServiceError::Io`] if writing to the stream fails.
    fn format_io(
        &mut self,
        message: &Message,
        write: &mut (dyn Write + Send + Sync),
    ) -> Result<(), ServiceError> {
        let json_payload = json!({
            "level": message.level().to_string(),
            "msg": message.content().to_string(),
        })
        .to_string();

        // CloudWatch usually expects a newline per log entry
        writeln!(write, "{}", json_payload)?;
        Ok(())
    }

    /// Formats the [`Message`] as a JSON string and writes it to a core fmt formatter.
    fn format_fmt(
        &mut self,
        message: &Message,
        write: &mut (dyn std::fmt::Write + Send + Sync),
    ) -> Result<(), ServiceError> {
        let json_payload = json!({
            "level": message.level().to_string(),
            "msg": message.content().to_string(),
        })
        .to_string();

        writeln!(write, "{}", json_payload)?;
        Ok(())
    }
}

/// A specialized [`CoutWrite`] service that outputs JSON logs for AWS CloudWatch.
///
/// This service is specifically designed for AWS CloudWatch Logs when running in
/// environments like AWS Lambda, ECS, or Fargate, where `stdout` is automatically
/// captured and ingested as structured logs.
pub type CloudWatchCout = CoutWrite<CloudWatchCoutMessageFormatter>;
