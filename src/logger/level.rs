use std::borrow::Cow;

/// Severity levels for log categorization.
///
/// Variants represent a standard range of log priorities. This enum implements
/// [`From<Level>`] for [`Cow<'static, str>`], allowing it to satisfy the string-based
/// level requirements of the [`Loggable`][`crate::logger::Loggable`] trait without unnecessary allocations.
/// Represents the severity level of a log message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Designates fine-grained informational events that are most useful to debug an application.
    Debug,
    /// Designates informational messages that highlight the progress of the application
    /// at coarse-grained level.
    Info,
    /// Designates a successful operation or a positive milestone in the application flow.
    Success,
    /// Designates potentially harmful situations that should be monitored but do not
    /// stop the application.
    Warn,
    /// Designates error events that might still allow the application to continue running.
    Error,
    /// Designates severe error events that could lead the application to abort or
    /// lose critical functionality.
    Critical,
    /// Designates very severe error events that will presumably lead the application
    /// to terminate immediately.
    Fatal,
}

impl From<Level> for Cow<'static, str> {
    /// Converts a [`Level`] variant into its static string representation.
    ///
    /// This uses [`Cow::Borrowed`] to ensure zero-allocation during
    /// the conversion process.
    fn from(level: Level) -> Self {
        match level {
            Level::Debug => Cow::Borrowed("DEBUG"),
            Level::Info => Cow::Borrowed("INFO"),
            Level::Success => Cow::Borrowed("SUCCESS"),
            Level::Warn => Cow::Borrowed("WARN"),
            Level::Error => Cow::Borrowed("ERROR"),
            Level::Critical => Cow::Borrowed("CRITICAL"),
            Level::Fatal => Cow::Borrowed("FATAL"),
        }
    }
}
