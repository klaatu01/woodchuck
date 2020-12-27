use crate::models::{Log, RawCloudWatchLog};

pub fn parse(log: &RawCloudWatchLog) -> Option<Log> {
    match &log.record {
        serde_json::Value::String(record) => match serde_json::from_str(&record) {
            Ok(data) => Some(Log::Formatted(data)),
            Err(_) => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::parse;
    use super::Log;
    use super::RawCloudWatchLog;

    #[test]
    fn test_parse_dotnet() {
        let input = RawCloudWatchLog {
            record: serde_json::Value::String(
                "{ \"statusCode\": 200, \"body\": \"DotNet\", \"level\": \"info\"  }".to_string(),
            ),
            time: "2020-11-18T23:52:30.128Z".to_string(),
            ..Default::default()
        };
        let output = parse(&input);

        assert_eq!(output.is_some(), true);
        match output.unwrap() {
            Log::Formatted(log) => {
                assert_eq!(log["statusCode"], 200);
                assert_eq!(log["body"], "DotNet");
            }
            _ => {
                panic!("Expected Preformatted log");
            }
        }
    }
}
