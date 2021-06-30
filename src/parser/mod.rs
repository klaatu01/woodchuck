use crate::models::{Log, RawCloudWatchLog};
use anyhow::{Error, Result};
use serde_json::Value;

mod dotnet;
mod node;
mod python;

pub fn parse(logs: Vec<RawCloudWatchLog>) -> Vec<Log> {
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
    use super::try_parse_cloudwatch_log;
    use crate::models::{RawCloudWatchLog, Log};

    #[test]
    fn can_parse_node() {
        let input =
            RawCloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\t{ \"statusCode\": 200, \"body\": \"Node\" }\n".to_string())
                , ..Default::default()
            };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output {
            Ok (Log { record, attempts }) => {
                assert_eq!(record["body"], "Node");
                assert_eq!(record["statusCode"], 200);
            }
            _ => {
                panic!("Failed to parse log");
            }
        }
    }

    #[test]
    fn can_parse_python() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "[INFO]	2020-11-18T23:52:30.128Z    6e48723a-1596-4313-a9af-e4da9214d637	{ \"statusCode\": 200, \"body\": \"Python\" }\n"
                    .to_string(),
            ),
            ..Default::default()
        };
        let output = try_parse_cloudwatch_log(&input);

        assert_eq!(output.is_ok(), true);

        match output {
            Ok (Log { record, attempts }) => {
                assert_eq!(record["body"], "Python");
                assert_eq!(record["statusCode"], 200);
            }
            _ => {
                panic!("Failed to parse log");
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

        match output {
            Ok (Log { record, attempts }) => {
                assert_eq!(record["body"], "DotNet");
                assert_eq!(record["statusCode"], 200);
            }
            _ => {
                panic!("Failed to parse log");
            }
        }
    }

    #[test]
    fn cannot_parse() {
        let input = RawCloudWatchLog { record: serde_json::Value::String("Bad log".to_string()), ..Default::default()};
        let output = try_parse_cloudwatch_log(&input);
        assert_eq!(output.is_err(), true);
    }
}
