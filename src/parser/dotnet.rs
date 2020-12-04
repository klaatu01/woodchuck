use super::StructuredLog;
use crate::log::CloudWatchLog;
use anyhow::Result;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Debug, Deserialize)]
struct DotnetCloudFormationLog {
    timestamp: String,
    data: serde_json::Value,
}

impl TryFrom<&CloudWatchLog> for DotnetCloudFormationLog {
    type Error = ();
    fn try_from(log: &CloudWatchLog) -> Result<Self, Self::Error> {
        match &log.record {
            serde_json::Value::String(record) => match serde_json::from_str(&record) {
                Ok(data) => Ok(DotnetCloudFormationLog {
                    timestamp: log.time.clone(),
                    data,
                }),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}

impl Into<StructuredLog> for DotnetCloudFormationLog {
    fn into(self) -> StructuredLog {
        StructuredLog {
            timestamp: Some(self.timestamp),
            guid: None,
            level: None,
            data: self.data,
        }
    }
}

pub fn parse(log: &CloudWatchLog) -> Option<StructuredLog> {
    let log: Result<DotnetCloudFormationLog, _> = DotnetCloudFormationLog::try_from(log);
    match log {
        Ok(l) => Some(l.into()),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::parse;
    use super::CloudWatchLog;

    #[test]
    fn test_parse_dotnet() {
        let input = CloudWatchLog {
            record: serde_json::Value::String(
                "{ \"statusCode\": 200, \"body\": \"DotNet\" }".to_string(),
            ),
            time: "2020-11-18T23:52:30.128Z".to_string(),
            ..Default::default()
        };
        let output = parse(&input);

        assert_eq!(output.is_some(), true);
        let output_unwrapped = output.unwrap();

        println!("{:?}", output_unwrapped);

        assert_eq!(
            output_unwrapped.timestamp.unwrap(),
            "2020-11-18T23:52:30.128Z"
        );
        assert_eq!(output_unwrapped.guid, None);
        assert_eq!(output_unwrapped.level, None);
        assert_eq!(output_unwrapped.data["statusCode"], 200);
        assert_eq!(output_unwrapped.data["body"], "DotNet");
    }
}
