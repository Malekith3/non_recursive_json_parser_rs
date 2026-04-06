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
    return match k {
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

    UnexpectedEOF { at: usize },

    ExpectedEOF { found: TokenTag, at: Option<usize> },

    UnexpectedToken { found: TokenTag, at: Option<usize> },

    InvalidArray { found: TokenTag, at: Option<usize> },
    InvalidJsonObject { found: TokenTag, at: Option<usize> },

    LexError(LexerError),
}


// =============================================================================
// V3 PARSER ERROR
// =============================================================================
// Design principles vs v2:
//   - Every error carries `at: usize` (byte offset) for source location
//   - Errors that involve a token carry `found: TokenTag`
//   - Errors that involve frame state carry `frame: FrameTypeTag`
//   - TODO next session: upgrade `at: usize` to `at: SourceLocation`
//     once lexer Span is extended to track line + column (easy upgrade)

#[derive(Debug, Clone, PartialEq)]
pub enum JsonParsingErrorV3 {
    // Input was empty before lexing even started.
    // Same as v2 — no extra context needed.
    EmptyJsonFile,

    // Ran out of tokens while still inside a structure.
    // TODO: add `at: usize` — byte offset where EOF was hit
    // TODO: add `frame: FrameTypeTag` — which frame was open when EOF hit
    // Example: parsing {"a": [1,2 and then nothing
    UnexpectedEOF,

    // Parsing completed successfully but there are leftover tokens after the root value.
    // TODO: add `at: usize`
    // TODO: add `found: TokenTag` — the unexpected token that followed the root value
    ExpectedEOF,

    // A closing token didn't match the open frame.
    // Example: opened with '[' but saw '}' — mismatch
    // TODO: add `at: usize`
    // TODO: add `found: TokenTag` — the closing token we got (e.g. RBrace)
    // TODO: add `frame: FrameTypeTag` — the frame that was open (e.g. ArrayFrame)
    MismatchedClosing,

    // A closing token arrived but the stack was empty — nothing to close.
    // Example: "1, 2]" — the ] has no matching [
    // TODO: add `at: usize`
    // TODO: add `found: TokenTag` — the unexpected closing token
    UnexpectedClosing,

    // Inside an object, got a non-string value where a key was expected.
    // pending_key was None but the incoming value wasn't a JsonString.
    // Example: { 1: "value" } — 1 is not a valid key
    // TODO: add `at: usize`
    // TODO: add `found: TokenTag` — the token that was not a string
    ObjectKeyNotString,

    // Got '}' while pending_key is Some — key was parsed but value never came.
    // Example: { "a": } — key "a" consumed, colon consumed, but } arrived
    // TODO: add `at: usize`
    // TODO: add `found: String` — the key that was left hanging (useful for debugging)
    ObjectKeyWithoutValue,

    // Lexer failed — bubble it up unchanged, same as v2.
    LexError(LexerError),
    
    ///Duno if i want to surface that to user ... 
    UneExpectedFrameType
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

pub enum FrameTypeTag {
    ObjectFrame,
    ArrayFrame,
}
