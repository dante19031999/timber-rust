// Copyright 2026 Dante Domenech Martinez dante19031999@gmail.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![doc = include_str!("../README.md")]
mod config;
mod factory;
pub mod logger;
mod manager;
mod message;
pub mod service;

pub use config::*;
pub use factory::*;
pub use manager::*;
pub use message::*;
pub use service::Fallback;
pub use service::Service;

pub use logger::Direct as DirectLogger;
pub use logger::Level as LogLevel;
pub use logger::Logger;
pub use logger::LoggerImpl;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use logger::Loki as LokiLogger;
pub use logger::Queued as QueuedLogger;
pub use logger::Silent as SilentLogger;
pub use logger::Status as LoggerStatus;

#[cfg(test)]
mod tests {}
