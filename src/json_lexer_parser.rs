use std::collections::VecDeque;
use indexmap::IndexMap;
use crate::json_definitions::{JsonParsingError, JsonValue};
use crate::json_definitions::JsonParsingError::{InvalidJsonFile};
use crate::json_definitions::JsonValue::JsonString;
use crate::json_lexer::{lex_all, Token, TokenKind};

pub fn process_json_string_v2(json_string: &str) -> Result<JsonValue, JsonParsingError>{

    if json_string.is_empty() {
        return Err(JsonParsingError::EmptyJsonFile);
    }

    let mut tokens = lex_all(json_string.as_bytes()).map_err(|e|JsonParsingError::LexError(e))?;
    let json_value = parse_json_value(&mut tokens)?;

    let Some(token) = next_token(&mut tokens) else {
        return Err(JsonParsingError::UnexpectedEOF)
    };

    if token.kind != TokenKind::Eof {
        return Err(JsonParsingError::InvalidJsonFile)
    }

    Ok(json_value)
}

fn peek_token(tokens: &VecDeque<Token>) -> Option<&Token> {
    tokens.front()
}

fn next_token(tokens: &mut VecDeque<Token>) -> Option<Token>{
    tokens.pop_front()
}

fn parse_json_value(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonParsingError> {

    let Some(token)   = next_token(tokens) else {
        return Err(InvalidJsonFile)
    };

    return match token.kind {
        TokenKind::Null => Ok(JsonValue::Null),
        TokenKind::Bool(b) => Ok(JsonValue::Boolean(b)),
        TokenKind::String(s) => Ok(JsonValue::JsonString(s)),
        TokenKind::Number(n) => Ok(JsonValue::Number(n)),
        TokenKind::LBrace => parse_json_object(tokens),
        TokenKind::LBracket => parse_json_array(tokens),
        TokenKind::Eof => Err(JsonParsingError::UnexpectedEOF),
        _ => Err(JsonParsingError::InvalidJsonFile),
    }
}

fn parse_json_array(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonParsingError> {

    let Some(t0) = peek_token(tokens) else {
        return Err(JsonParsingError::UnexpectedEOF);
    };

    if t0.kind == TokenKind::RBracket {
        let _ = next_token(tokens);
        return Ok(JsonValue::Array(Vec::new()));
    }

    let mut out: Vec<JsonValue> = Vec::new();

    let v0 = parse_json_value(tokens)?;
    out.push(v0);

    loop {
        let Some(t) = peek_token(tokens) else {
            return Err(JsonParsingError::UnexpectedEOF);
        };

        match t.kind {
            TokenKind::Comma => {
                // consume ','
                let _ = next_token(tokens);

                // must have another value after comma
                let v = parse_json_value(tokens)?;
                out.push(v);
            }

            TokenKind::RBracket => {
                let _ = next_token(tokens);
                return Ok(JsonValue::Array(out));
            }

            _ => {
                return Err(JsonParsingError::InvalidArray);
            }
        }
    }
}


fn parse_json_object(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonParsingError> {
    // We enter here after '{' has already been consumed.

    // empty object: "{}"
    let Some(t0) = peek_token(tokens) else {
        return Err(JsonParsingError::UnexpectedEOF);
    };

    if t0.kind == TokenKind::RBrace {
        let _ = next_token(tokens); // consume '}'
        return Ok(JsonValue::Object(IndexMap::new()));
    }

    let mut out: IndexMap<String, JsonValue> = IndexMap::new();

    // First pair: key ":" value
    let key = parse_object_key(tokens)?;
    consume_kind(tokens, TokenKind::Colon)?;
    let value = parse_json_value(tokens)?;
    out.insert(key, value);

    loop {
        let Some(t) = peek_token(tokens) else {
            return Err(JsonParsingError::UnexpectedEOF);
        };

        match &t.kind {
            TokenKind::Comma => {
                let _ = next_token(tokens); // consume ','

                let key = parse_object_key(tokens)?;
                consume_kind(tokens, TokenKind::Colon)?;
                let value = parse_json_value(tokens)?;
                out.insert(key, value);
            }

            TokenKind::RBrace => {
                let _ = next_token(tokens); // consume '}'
                return Ok(JsonValue::Object(out));
            }

            _ => {
                return Err(JsonParsingError::InvalidJsonObject);
            }
        }
    }
}

fn parse_object_key(tokens: &mut VecDeque<Token>) -> Result<String, JsonParsingError> {
    let Some(t) = next_token(tokens) else {
        return Err(JsonParsingError::UnexpectedEOF);
    };

    match t.kind {
        TokenKind::String(s) => Ok(s),
        TokenKind::Eof => Err(JsonParsingError::UnexpectedEOF),
        _ => Err(JsonParsingError::InvalidJsonObject),
    }
}

fn consume_kind(tokens: &mut VecDeque<Token>, expected: TokenKind) -> Result<(), JsonParsingError> {
    let Some(t) = next_token(tokens) else {
        return Err(JsonParsingError::UnexpectedEOF);
    };

    if t.kind == expected {
        Ok(())
    } else {
        Err(JsonParsingError::InvalidJsonObject)
    }
}


