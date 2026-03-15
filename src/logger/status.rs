
/// Represents the operational state of a logging [backend service][`crate::Service`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The service is active and ready to process messages.
    Running,
    /// The service has encountered a failure and cannot process messages.
    Broken,
}