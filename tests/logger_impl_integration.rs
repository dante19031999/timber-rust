// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Dante Doménech Martinez dante19031999@gmail.com

use std::borrow::Cow;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use timber_rust::service::{
    ArcedFmtWriteService, AtemporalMessageFormatter, DefaultMessageFormatter, MessageFormatter,
    StringWriteService,
};
use timber_rust::{QueuedLogger, LogLevel, Logger, MessageFactory, DirectLogger};

#[test]
pub fn test_default_message_formatter() {
    let mut formatter = DefaultMessageFormatter::new();

    let message = MessageFactory::string_msg(LogLevel::Debug, "Test message");

    // Esta regex acepta:
    // 1. Fecha con o sin nanosegundos
    // 2. Terminación en 'Z' o '+00:00'
    // 3. El nivel de log con sus espacios: [ DEBUG ]
    let iso_8601_regex = regex::Regex::new(
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})\s+\[\s+DEBUG\s+\]\s+Test message"
    ).unwrap();

    // Usamos un Cursor sobre un Vec<u8> para simular un destino I/O
    let mut buffer = Vec::new();
    let mut sink = Cursor::new(&mut buffer);

    // Ejecutamos el formateo
    formatter
        .format_io(&message, &mut sink)
        .expect("Could not format message");

    let output = String::from_utf8(buffer).expect("Output is not valid UTF-8");

    assert!(
        iso_8601_regex.is_match(output.trim()),
        "Output format mismatch! Got: {}",
        output
    );

    // Usamos un Cursor sobre un Vec<u8> para simular un destino I/O
    let mut buffer = String::with_capacity(128);

    // Ejecutamos el formateo
    formatter
        .format_fmt(&message, &mut buffer)
        .expect("Could not format message");

    assert!(
        iso_8601_regex.is_match(&buffer),
        "Output format mismatch! Got: {}",
        output
    );
}

#[test]
pub fn test_logger_level() {
    assert_eq!(Cow::from(LogLevel::Debug), "DEBUG");
    assert_eq!(Cow::from(LogLevel::Info), "INFO");
    assert_eq!(Cow::from(LogLevel::Success), "SUCCESS");
    assert_eq!(Cow::from(LogLevel::Warn), "WARN");
    assert_eq!(Cow::from(LogLevel::Error), "ERROR");
    assert_eq!(Cow::from(LogLevel::Critical), "CRITICAL");
    assert_eq!(Cow::from(LogLevel::Fatal), "FATAL");
}

#[test]
pub fn test_direct_logger() {
    let buffer = String::with_capacity(128);
    let formatter = AtemporalMessageFormatter {};
    let service = StringWriteService::<AtemporalMessageFormatter>::new(buffer, formatter);
    let logger_impl = DirectLogger::new(service, 0);
    let logger = Logger::new(logger_impl);

    logger
        .log(("DEBUG", "Hello world 1"))
        .log(("DEBUG", "Hello world 2"))
        .log(("DEBUG", "Hello world 3"));

    let expected = "\
[ DEBUG ] Hello world 1\n\
[ DEBUG ] Hello world 2\n\
[ DEBUG ] Hello world 3\n";

    let logger_impl = logger.get_implementation();
    let logger_impl = logger_impl
        .as_any()
        .downcast_ref::<DirectLogger>()
        .expect("Can't downcast to DirectLogger");

    let service = logger_impl
        .get_service()
        .as_any()
        .downcast_ref::<StringWriteService<AtemporalMessageFormatter>>()
        .expect("Can't downcast to AtemporalMessageFormatter");

    let res = service.inspect_writer(|writer| {
        assert_eq!(writer, expected, "Buffer content and expected do not math");
        true
    });
    assert!(res.is_some());
}

#[test]
pub fn test_queued_logger_1woker() {
    let buffer = Arc::new(Mutex::new(String::with_capacity(128)));
    let formatter = AtemporalMessageFormatter {};
    let service = ArcedFmtWriteService::new(buffer.clone(), formatter);
    let logger_impl = QueuedLogger::new(service, 0, 1);
    let logger = Logger::new(logger_impl);

    logger
        .log(("DEBUG", "Hello world 1"))
        .log(("DEBUG", "Hello world 2"))
        .log(("DEBUG", "Hello world 3"));

    // Wait for it to finsh
    drop(logger);

    let expected = "\
[ DEBUG ] Hello world 1\n\
[ DEBUG ] Hello world 2\n\
[ DEBUG ] Hello world 3\n";

    let string = buffer.lock().expect("Couldn't lock buffer");

    assert_eq!(
        string.as_str(),
        expected,
        "Buffer content and expected do not math"
    );
}

#[test]
pub fn test_queued_logger() {
    let buffer = Arc::new(Mutex::new(String::with_capacity(128)));
    let formatter = AtemporalMessageFormatter {};
    let service = ArcedFmtWriteService::new(buffer.clone(), formatter);
    let logger_impl = QueuedLogger::new(service, 0, 4);
    let logger = Logger::new(logger_impl);

    for i in 1..1000 {
        let line = format!("Hello world {}", i);
        logger.log(("DEBUG", line));
    }

    // Wait for it to finsh
    drop(logger);

    let string = buffer.lock().expect("Couldn't lock buffer");

    for i in 1..1000 {
        let line = format!("Hello world {}\n", i);
        assert!(string.contains(line.as_str()), "Log line not found");
    }
}
