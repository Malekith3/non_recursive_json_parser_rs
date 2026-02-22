use std::collections::VecDeque;
use indexmap::IndexMap;
use crate::json_definitions::{JsonParsingErrorV2, JsonValue, token_tag_of};
use crate::json_lexer::{lex_all, Token, TokenKind};

pub fn process_json_string_v2(json_string: &str) -> Result<JsonValue, JsonParsingErrorV2>{

    if json_string.is_empty() {
        return Err(JsonParsingErrorV2::EmptyJsonFile);
    }

    let mut tokens = lex_all(json_string.as_bytes()).map_err(|e|JsonParsingErrorV2::LexError(e))?;
    let mut cursor = tokens.front().map_or(0, |token| token.span.start);
    let json_value = parse_json_value(&mut tokens, &mut cursor)?;

    let Some(token) = next_token(&mut tokens, &mut cursor) else {
        return Err(JsonParsingErrorV2::UnexpectedEOF{at: Some(cursor) })
    };

    if token.kind != TokenKind::Eof {
        return Err(JsonParsingErrorV2::ExpectedEOF { found: token_tag_of(&token.kind), at: Some(token.span.start) })
    }

    Ok(json_value)
}

fn peek_token(tokens: &VecDeque<Token>) -> Option<&Token> {
    tokens.front()
}

fn next_token(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Option<Token>{
    let token = tokens.pop_front()?;
    *cursor = token.span.end;
    Some(token)
}

fn parse_json_value(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Result<JsonValue, JsonParsingErrorV2> {

    let Some(token) = next_token(tokens, cursor) else {
        return Err(JsonParsingErrorV2::UnexpectedEOF {at: Some(*cursor) })
    };

    return match token.kind {
        TokenKind::Null => Ok(JsonValue::Null),
        TokenKind::Bool(b) => Ok(JsonValue::Boolean(b)),
        TokenKind::String(s) => Ok(JsonValue::JsonString(s)),
        TokenKind::Number(n) => Ok(JsonValue::Number(n)),
        TokenKind::LBrace => parse_json_object(tokens, cursor),
        TokenKind::LBracket => parse_json_array(tokens, cursor),
        TokenKind::Eof => Err(JsonParsingErrorV2::UnexpectedEOF{at:Some(*cursor)}),
        _ => Err(JsonParsingErrorV2::UnexpectedToken { found: token_tag_of(&token.kind), at: Some(token.span.start) }),
    }
}

fn parse_json_array(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Result<JsonValue, JsonParsingErrorV2> {
    let Some(t0) = peek_token(tokens) else {
        return Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) });
    };

    if t0.kind == TokenKind::RBracket {
        let _ = next_token(tokens, cursor);
        return Ok(JsonValue::Array(Vec::new()));
    }

    let mut out = Vec::new();
    out.push(parse_json_value(tokens, cursor)?);

    loop {
        let Some(t) = peek_token(tokens) else {
            return Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) });
        };

        match t.kind {
            TokenKind::Comma => {
                let _ = next_token(tokens, cursor);
                out.push(parse_json_value(tokens, cursor)?);
            }
            TokenKind::RBracket => {
                let _ = next_token(tokens, cursor);
                return Ok(JsonValue::Array(out));
            }
            _ => {
                return Err(JsonParsingErrorV2::InvalidArray { found: token_tag_of(&t.kind), at: Some(t.span.start) });
            }
        }
    }
}


fn parse_json_object(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Result<JsonValue, JsonParsingErrorV2> {
    // We enter here after '{' has already been consumed.

    // empty object: "{}"
    let Some(t0) = peek_token(tokens) else {
        return Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) });
    };

    if t0.kind == TokenKind::RBrace {
        let _ = next_token(tokens, cursor); // consume '}'
        return Ok(JsonValue::Object(IndexMap::new()));
    }

    let mut out: IndexMap<String, JsonValue> = IndexMap::new();

    // First pair: key ":" value
    let key = parse_object_key(tokens, cursor)?;
    consume_kind(tokens, TokenKind::Colon, cursor)?;
    let value = parse_json_value(tokens, cursor)?;
    out.insert(key, value);

    loop {
        let Some(t) = peek_token(tokens) else {
            return Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) });
        };

        match &t.kind {
            TokenKind::Comma => {
                let _ = next_token(tokens, cursor); // consume ','

                let key = parse_object_key(tokens, cursor)?;
                consume_kind(tokens, TokenKind::Colon, cursor)?;
                let value = parse_json_value(tokens, cursor)?;
                out.insert(key, value);
            }

            TokenKind::RBrace => {
                let _ = next_token(tokens, cursor); // consume '}'
                return Ok(JsonValue::Object(out));
            }

            _ => {
                return Err(JsonParsingErrorV2::InvalidJsonObject { found: token_tag_of(&t.kind), at: Some(t.span.start) });
            }
        }
    }
}

fn parse_object_key(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Result<String, JsonParsingErrorV2> {
    let Some(t) = next_token(tokens, cursor) else {
        return Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) });
    };

    match t.kind {
        TokenKind::String(s) => Ok(s),
        TokenKind::Eof => Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) }),
        _ => Err(JsonParsingErrorV2::InvalidJsonObject { found: token_tag_of(&t.kind), at: Some(t.span.start) }),
    }
}

fn consume_kind(tokens: &mut VecDeque<Token>, expected: TokenKind, cursor: &mut usize) -> Result<(), JsonParsingErrorV2> {
    let Some(t) = next_token(tokens, cursor) else {
        return Err(JsonParsingErrorV2::UnexpectedEOF { at: Some(*cursor) });
    };

    if t.kind == expected {
        Ok(())
    } else {
        Err(JsonParsingErrorV2::InvalidJsonObject { found: token_tag_of(&t.kind), at: Some(t.span.start) })
    }
}
