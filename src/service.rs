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

pub use crate::service::write::DefaultMessageFormatter as DefaultWriteMessageFormatter;
pub use crate::service::write::MessageFormatter as WriteMessageFormatter;

pub use crate::service::write::BoxedIoService as BoxedIoWriteService;
pub use crate::service::write::DefaultBoxedIoService as DefaultBoxedIoWriteService;
pub use crate::service::write::DefaultFileWriteService;
pub use crate::service::write::FileWriteService;
pub use crate::service::write::IoService as IoWriteService;

pub use crate::service::write::DefaultStringService as DefaultStringWriteService;
pub use crate::service::write::FmtService as FmtWriteService;
pub use crate::service::write::StringService as StringWriteService;

pub use crate::service::write::CerrService as CerrWriteService;
pub use crate::service::write::CoutService as CoutWriteService;
pub use crate::service::write::DefaultCerrService as DefaultCerrWriteService;
pub use crate::service::write::DefaultCoutService as DefaultCoutWriteService;

pub use crate::service::write::BoxedFmtService as BoxedFmtWriteService;
pub use crate::service::write::DefaultBoxedFmtService as DefaultBoxedFmtWriteService;

pub use crate::service::write::ArcedFmtService as ArcedFmtWriteService;
pub use crate::service::write::DefaultArcedFmtService as DefaultArcedFmtWriteService;

#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Config as LokiConfig;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Data as LokiData;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::DefaultService as DefaultLokiService;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Message as LokiMessage;
#[cfg(feature = "loki")]
#[cfg_attr(docsrs, doc(cfg(feature = "loki")))]
pub use crate::service::loki::Service as LokiService;

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
