use std::fmt::Formatter;

use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

use crate::data::archive::ArchiveDocument;
use crate::result::{AppError, AppResult};

pub const METADATA_LIMIT: usize = 64 * 1024 * 1024;

struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON with unique object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();

        while let Some(value) = input.next_element_seed(Self)? {
            values.push(value);
        }

        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();

        while let Some(key) = input.next_key::<String>()? {
            let value = input.next_value_seed(Self)?;

            if values.insert(key, value).is_some() {
                return Err(serde::de::Error::custom("duplicate JSON key"));
            }
        }

        Ok(Value::Object(values))
    }
}

impl<'de> DeserializeSeed<'de> for UniqueVisitor {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

/// # Errors
/// Rejects oversized, duplicate-key or invalid format input without fallback.
pub fn parse_prk(bytes: &[u8]) -> AppResult<ArchiveDocument> {
    if bytes.len() > METADATA_LIMIT {
        return Err(AppError::InvalidInput);
    }

    let mut deserializer = serde_json::Deserializer::from_slice(bytes);

    let value = UniqueVisitor
        .deserialize(&mut deserializer)
        .map_err(|_| AppError::InvalidInput)?;

    deserializer.end().map_err(|_| AppError::InvalidInput)?;

    if value.get("format").is_some() || value.get("format_version").is_some() {
        return crate::complex::archive::native::parse(value);
    }

    if value.get("comic_title").is_some() {
        return crate::complex::archive::legacy::parse(&value, false);
    }

    crate::complex::archive::legacy::parse(&value, true)
}
