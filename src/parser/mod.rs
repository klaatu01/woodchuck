use crate::models::{Log, RawCloudWatchLog};
use anyhow::{Error, Result};
use serde_json::Value;

mod dotnet;
mod node;
mod python;

#[derive(Debug, Copy, Clone)]
pub struct Parser;

fn contains_woodchuck_ignore(obj : &Value) -> bool {
    match obj {
        Value::Object(obj) => 
            match obj.get("__WOODCHUCK_IGNORE__") {
                Some(Value::Bool(true)) => true,
                _ => false
            },
        _ => false
    }
}

fn include_log(log: &Log) -> bool {
    match log {
        Log::Formatted(data) => 
            !contains_woodchuck_ignore(data),
        _ => true
    }
}

impl Parser {
    pub fn parse(self, logs: Vec<RawCloudWatchLog>) -> Vec<Log> {
        logs.into_iter()
            .filter(|log| match log.r#type.as_str() {
                "function" => true,
                _ => {
                    println!("{:?}", log);
                    false
                }
            })
            .map(|log| match log.record {
                Value::String(_) => try_parse_cloudwatch_log(&log),
                _ => Err(Error::msg(format!("Expected String {}", log.record))),
            })
            .flatten()
            .filter(include_log)
            .collect()
    }
}

fn try_parse_cloudwatch_log(log: &RawCloudWatchLog) -> Result<Log> {
    match node::parse(log) {
        Some(dto) => {
            return Ok(dto);
        }
        _ => (),
    };
    match python::parse(log) {
        Some(dto) => {
            return Ok(dto);
        }
        _ => (),
    };
    match dotnet::parse(log) {
        Some(dto) => {
            return Ok(dto);
        }
        _ => (),
    };
    Err(Error::msg(format!("Unable to parse {:?}", log)))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::{try_parse_cloudwatch_log, include_log};
    use crate::models::{LogLevel, RawCloudWatchLog, Log};

    #[test]
    fn can_parse_node() {
        let input =
            RawCloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Unformatted(log) => {
                assert_eq!(log.timestamp.unwrap(), "2020-11-18T23:52:30.128Z");
                assert_eq!(log.guid.unwrap(), "6e48723a-1596-4313-a9af-e4da9214d637");
                assert_eq!(log.level.unwrap(), LogLevel::Info);
                assert_eq!(log.data, "Hello World\n");
            },
            _ => {
                panic!("Expected Cloudwatch formatted log");
            }
        }
    }

    #[test]
    fn can_parse_python() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "[INFO]	2020-11-18T23:52:30.128Z    6e48723a-1596-4313-a9af-e4da9214d637	Hello World\n"
                    .to_string(),
            ),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Unformatted(log) => {
                assert_eq!(log.timestamp.unwrap(), "2020-11-18T23:52:30.128Z");
                assert_eq!(log.guid.unwrap(), "6e48723a-1596-4313-a9af-e4da9214d637");
                assert_eq!(log.level.unwrap(), LogLevel::Info);
                assert_eq!(log.data, "Hello World\n");
            },
            _ => {
                panic!("Expected Cloudwatch formatted log");
            }
        }
    }

    #[test]
    fn can_parse_dotnet() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "{ \"statusCode\": 200, \"body\": \"DotNet\" }".to_string(),
            ),
            time: "2020-11-18T23:52:30.128Z".to_string(),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Formatted(log) => {
                assert_eq!(log["body"], "DotNet");
                assert_eq!(log["statusCode"], 200);
            }
            _ => {
                panic!("Expected Preformatted log");
            }
        }
    }

    #[test]
    fn cannot_parse() {
        let input = RawCloudWatchLog { record: serde_json::Value::String("Bad log".to_string()), ..Default::default()};
        let output = try_parse_cloudwatch_log(&input);
        assert_eq!(output.is_err(), true);
    }

    #[test]
    fn should_ignore_log_with_woodchuck_ignore_true() {
        let input =
            RawCloudWatchLog { 
                record: serde_json::Value::String(
                    "{ \"statusCode\": 200, \"__WOODCHUCK_IGNORE__\": true }".to_string(),
                ) , ..Default::default()
            };
        let log = try_parse_cloudwatch_log(&input).unwrap();
        
        assert_eq!(include_log(&log), false);
    }

    #[test]
    fn should_not_ignore_log_with_woodchuck_ignore_false() {
        let input =
            RawCloudWatchLog { 
                record: serde_json::Value::String(
                    "{ \"statusCode\": 200, \"__WOODCHUCK_IGNORE__\": false }".to_string(),
                ) , ..Default::default()
            };
        let log = try_parse_cloudwatch_log(&input).unwrap();
        
        assert_eq!(include_log(&log), true);
    }

    #[test]
    fn should_not_ignore_log_without_woodchuck_ignore() {
        let input =
            RawCloudWatchLog { 
                record: serde_json::Value::String(
                    "{ \"statusCode\": 200 }".to_string(),
                ) , ..Default::default()
            };
        let log = try_parse_cloudwatch_log(&input).unwrap();
        
        assert_eq!(include_log(&log), true);
    }
}
