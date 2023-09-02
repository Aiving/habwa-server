use chrono::{DateTime, FixedOffset};
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = format!("{}", date.format(FORMAT));

    serializer.serialize_str(&date)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let date = String::deserialize(deserializer)?;

    DateTime::parse_from_str(&date, FORMAT)
        .map_err(serde::de::Error::custom)
}
