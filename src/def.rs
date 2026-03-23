use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Defines the execution strategy for log processing and delivery.
///
/// This setting determines whether the logging operations will block the
/// current thread or run concurrently using an asynchronous runtime.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Concurrency {
    /// Synchronous execution.
    ///
    /// Logging operations are performed on the caller's thread. The program
    /// execution will wait until the log is processed/sent before continuing.
    /// Recommended for CLI tools or simple scripts where latency is not critical.
    Sync,

    /// Asynchronous execution.
    ///
    /// Logging operations are offloaded to an async task. This prevents
    /// blocking the main application flow, making it suitable for high-performance
    /// servers and applications using runtimes like `tokio` or `async-std`.
    Async,
}

impl Display for Concurrency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Sync => write!(f, "sync"),
            Self::Async => write!(f, "async"),
        }
    }
}

/// Credentials for HTTP Basic Authentication.
///
/// This structure is used by network-based loggers (like Loki or CloudWatch)
/// to authenticate requests. It follows the standard `username:password`
/// pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuth {
    /// The identification string for the user or service account.
    ///
    /// In many cloud-native logging services (e.g., Grafana Cloud),
    /// this corresponds to the "Instance ID" or "User ID".
    pub username: String,
    /// The secret component of the credentials.
    ///
    /// This is optional because some internal proxies or development
    /// environments may only require a username (or use an empty password).
    /// When provided, it is combined with the username and Base64 encoded
    /// in the HTTP `Authorization` header.
    pub password: Option<String>,
}

impl BasicAuth {
    // Creates a new [`BasicAuth`]
    pub fn new<S1, S2>(username: S1, password: Option<S2>) -> BasicAuth
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        BasicAuth {
            username: username.into(),
            password: password.map(|password| password.into()),
        }
    }

    // Creates a new [`BasicAuth`] wrapped in [`Option`]
    pub fn some<S1, S2>(username: S1, password: Option<S2>) -> Option<BasicAuth>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Some(BasicAuth {
            username: username.into(),
            password: password.map(|password| password.into()),
        })
    }
}
