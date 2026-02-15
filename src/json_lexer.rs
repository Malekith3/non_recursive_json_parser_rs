use std::collections::VecDeque;
use std::str::from_utf8;
use crate::json_definitions::LexerError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Punctuation (single-byte structural tokens)
    LBrace,   // '{'
    RBrace,   // '}'
    LBracket, // '['
    RBracket, // ']'
    Colon,    // ':'
    Comma,    // ','

    // Literals (exact keywords)
    Bool(bool), // "true" or "false"
    Null,       // "null"

    // Atoms (already decoded/validated by the lexer)
    String(String), // JSON string value (unescaped, UTF-8 validated)
    Number(f64),    // JSON number value (grammar-validated, parsed)
    Eof,            // end of input sentinel
}

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub enum NumberError {
    Empty,                 // nothing after '-' or no digits
    LeadingZero,           // "01"
    MissingFracDigit,      // "1."
    MissingExpDigit,       // "1e" or "1e+"
    InvalidChar,           // sign/dot/e in wrong place
    ParseFloatFailed,      // Rust parse failed (should be rare if FSM correct)
    NonFinite,             // parsed to inf/nan (policy choice)
}

#[derive(Debug, PartialEq, Clone)]
pub enum StringError {
    Unterminated,                 // hit EOF before closing quote
    InvalidEscape { found: u8 },  // e.g. "\x"
    TrailingBackslash,            // ends right after '\'
    ControlChar { found: u8 },    // raw < 0x20 in string
    InvalidUtf8,                  // if you decide to validate input bytes in strings
    InvalidUnicodeEscape,         // "\u" not followed by 4 hex
    SurrogateNotAllowed,          // D800–DFFF
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub fn lex_all(input: &[u8]) -> Result<VecDeque<Token>, LexerError> {
    let mut cursor = 0;
    let mut tokens = VecDeque::new();

    loop {
        let tok = next_token(input, &mut cursor)?;
        let is_eof = matches!(tok.kind, TokenKind::Eof);
        tokens.push_back(tok);
        if is_eof { break; }
    }

    Ok(tokens)
}

fn skip_ws(json_bytes_string: &[u8], current_index: &mut usize) {
    while *current_index < json_bytes_string.len() {
        let current_byte = json_bytes_string[*current_index];
        match current_byte {
            b' ' | b'\n' | b'\r' | b'\t' => {
                *current_index += 1;
            }

            _ => break
        }
    }
}

fn consume_one(cursor: &mut usize) -> Span {
    let start = *cursor;
    *cursor += 1;
    Span { start, end: *cursor } // [start, end)
}

pub fn consume_literal(
    bytes: &[u8],
    cursor: &mut usize,
    lit: &[u8],
    expected_name: &'static str,
) -> Result<Span, LexerError> {
    let start = *cursor;
    let end = start + lit.len();

    if end > bytes.len() {
        return Err(LexerError::UnexpectedEof {
            at: bytes.len(),
            expected: expected_name,
        });
    }

    if &bytes[start..end] != lit {
        return Err(LexerError::InvalidLiteral {
            at: start,
            expected: expected_name,
        });
    }

    *cursor = end;
    Ok(Span { start, end })
}
fn consume_bool(bytes: &[u8], cursor: &mut usize) -> Result<Token, LexerError> {
    const TRUE_BYTES: &[u8] = b"true";
    const FALSE_BYTES: &[u8] = b"false";

    let b = bytes[*cursor];

    match b {
        b't' => {
            let span = consume_literal(bytes, cursor, TRUE_BYTES, "true")?;
            Ok(Token { kind: TokenKind::Bool(true), span })
        }
        b'f' => {
            let span = consume_literal(bytes, cursor, FALSE_BYTES, "false")?;
            Ok(Token { kind: TokenKind::Bool(false), span })
        }
        _ => Err(LexerError::UnexpectedByte {
            at: *cursor,
            found: b,
            expected: "bool literal",
        }),
    }
}

fn consume_null(bytes: &[u8], cursor: &mut usize) -> Result<Token, LexerError> {
    const NULL_LITERAL: &[u8] = b"null";
    let span = consume_literal(bytes, cursor, NULL_LITERAL, "null")?;
    Ok(Token { kind: TokenKind::Null, span })
}

fn next_token(bytes: &[u8], cursor: &mut usize) -> Result<Token, LexerError> {
    skip_ws(bytes, cursor);

    if *cursor == bytes.len() {
        return Ok(Token {
            kind: TokenKind::Eof,
            span: Span { start: *cursor, end: *cursor },
        });
    }

    if *cursor > bytes.len() {
        return Err(LexerError::CursorOutOfBounds {
            cursor: *cursor,
            len: bytes.len(),
        });
    }

    let current_byte = bytes[*cursor];

    match current_byte {
        b'-' | b'0'..=b'9' => consume_number(bytes, cursor),
        b'\"' => consume_string(bytes, cursor),
        b'n' => consume_null(bytes, cursor),
        b't' | b'f' => consume_bool(bytes, cursor),

        b'[' => Ok(Token { kind: TokenKind::LBracket, span: consume_one(cursor) }),
        b']' => Ok(Token { kind: TokenKind::RBracket, span: consume_one(cursor) }),
        b'{' => Ok(Token { kind: TokenKind::LBrace, span: consume_one(cursor) }),
        b'}' => Ok(Token { kind: TokenKind::RBrace, span: consume_one(cursor) }),
        b':' => Ok(Token { kind: TokenKind::Colon, span: consume_one(cursor) }),
        b',' => Ok(Token { kind: TokenKind::Comma, span: consume_one(cursor) }),

        _ => Err(LexerError::UnexpectedByte {
            at: *cursor,
            found: current_byte,
            expected: "token",
        }),
    }
}

fn consume_escape_char(bytes: &[u8], cursor: &mut usize) -> Result<([u8; 4], usize), LexerError> {
    // We are at '\' already.
    *cursor += 1;

    if *cursor >= bytes.len() {
        return Err(LexerError::InvalidString {
            at: *cursor,
            reason: StringError::TrailingBackslash,
        });
    }

    let mut out = [0u8; 4];

    match bytes[*cursor] {
        b'"' => {
            out[0] = b'"';
            *cursor += 1;
            Ok((out, 1))
        }
        b'\\' => {
            out[0] = b'\\';
            *cursor += 1;
            Ok((out, 1))
        }
        b'/' => {
            out[0] = b'/';
            *cursor += 1;
            Ok((out, 1))
        }
        b'b' => {
            out[0] = 0x08;
            *cursor += 1;
            Ok((out, 1))
        }
        b'f' => {
            out[0] = 0x0C;
            *cursor += 1;
            Ok((out, 1))
        }
        b'n' => {
            out[0] = b'\n';
            *cursor += 1;
            Ok((out, 1))
        }
        b'r' => {
            out[0] = b'\r';
            *cursor += 1;
            Ok((out, 1))
        }
        b't' => {
            out[0] = b'\t';
            *cursor += 1;
            Ok((out, 1))
        }
        b'u' => {
            // cursor currently points at 'u'
            consume_unicode(bytes, cursor)
        }
        _ => Err(LexerError::InvalidString {
            at: *cursor,
            reason: StringError::InvalidEscape { found: bytes[*cursor] },
        }),
    }
}

fn consume_unicode(bytes: &[u8], cursor: &mut usize) -> Result<([u8; 4], usize), LexerError> {
    // We are at 'u'
    *cursor += 1;

    // Need 4 hex digits
    if *cursor + 4 > bytes.len() {
        return Err(LexerError::InvalidString {
            at: bytes.len(),
            reason: StringError::InvalidUnicodeEscape,
        });
    }

    let mut nibbles = [0u8; 4];

    for i in 0..4 {
        let b = bytes[*cursor];

        let v = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => (b - b'a') + 0xA,
            b'A'..=b'F' => (b - b'A') + 0xA,
            _ => {
                return Err(LexerError::InvalidString {
                    at: *cursor,
                    reason: StringError::InvalidUnicodeEscape,
                });
            }
        };

        nibbles[i] = v;
        *cursor += 1;
    }

