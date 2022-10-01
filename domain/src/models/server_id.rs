use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct ServerId(String);

impl ServerId {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4().to_string();
        Self(uuid)
    }
}

impl AsRef<str> for ServerId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ServerId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl From<String> for ServerId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for ServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for ServerId {
    fn serialize<S: Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ServerId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
        let value =
            String::deserialize(deserializer).map_err(|e| D::Error::custom(format!("{:?}", e)))?;
        Ok(Self::from(value))
    }
}
