use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Deserializer, Serializer};

pub mod b64 {
    use super::*;

    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        decode(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

pub mod b64_opt {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<u8>>, D::Error> {
        let Some(value) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };
        decode(&value).map(Some).map_err(serde::de::Error::custom)
    }
}

/// PATCH field: missing stays unchanged (`None` via `serde(default)`), JSON null clears
/// (`Some(None)`), and a base64 string sets the ciphertext (`Some(Some(bytes))`).
pub mod b64_clearable {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Option<Vec<u8>>>, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(Some(None)),
            Some(value) => decode(&value)
                .map(|bytes| Some(Some(bytes)))
                .map_err(serde::de::Error::custom),
        }
    }
}

pub mod b64_nullable {
    use super::*;

    pub fn serialize<S: Serializer>(
        data: &Option<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match data {
            Some(bytes) => serializer.serialize_str(&STANDARD.encode(bytes)),
            None => serializer.serialize_none(),
        }
    }
}

fn decode(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD.decode(value.trim())
}
