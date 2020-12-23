use super::{LogLevel, StructuredLog, Log};
use crate::log::CloudWatchLog;
use recap::Recap;
use serde::Deserialize;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"(?x)
    (?P<timestamp>\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d\.\d+([+-][0-2]\d:[0-5]\d|Z))
    \s+
    (?P<guid>[0-9A-Fa-f]{8}[-][0-9A-Fa-f]{4}[-][0-9A-Fa-f]{4}[-][0-9A-Fa-f]{4}[-][0-9A-Fa-f]{12})
    \s+
    (?P<level>(INFO)|(WARN)|(ERROR))
    \s+
    (?P<data>(?s).*)
  "#)]
struct NodeCloudWatchLog {
    timestamp: String,
    guid: String,
    level: String,
    data: String,
}

impl Into<StructuredLog> for NodeCloudWatchLog {
    fn into(self) -> StructuredLog {
        StructuredLog {
            timestamp: Some(self.timestamp),
            guid: Some(self.guid),
            level: match self.level.as_str() {
                "INFO" => Some(LogLevel::Info),
                "WARN" => Some(LogLevel::Warn),
                "ERROR" => Some(LogLevel::Error),
                _ => None,
            },
            data: match serde_json::from_str(&self.data) {
                Ok(value) => value,
                Err(_) => serde_json::to_value(&self.data).unwrap(),
            },
        }
    }
}

pub fn parse(log: &CloudWatchLog) -> Option<Log> {
    match &log.record {
        serde_json::Value::String(record) => 
            match record.parse() as Result<NodeCloudWatchLog, _> {
                Ok(l) => {
                    let structured_log: StructuredLog = l.into();
                    match structured_log.data {
                        serde_json::Value::Object(_) => Some(Log::Preformatted(structured_log.data)),
                        _ => Some(Log::CloudWatch(structured_log)),
                    }
                },
                _ => None
            }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::super::{Log,LogLevel};
    use super::parse;
    use super::CloudWatchLog;


    #[test]
    fn test_parse_node_non_json() {
        let input =
            CloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            };
        let output = parse(&input);

        assert_eq!(output.is_some(), true);


        match output.unwrap() {
            Log::CloudWatch(log) => {
                assert_eq!(
                    log.timestamp.unwrap(),
                    "2020-11-18T23:52:30.128Z"
                );
                assert_eq!(
                    log.guid.unwrap(),
                    "6e48723a-1596-4313-a9af-e4da9214d637"
                );
                assert_eq!(log.level.unwrap(), LogLevel::Info);
                assert_eq!(log.data, "Hello World\n");
            },
            _ => {
                panic!("Expected Preformatted log");
            }
        }
    }

    #[test]
    fn test_parse_node_json() {
        let input =
            CloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\t{\"data\":\"Hello World\"}\n".to_string())
                , ..Default::default()
            };
        let output = parse(&input);

        assert_eq!(output.is_some(), true);

        let l = output.unwrap();

        println!("{}", l.to_string());

        match l {
            Log::Preformatted(log) => {
                assert_eq!(log["data"], "Hello World");
            },
            _ => {
                panic!("Expected Preformatted log");
            }
        }
    }
}
