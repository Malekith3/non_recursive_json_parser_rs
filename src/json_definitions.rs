use indexmap::IndexMap;
use crate::json_lexer::{NumberError, StringError};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonParsingError {
    EmptyJsonFile,
    InvalidJsonFile,
    LeadingZero,
    InvalidUnicodeInString,
    InvalidArray,
    InvalidJsonObject,
    LexError(LexerError),
    UnexpectedEOF
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(IndexMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    JsonString(String),
    Number(f64),
    Boolean(bool),
    Null,
    EOF
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexerError {
    /// Cursor ended up past the input length (should be rare, but you want runtime error)
    CursorOutOfBounds { cursor: usize, len: usize },

    /// Ran out of input while trying to consume something specific (e.g. "true")
    UnexpectedEof { at: usize, expected: &'static str },

    /// Current byte can't start any token (or a specific byte was expected)
    UnexpectedByte { at: usize, found: u8, expected: &'static str },

    /// The bytes at `at` did not match the expected literal (e.g. "true")
    InvalidLiteral { at: usize, expected: &'static str },

    InvalidString { at: usize, reason: StringError },

    InvalidNumber { at: usize, reason: NumberError },
}
