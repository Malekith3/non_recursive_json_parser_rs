use indexmap::IndexMap;
use crate::json_lexer::{NumberError, StringError, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonParsingError {
    EmptyJsonFile,
    InvalidJsonFile,
    LeadingZero,
    InvalidUnicodeInString,
    InvalidArray,
    InvalidJsonObject,
    LexError(LexerError),
    UnexpectedEOF,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenTag {
    LBrace, RBrace, LBracket, RBracket, Colon, Comma,
    Bool, Null, String, Number, Eof,
}

pub fn token_tag_of(k: &TokenKind) -> TokenTag {
    match k {
        TokenKind::LBrace => TokenTag::LBrace,
        TokenKind::RBrace => TokenTag::RBrace,
        TokenKind::LBracket => TokenTag::LBracket,
        TokenKind::RBracket => TokenTag::RBracket,
        TokenKind::Colon => TokenTag::Colon,
        TokenKind::Comma => TokenTag::Comma,
        TokenKind::Bool(_) => TokenTag::Bool,
        TokenKind::Null => TokenTag::Null,
        TokenKind::String(_) => TokenTag::String,
        TokenKind::Number(_) => TokenTag::Number,
        TokenKind::Eof => TokenTag::Eof,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonParsingErrorV2 {
    EmptyJsonFile,

    UnexpectedEOF { at: Option<usize> },

    ExpectedEOF { found: TokenTag, at: Option<usize> },

    UnexpectedToken { found: TokenTag, at: Option<usize> },

    InvalidArray { found: TokenTag, at: Option<usize> },
    InvalidJsonObject { found: TokenTag, at: Option<usize> },

    LexError(LexerError),
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
