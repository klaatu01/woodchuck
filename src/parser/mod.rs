use crate::extension::log::CloudWatchLog;
use anyhow::{Error, Result};
use serde_json::Value;
use std::convert::TryFrom;

mod dotnet;
mod node;
mod python;

#[derive(Debug, Serialize, PartialEq)]
pub enum LogLevel {
    #[serde(rename(serialize = "INFO"))]
    Info,
    #[serde(rename(serialize = "WARN"))]
    Warn,
    #[serde(rename(serialize = "ERROR"))]
    Error,
}

#[derive(Debug, Serialize)]
pub struct StructuredLog {
    timestamp: Option<String>,
    guid: Option<String>,
    level: Option<LogLevel>,
    data: Value,
}

#[derive(Debug)]
pub enum Log {
    CloudWatch(StructuredLog),
    Preformatted(serde_json::Value)
}

impl ToString for Log {
    fn to_string(&self) -> String {
        match self {
            Log::CloudWatch(data) => serde_json::to_string(data).unwrap(),
            Log::Preformatted(data)=> serde_json::to_string(data).unwrap(),
        }
    }
}

impl TryFrom<String> for LogLevel {
    type Error = anyhow::Error;
    fn try_from(level: String) -> Result<Self> {
        match level.as_str() {
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(Error::msg(format!("Unable to parse {} as LogLevel", level))),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Parser;

impl Parser {
    pub fn parse(self, logs: Vec<CloudWatchLog>) -> Vec<Log> {
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
            .collect()
    }
}

fn try_parse_cloudwatch_log(log: &CloudWatchLog) -> Result<Log> {
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
    use super::try_parse_cloudwatch_log;
    use super::LogLevel;
    use super::CloudWatchLog;
    use super::Log;

    #[test]
    fn can_parse_node() {
        let input =
            CloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::CloudWatch(log) => {
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
        let input = CloudWatchLog {
            record: serde_json::Value::String(
                "[INFO]	2020-11-18T23:52:30.128Z    6e48723a-1596-4313-a9af-e4da9214d637	Hello World\n"
                    .to_string(),
            ),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::CloudWatch(log) => {
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
        let input = CloudWatchLog {
            record: serde_json::Value::String(
                "{ \"statusCode\": 200, \"body\": \"DotNet\" }".to_string(),
            ),
            time: "2020-11-18T23:52:30.128Z".to_string(),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output.unwrap() {
            Log::Preformatted(log) => {
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
        let input = CloudWatchLog { record: serde_json::Value::String("Bad log".to_string()), ..Default::default()};
        let output = try_parse_cloudwatch_log(&input);
        assert_eq!(output.is_err(), true);
    }
}
