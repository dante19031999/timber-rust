use crate::Message;
use serde_json::json;

/// Defines a strategy for converting a log message into a string format
/// compatible with CloudWatch Logs.
///
/// Implementations of this trait determine how the structured log data
/// (metadata, levels, and content) is serialized before transmission.
pub trait MessageFormatter: Send + Sync {
    /// Formats a single [`Message`] into its string representation.
    fn format(&self, message: &Message) -> String;
}

/// The default formatter for CloudWatch messages.
///
/// This formatter serializes messages into a **JSON** string. This is considered
/// a best practice for CloudWatch, as it enables advanced querying and filtering
/// using CloudWatch Logs Insights.
///
/// # Output Example
/// ```json
/// {
///   "level": "INFO",
///   "msg": "Application started"
/// }
/// ```
pub struct DefaultMessageFormatter {}

impl DefaultMessageFormatter {
    /// Creates a new instance of the default formatter.
    pub fn new() -> Self {
        Self {}
    }
}

impl MessageFormatter for DefaultMessageFormatter {
    /// Converts the message into a JSON string containing the log level and content.
    ///
    /// The resulting string is what will be displayed in the "Message" column
    /// of the CloudWatch Logs console.
    fn format(&self, message: &Message) -> String {
        json!({
            "level": message.level().to_string(),
            "msg": message.content().to_string(),
        })
        .to_string()
    }
}
