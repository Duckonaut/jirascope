use serde::Deserialize;
use ureq::serde_json;

/// Deserialize a Jira ID.
/// Sometimes Jira IDs are strings, sometimes they are integers.
/// Convert both to i64.
pub fn deserialize_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let id = serde_json::Value::deserialize(deserializer)?;
    match id {
        serde_json::Value::String(s) => s.parse::<i64>().map_err(serde::de::Error::custom),
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom(format!("Expected i64, got {:?}", n))),
        _ => Err(serde::de::Error::custom(format!(
            "Expected string or number, got {:?}",
            id
        ))),
    }
}

/// Serialize a Jira ID.
/// Always serialize as a string.
pub fn serialize_id<S>(id: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_id() {
        assert_eq!(
            deserialize_id(serde_json::Value::String("123".into())).unwrap(),
            123
        );
        assert_eq!(
            deserialize_id(serde_json::Value::Number(serde_json::Number::from(123))).unwrap(),
            123
        );
        assert!(deserialize_id(serde_json::Value::Bool(true)).is_err());
    }

    #[test]
    fn test_serialize_id() {
        let mut buf = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut buf);
        serialize_id(&123, &mut serializer).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "\"123\"");
    }
}
