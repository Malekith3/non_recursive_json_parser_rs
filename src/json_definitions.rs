use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonParsingError {
    EmptyJsonFile,
    InvalidJsonFile,
    LeadingZero,
    InvalidUnicodeInString,
    InvalidArray,
    InvalidJsonObject
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(IndexMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    JsonString(String),
    Number(f64),
    Boolean(bool),
    Null,
}
