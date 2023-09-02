use chrono::{DateTime, FixedOffset};
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub fn serialize<S>(date: &Option<DateTime<FixedOffset>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => serializer.serialize_some(&date.format(FORMAT).to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let date = <Option<String>>::deserialize(deserializer)?;

    if let Some(date) = date {
        return DateTime::parse_from_str(&date, FORMAT)
            .map(Some)
            .map_err(serde::de::Error::custom);
    }

    Ok(None)
}
