use crate::models::{Log, RawCloudWatchLog};
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

pub fn parse(log: &RawCloudWatchLog) -> Option<Log> {
    match &log.record {
        serde_json::Value::String(record) => 
            match record.parse() as Result<NodeCloudWatchLog, _> {
                Ok(l) => {
                    match serde_json::from_str(&l.data) {
                        Ok(value) => Some(Log::new(value)),
                        Err(_) => None,
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
    use crate::models::{Log, RawCloudWatchLog};
    use super::parse;

    #[test]
    fn test_parse_node_non_json() {
        let input =
            RawCloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\tHello World\n".to_string())
                , ..Default::default()
            };
        let output = parse(&input);

        assert!(output.is_none());
    }

    #[test]
    fn test_parse_node_json() {
        let input =
            RawCloudWatchLog { 
                record:
            serde_json::Value::String("2020-11-18T23:52:30.128Z\t6e48723a-1596-4313-a9af-e4da9214d637\tINFO\t{\"data\":\"Hello World\"}\n".to_string())
                , ..Default::default()
            };
        let output = parse(&input);

        assert_eq!(output.is_some(), true);

        match output {
            Some (Log { record, attempts:_ } )=> {
                assert_eq!(record["data"], "Hello World");
            },
            _ => {
                panic!("Expected Preformatted log");
            }
        }
    }
}