    let code: u32 =
            ((nibbles[0] as u32) << 12) |
            ((nibbles[1] as u32) << 8)  |
            ((nibbles[2] as u32) << 4)  |
            ((nibbles[3] as u32) << 0);

    if code >= 0xD800 && code <= 0xDFFF {
        return Err(LexerError::InvalidString {
            at: *cursor,
            reason: StringError::SurrogateNotAllowed,
        });
    }

    let unicode_char = match char::from_u32(code) {
        Some(c) => c,
        None => {
            return Err(LexerError::InvalidString {
                at: *cursor,
                reason: StringError::InvalidUnicodeEscape,
            });
        }
    };

    let mut buf = [0u8; 4];
    let len = unicode_char.encode_utf8(&mut buf).len();

    Ok((buf, len))
}

fn consume_string(bytes: &[u8], cursor: &mut usize) -> Result<Token, LexerError> {
    let string_start = *cursor;

    // consume opening quote
    *cursor += 1;

    let mut out: Vec<u8> = Vec::new();
    let mut run_start = *cursor;
    let mut closed = false;

    while *cursor < bytes.len() {
        match bytes[*cursor] {
            b'\\' => {
                out.extend_from_slice(&bytes[run_start..*cursor]);

                let (buf, n) = consume_escape_char(bytes, cursor)?;
                out.extend_from_slice(&buf[..n]);

                run_start = *cursor;
            }

            b'"' => {
                out.extend_from_slice(&bytes[run_start..*cursor]);

                *cursor += 1;
                closed = true;
                break;
            }

            0x00..=0x1F => {
                return Err(LexerError::InvalidString {
                    at: *cursor,
                    reason: StringError::ControlChar { found: bytes[*cursor] },
                });
            }

            _ => {
                *cursor += 1;
            }
        }
    }

    if !closed {
        return Err(LexerError::InvalidString {
            at: bytes.len(),
            reason: StringError::Unterminated,
        });
    }

    let s = match from_utf8(&out) {
        Ok(v) => v.to_owned(),
        Err(_) => {
            return Err(LexerError::InvalidString {
                at: *cursor,
                reason: StringError::InvalidUtf8,
            });
        }
    };

    Ok(Token {
        kind: TokenKind::String(s),
        span: Span { start: string_start, end: *cursor },
    })
}

