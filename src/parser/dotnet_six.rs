use crate::models::{Log, LogLevel, RawCloudWatchLog, StructuredLog};
use recap::Recap;
use serde::Deserialize;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"(?x)
    (?P<timestamp>\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d\.\d+([+-][0-2]\d:[0-5]\d|Z))
    \s+
    (?P<guid>[0-9A-Fa-f]{8}[-][0-9A-Fa-f]{4}[-][0-9A-Fa-f]{4}[-][0-9A-Fa-f]{4}[-][0-9A-Fa-f]{12})
    \s+
    (?P<level>(info)|(warn)|(fail)|(trce)|(dbug)|(crit))
    \s+
    (?P<data>(?s).*)
  "#)]
struct DotnetSixCloudWatchLog {
    timestamp: String,
    guid: String,
    level: String,
    data: String,
}

impl Into<StructuredLog> for DotnetSixCloudWatchLog {
    fn into(self) -> StructuredLog {
        StructuredLog {
            timestamp: Some(self.timestamp),
            guid: Some(self.guid),
            level: match self.level.as_str() {
                "info" => Some(LogLevel::Info),
                "warn" => Some(LogLevel::Warn),
                "fail" => Some(LogLevel::Error),
                "crit" => Some(LogLevel::Critical),
                "dbug" => Some(LogLevel::Debug),
                "trce" => Some(LogLevel::Trace),
                _ => None,
            },
            data: match serde_json::from_str(&self.data) {
                Ok(value) => value,
                Err(_) => serde_json::to_value(&self.data).unwrap(),
            },
        }
    }
}

pub fn parse(log: &RawCloudWatchLog) -> Option<Log> {
    match &log.record {
        serde_json::Value::String(record) => {
            match record.parse() as Result<DotnetSixCloudWatchLog, _> {
                Ok(l) => Some(Log::Unformatted(l.into())),
                Err(_) => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::parse;
    use super::Log;
    use super::RawCloudWatchLog;
    use crate::models::LogLevel;

    #[test]
    fn test_parse_dotnet_six() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "2019-10-23T14:40:59.59Z	313e0588-e4f1-4d19-8ae4-44980a446805	info	Hello World\n"
                    .to_string(),
            ),
            ..Default::default()
        };
        let output = parse(&input);

        assert!(output.is_some());
        match output.unwrap() {
            Log::Unformatted(log) => {
                assert_eq!(log.timestamp.unwrap(), "2019-10-23T14:40:59.59Z");
                assert_eq!(log.guid.unwrap(), "313e0588-e4f1-4d19-8ae4-44980a446805");
                assert_eq!(log.level.unwrap(), LogLevel::Info);
                assert_eq!(log.data, "Hello World\n");
            }
            _ => {
                panic!("Expected CloudWatch Formatted log");
            }
        }
    }
}
