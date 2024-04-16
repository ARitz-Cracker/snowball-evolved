use std::{fmt, num::NonZeroU16, ops::Deref};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use indoc::formatdoc;
use schemars::{gen::SchemaGenerator, schema::SchemaObject, JsonSchema};
use static_assertions::{assert_eq_align, assert_eq_size};

use crate::response::body::BytesBody;


// This will hopefully result in a compiler error so we don't get UB when transmuting
assert_eq_size!(SerializableStatusCode, http::StatusCode);
assert_eq_align!(SerializableStatusCode, http::StatusCode);

/// A wrapper over `http::StatusCode`. Inherents all its methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BorshSchema, BorshSerialize)]
pub struct SerializableStatusCode(NonZeroU16);
impl BorshDeserialize for SerializableStatusCode {
	fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
		Ok(
			http::StatusCode::from_u16(u16::deserialize_reader(reader)?).map_err(|err| {
				std::io::Error::new(std::io::ErrorKind::InvalidData, err)
			})?.into()
		)
	}
}

impl From<http::StatusCode> for SerializableStatusCode {
	fn from(value: http::StatusCode) -> Self {
		// SAFETY: Both types of are the exact same size, alignment, and represent valid HTTP status codes
		unsafe {
			std::mem::transmute(value)
		}
	}
}
impl From<SerializableStatusCode> for http::StatusCode {
	fn from(value: SerializableStatusCode) -> Self {
		// SAFETY: Both types of are the exact same size, alignment, and represent valid HTTP status codes
		unsafe {
			std::mem::transmute(value)
		}
	}
}
impl Deref for SerializableStatusCode {
    type Target = http::StatusCode;
    fn deref(&self) -> &Self::Target {
		// SAFETY: Both types of are the exact same size, alignment, and represent valid HTTP status codes
		unsafe { &*(self as *const Self as *const Self::Target) }
    }
}
impl serde::Serialize for SerializableStatusCode {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer
	{
		<u16 as serde::Serialize>::serialize(&self.as_u16(), serializer)
	}
}
impl<'de> serde::Deserialize<'de> for SerializableStatusCode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
		deserializer.deserialize_u16(ValidStatusCodeVisitor)
	}
}
struct ValidStatusCodeVisitor;
impl<'de> serde::de::Visitor<'de> for ValidStatusCodeVisitor {
	type Value = SerializableStatusCode;
	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("HTTP status number to be 3 digits long")
	}
	fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> where
		E:serde::de::Error,
	{
		Ok(
			http::StatusCode::from_u16(v).map_err(|_| {
				E::custom(format!("{v} is not a valid HTTP status code"))
			})?.into()
		)
	}
}

impl JsonSchema for SerializableStatusCode {
	fn schema_name() -> String {
		"SerializableStatusCode".into()
	}

	fn json_schema(gen: &mut SchemaGenerator) -> schemars::schema::Schema {
		let mut schema: SchemaObject = u16::json_schema(gen).into();
		schema.number().minimum = Some(100.0);
		schema.number().maximum = Some(999.0);
		schema.into()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct HTTPStatusMessage {
	pub status: SerializableStatusCode,
	pub message: String,
	pub details: String
}
impl HTTPStatusMessage {
	pub fn new(status: http::StatusCode, message: impl Into<String>, details: impl Into<String>) -> Self {
		Self {
			status: status.into(),
			message: message.into(),
			details: details.into()
		}
	}
	pub fn into_plaintext_body(&self) -> BytesBody {
		let mut result = BytesBody::with_capacity(5);
		result.append(format!("{} {}\r\n", self.status.as_str(), self.status.canonical_reason().unwrap_or("Unknown")));
		result.append(self.message.clone());
		if self.details.len() > 0 {
			result.append("\r\n\r\n");
			result.append(self.details.clone());
		}
		result.append("\r\n");
		result
	}
	pub fn into_json_body(&self) -> Result<BytesBody, serde_json::Error> {
		Ok(
			if self.status.as_u16() >= 400 {
				BytesBody::new(serde_json::to_vec(&Err::<(), _>(self))?)
			}else{
				BytesBody::new(serde_json::to_vec(&Ok::<_, ()>(self))?)
			}
		)
	}
	pub fn into_html_body(&self) -> BytesBody {
		let status_string = self.status.to_string();
		let message = &self.message;
		let details = &self.details;
		// TODO: finish this with HTML escapes
		BytesBody::new(formatdoc!("
			<!DOCTYPE html>
			<html>
				<head>
					<title>{status_string}</title>
				</head>
				<body>
					<h1 style=\"text-align:center\"></h1>
				</body>
			</html>
		"))
	}
}
