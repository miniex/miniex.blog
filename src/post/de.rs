use chrono::{DateTime, FixedOffset};
use serde::{self, Deserialize, Deserializer, Serializer};

pub mod date_format {
    use super::*;

    const FORMAT: &str = "%Y/%m/%d %H:%M %:z";

    pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