fn consume_number(bytes: &[u8], cursor: &mut usize) -> Result<Token, LexerError> {
    let start = *cursor;

    if bytes[*cursor] == b'-' {
        *cursor += 1;
        if *cursor >= bytes.len() {
            return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::Empty });
        }
    }

    // must have at least one digit
    if *cursor >= bytes.len() || !bytes[*cursor].is_ascii_digit() {
        return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::Empty });
    }

    // reject leading zeros like 01 (but allow "0", "0.xxx", "0e..")
    if bytes[*cursor] == b'0' {
        let next = *cursor + 1;
        if next < bytes.len() && bytes[next].is_ascii_digit() {
            return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::LeadingZero });
        }
    }

    let mut seen_dot = false;
    let mut seen_exp = false;

    let mut need_frac_digit = false;
    let mut need_exp_digit = false;

    let mut exp_sign_allowed = false;

    while *cursor < bytes.len() {
        let b = bytes[*cursor];

        match b {
            b'0'..=b'9' => {
                if need_frac_digit { need_frac_digit = false; }
                if need_exp_digit { need_exp_digit = false; }
                exp_sign_allowed = false;
                *cursor += 1;
            }

            b'.' => {
                if seen_dot || seen_exp {
                    return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::InvalidChar });
                }
                seen_dot = true;
                need_frac_digit = true;
                exp_sign_allowed = false;
                *cursor += 1;
            }

            b'e' | b'E' => {
                if seen_exp || need_frac_digit {
                    return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::InvalidChar });
                }
                seen_exp = true;
                need_exp_digit = true;
                exp_sign_allowed = true;
                *cursor += 1;
            }

            b'+' | b'-' => {
                if !exp_sign_allowed {
                    return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::InvalidChar });
                }
                exp_sign_allowed = false;
                *cursor += 1;
            }

            _ => break, // end of number token
        }
    }

    if need_frac_digit {
        return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::MissingFracDigit });
    }

    if need_exp_digit {
        return Err(LexerError::InvalidNumber { at: *cursor, reason: NumberError::MissingExpDigit });
    }

    let end = *cursor;

    // Parse the exact lexeme slice (includes '-' if present)
    let s = match std::str::from_utf8(&bytes[start..end]) {
        Ok(v) => v,
        Err(_) => {
            // Shouldn't happen for ascii number grammar, but keep it explicit.
            return Err(LexerError::InvalidNumber { at: start, reason: NumberError::ParseFloatFailed });
        }
    };

    let num: f64 = match s.parse() {
        Ok(v) => v,
        Err(_) => return Err(LexerError::InvalidNumber { at: start, reason: NumberError::ParseFloatFailed }),
    };

    // Policy choice: reject NaN/Inf or allow?
    if !num.is_finite() {
        return Err(LexerError::InvalidNumber { at: start, reason: NumberError::NonFinite });
    }

    Ok(Token {
        kind: TokenKind::Number(num),
        span: Span { start, end },
    })
}

