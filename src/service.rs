// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#[cfg(feature = "aws")]
pub mod aws;
mod error;
mod fallback;
mod fv1hash;
#[cfg(feature = "loki")]
pub mod loki;
pub(crate) mod serde;
mod serror;
mod service;
pub mod write;

pub use error::*;
pub use fallback::*;
pub(crate) use fv1hash::*;

#[cfg(feature = "serde_tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde_tools")))]
pub use serde::BasicAuth;
pub use serror::*;
pub use service::*;

pub use crate::service::write::StandardMessageFormatter as StandardWriteMessageFormatter;
pub use crate::service::write::MessageFormatter as WriteMessageFormatter;

pub use crate::service::write::BoxedIo as BoxedIoWrite;
pub use crate::service::write::StandardBoxedIo as StandardBoxedIoWrite;
pub use crate::service::write::StandardFileWrite;
pub use crate::service::write::FileWrite;
pub use crate::service::write::Io as IoWrite;

pub use crate::service::write::StandardStringFmt as StandardStringFmtWrite;
pub use crate::service::write::Fmt as FmtWrite;
pub use crate::service::write::StringFmt as StringFmtWrite;

pub use crate::service::write::Cerr as CerrWrite;
pub use crate::service::write::Cout as CoutWrite;
pub use crate::service::write::StandardCerr as StandardCerrWrite;
pub use crate::service::write::StandardCout as StandardCoutWrite;

pub use crate::service::write::BoxedFmt as BoxedFmtWrite;
pub use crate::service::write::StandardBoxedFmt as StandardBoxedFmtWrite;

pub use crate::service::write::ArcedFmt as ArcedFmtWrite;
pub use crate::service::write::StandardArcedFmt as StandardArcedFmtWrite;

#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Loki;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::StandardLoki;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Config as LokiConfig;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Data as LokiData;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Message as LokiMessage;

#[cfg(feature = "aws")]
#[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
pub use crate::service::aws::CloudWatch;
#[cfg(feature = "aws")]
#[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
pub use crate::service::aws::Config as CloudWatchConfig;
#[cfg(feature = "aws")]
#[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
pub use crate::service::aws::Data as CloudWatchData;
#[cfg(feature = "aws")]
#[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
pub use crate::service::aws::Message as CloudWatchMessage;
#[cfg(feature = "aws")]
#[cfg_attr(docsrs, doc(cfg(feature = "aws")))]
pub use crate::service::aws::SimpleCloudWatch;
