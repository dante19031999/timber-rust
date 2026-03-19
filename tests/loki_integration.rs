// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

#![cfg(feature = "loki")]
#![cfg_attr(docsrs, doc(cfg(feature = "loki")))]

use std::sync::Arc;
use std::time::{Duration, SystemTime};
use timber_rust::service::Loki as LokiService;
use timber_rust::service::StandardLoki as StandardLokiService;
use timber_rust::service::{BasicAuth, LokiConfig, LokiData, LokiMessage};
use timber_rust::{LoggerImpl, LoggerStatus, LokiLogger, MessageFactory};

#[test]
fn test_loki_config_full_builder() {
    let auth = BasicAuth {
        username: "admin".into(),
        password: Some("secret".into()),
    };

    let mut config = LokiConfig::new("http://loki:3100")
        .app("my-service")
        .job("backend")
        .env("staging")
        .basic_auth(Some(auth))
        .bearer_auth(Some("token123"))
        .connection_timeout(Duration::from_secs(10))
        .request_timeout(Duration::from_secs(60))
        .max_retries(5)
        .workers(4);

    // Verificaciones
    assert_eq!(config.get_url(), "http://loki:3100/");
    assert_eq!(config.get_app(), "my-service");
    assert_eq!(config.get_job(), "backend");
    assert_eq!(config.get_env(), "staging");

    // Verificamos el Option de BasicAuth
    let auth_res = config.get_basic_auth().unwrap();
    assert_eq!(auth_res.username(), "admin");
    assert_eq!(auth_res.password().unwrap(), "secret");

    assert_eq!(config.get_bearer_auth().unwrap(), "token123");
    assert_eq!(config.get_connection_timeout(), Duration::from_secs(10));
    assert_eq!(config.get_request_timeout(), Duration::from_secs(60));
    assert_eq!(config.get_max_retries(), 5);
    assert_eq!(config.get_workers(), 4);

    config = LokiConfig::with_labels("http://loki:3100", "my-service", "backend", "staging");

    // Verificaciones
    assert_eq!(config.get_url(), "http://loki:3100/");
    assert_eq!(config.get_app(), "my-service");
    assert_eq!(config.get_job(), "backend");
    assert_eq!(config.get_env(), "staging");
}

#[test]
#[ignore]
pub fn test_loki_service() {
    // Test 1
    let config = LokiConfig::new("http://localhost:3100");
    let loki = LokiLogger::new(config);
    let status = loki.status();
    assert_eq!(
        status,
        LoggerStatus::Running,
        "Loki service should be reachable at localhost:3100"
    );

    // Test 2
    let config = LokiConfig::new("http://localhost:3100");
    let client = LokiLogger::build_client(&config);
    let post_url = "http://localhost:3100/loki/api/v1/push".to_string();
    let message = MessageFactory::string_msg("normal", "hello world");

    let data = Arc::new(LokiData {
        config,
        client,
        post_url,
    });

    let loki = StandardLokiService {};
    let mut batch = vec![LokiMessage {
        message,
        timestamp: SystemTime::now(),
    }];
    let result = loki.work_batch("debug", &mut batch, &data);

    assert!(result.is_ok());
    assert_eq!(batch.len(), 0);
}
