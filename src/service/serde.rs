// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "serde_tools")]
#![cfg_attr(docsrs, doc(cfg(feature = "serde_tools")))]

use std::time::Duration;

/// A helper type for flexible [`Duration`] deserialization.
///
/// This enum uses `#[serde(untagged)]` to support three different formats,
/// providing a more ergonomic configuration experience:
///
/// | Format | Example | Description |
/// | :--- | :--- | :--- |
/// | **Structured** | `{"secs": 1, "nsecs": 500}` | Full precision object. |
/// | **Integer** | `10` | Whole seconds. |
/// | **Floating** | `0.5` | Fractional seconds (e.g., 500ms). |
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub(crate) enum FlexibleDuration {
    /// Full precision representation with explicit seconds and nanoseconds.
    Full {
        /// Seconds component.
        secs: u64,
        /// Sub-second nanoseconds component.
        nsecs: u32,
    },
    /// Simple representation using whole seconds.
    Integer {
        /// Seconds value.
        secs: u64,
    },
    /// Fractional representation (e.g., `1.5` for 1 second and 500ms).
    Floating {
        /// Fractional seconds value.
        fsecs: f64,
    },
}

impl FlexibleDuration {
    /// Wraps a standard [`Duration`] into a [`FlexibleDuration`].
    pub(crate) fn from_duration(duration: Duration) -> Self {
        FlexibleDuration::Full {
            secs: duration.as_secs(),
            nsecs: duration.subsec_nanos(),
        }
    }

    /// Converts the helper into a standard [`Duration`].
    ///
    /// # Errors
    ///
    /// Returns a Serde custom error if:
    /// - The `Floating` variant contains a negative value.
    /// - The floating point value exceeds the limits of a `u64` for seconds.
    pub(crate) fn as_duration<E>(&self) -> Result<Duration, E>
    where
        E: serde::de::Error,
    {
        match *self {
            FlexibleDuration::Full { secs, nsecs } => Ok(Duration::new(secs, nsecs)),
            FlexibleDuration::Integer { secs } => Ok(Duration::from_secs(secs)),
            FlexibleDuration::Floating { fsecs } => {
                if fsecs < 0.0 {
                    return Err(E::custom("Duration cannot be negative"));
                }
                if fsecs > u64::MAX as f64 {
                    return Err(E::custom("Duration cannot surpass u64::MAX"));
                }

                // Conversión segura de punto flotante a duración de segundos y nanos
                let secs = fsecs as u64;
                let nanos = (fsecs.fract() * 1_000_000_000.0).round() as u32;

                Ok(Duration::new(secs, nanos))
            }
        }
    }
}

/// Authentication credentials for services requiring Basic Auth.
///
/// Can be easily constructed from a tuple of `(username, password)` where the
/// password is an `Option`.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct BasicAuth {
    /// The username for authentication.
    pub username: String,
    /// The optional password for authentication.
    pub password: Option<String>,
}

impl BasicAuth {
    /// Creates a new [`BasicAuth`] instance with the given credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use timber_rust::service::BasicAuth;
    /// let auth1 = BasicAuth::new("admin", Some("secret"));
    /// let auth2 : BasicAuth = ("admin", Some("secret")).into();
    /// ```
    pub fn new<S1, S2>(username: S1, password: Option<S2>) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        BasicAuth {
            username: username.into(),
            password: password.map(|pwd| pwd.into()),
        }
    }

    /// Returns the username as a string slice.
    ///
    /// This may represent a literal username, a User ID, or a Service Account name
    /// depending on the remote service requirements.
    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    /// Returns the password as an optional string slice.
    ///
    /// In modern APIs, this field often carries an API Token or a Private Key.
    /// Returns [`None`] if no password was provided.
    pub fn password(&self) -> Option<&str> {
        self.password.as_ref().map(|s| s.as_str())
    }
}

impl<S1, S2> From<(S1, Option<S2>)> for BasicAuth
where
    S1: Into<String>,
    S2: Into<String>,
{
    /// Allows creating [`BasicAuth`] from a tuple: `("admin", Some("secret"))`.
    fn from(data: (S1, Option<S2>)) -> Self {
        BasicAuth {
            username: data.0.into(),
            password: data.1.map(Into::into),
        }
    }
}
