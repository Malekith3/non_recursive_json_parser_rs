use std::collections::VecDeque;
use indexmap::IndexMap;
use crate::json_definitions::{JsonParsingErrorV3, JsonValue};
use crate::json_definitions::JsonParsingErrorV2::UnexpectedToken;
use crate::json_lexer::{lex_all, Token, TokenKind};

// =============================================================================
// JSON Parser v3 — Stack-Based (Non-Recursive)
// =============================================================================
//
// CONCEPT:
//   v2 used recursion — the CPU call stack implicitly stored unfinished work
//   (partial arrays, partial objects) while parsing nested structures.
//
//   v3 replaces that implicit call stack with an explicit Vec<JsonFrame> that
//   we manage ourselves. Same computation, same logic, you hold the bookkeeping.
//
// HIGH LEVEL FLOW:
//   input string
//     ↓
//   lexer → VecDeque<Token>          (reuse from v2, no changes)
//     ↓
//   main loop over tokens
//     ↓
//   explicit stack of JsonFrame
//     ↓
//   JsonValue (the final tree)
//
// THE MAIN LOOP — 3 categories of token:
//   1. Leaf value (Null, Bool, String, Number)
//      → produce JsonValue immediately, call attach_value()
//
//   2. Open structure ({ or [)
//      → push a new frame onto the stack
//
//   3. Close structure (} or ])
//      → pop top frame, build completed JsonValue, call attach_value()
//
//   4. Separators (: and ,)
//      → validate structure that you have value in array in frame aray or in jsonvvalue for ,
//      -> for : you need to validate that key string was consumed in object array or in json value of type object
//
//   5. EOF
//      → validate stack is empty, return result
//
// KEY INSIGHT — attach_value():
//   When a JsonValue is completed (leaf token or just-popped frame), it needs
//   to go somewhere. "Somewhere" depends on what's on top of the stack:
//     - Stack empty        → this is the root value, store in `result`
//     - Top is ArrayFrame  → append to frame.items
//     - Top is ObjectFrame → depends on pending_key state (see below)
//
// KEY INSIGHT — pending_key: Option<String>:
//   In an object, a String token plays double duty:
//     - pending_key is None    → string is a KEY, store it as pending_key
//     - pending_key is Some(k) → string is a VALUE, insert (k, value) into items
//   The frame state does the disambiguation — not the token itself.
//
// =============================================================================

enum JsonFrame {
    Object(ObjectFrame),
    Array(ArrayFrame),
}

struct ObjectFrame {
    pending_key: Option<String>,
    items: IndexMap<String, JsonValue>,
}

impl ObjectFrame {
    fn new() -> Self {
        Self {
            pending_key: None,
            items: IndexMap::new(),
        }
    }
}

struct ArrayFrame {
    items: Vec<JsonValue>,
}

impl ArrayFrame {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
}

pub fn process_json_string_v3(json_string: &str) -> Result<JsonValue, JsonParsingErrorV3> {

    if json_string.is_empty() {
        return Result::Err(JsonParsingErrorV3::EmptyJsonFile)
    }

    let mut tokens =  lex_all(json_string.as_bytes())
        .map_err(|e|JsonParsingErrorV3::LexError(e))?;

    let mut stack: Vec<JsonFrame> = Vec::new();
    let mut result: Option<JsonValue> = None;
    let mut cursor = tokens.front().map_or(0, |token| token.span.start);

    parse_loop(&mut tokens, &mut cursor, &mut stack, &mut result)?;

    result.ok_or(JsonParsingErrorV3::UnexpectedEOF)
}

fn parse_loop(
    tokens: &mut VecDeque<Token>,
    cursor: &mut usize,
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
) -> Result<(), JsonParsingErrorV3> {
    loop {
        let Some(token) = next_token(tokens, cursor) else {
            return Err(JsonParsingErrorV3::UnexpectedEOF)
        };

        match token.kind {
            TokenKind::Null => attach_value(stack, result, JsonValue::Null)?,
            TokenKind::Bool(b) => attach_value(stack, result, JsonValue::Boolean(b))?,
            TokenKind::String(s) => attach_value(stack, result, JsonValue::JsonString(s))?,
            TokenKind::Number(n) => attach_value(stack, result, JsonValue::Number(n))?,
            TokenKind::LBracket => stack.push(JsonFrame::Array(ArrayFrame::new())),
            TokenKind::LBrace => stack.push(JsonFrame::Object(ObjectFrame::new())),
            TokenKind::RBracket => close_array(stack, result, cursor)?,
            TokenKind::RBrace => close_object(stack, result, cursor)?,
            TokenKind::Colon => validate_colon(stack, cursor)?,
            TokenKind::Comma => validate_comma(stack, cursor)?,
            TokenKind::Eof => {
                validate_end(stack, result, cursor)?;
                return Ok(());
            }
        }
    }
}

fn attach_value(
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
    value: JsonValue,
) -> Result<(), JsonParsingErrorV3> {
    let Some(frame) = stack.last_mut() else {
        return if result.is_none() {
            result.replace(value);
            Ok(())
        } else {
            Err(JsonParsingErrorV3::ExpectedEOF)
        };
    };

    match frame {
        JsonFrame::Array(a) => {
            a.items.push(value);
            Ok(())
        }
        JsonFrame::Object(o) => match o.pending_key.take() {
            None => match value {
                JsonValue::JsonString(s) => {
                    o.pending_key = Some(s);
                    Ok(())
                }
                _ => Err(JsonParsingErrorV3::ObjectKeyNotString),
            },
            Some(key) => {
                o.items.insert(key, value);
                Ok(())
            }
        },
    }
}

fn close_array(
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    let Some(frame) = stack.pop() else {
        return Err(JsonParsingErrorV3::UnexpectedClosing)
    };

    match frame {
        JsonFrame::Array(a) => Ok(attach_value(stack, result, JsonValue::Array(a.items))?),
        _ =>  Err(JsonParsingErrorV3::UneExpectedFrameType)
    }
}

fn close_object(
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    let Some(frame) = stack.pop() else {
        return Err(JsonParsingErrorV3::UnexpectedClosing)
    };

    match frame {
        JsonFrame::Object(o) => {
            if o.pending_key.is_some() {
                return Err(JsonParsingErrorV3::ObjectKeyWithoutValue)
            }

            Ok(attach_value(stack, result, JsonValue::Object(o.items))?)
        },
        _ =>  Err(JsonParsingErrorV3::UneExpectedFrameType)
    }
}

fn validate_end(
    stack: &Vec<JsonFrame>,
    result: &Option<JsonValue>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    if !stack.is_empty() || result.is_none(){
        return Err(JsonParsingErrorV3::UnexpectedEOF)
    }
    return Ok(());
}

fn validate_colon(
    stack: &mut Vec<JsonFrame>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    let Some(frame) = stack.last() else {
        // TODO: Need create seprate error probably
        return Err(JsonParsingErrorV3::UnexpectedClosing)
    };

    match frame {
        JsonFrame::Object(o) => {
            if o.pending_key.is_none() {
                // TODO: Probaly too new error type
                return Err(JsonParsingErrorV3::ObjectKeyNotString)
            }
        },
        _ => return Err(JsonParsingErrorV3::UneExpectedFrameType)
    }

    return Ok(());
}

fn validate_comma(
    stack: &mut Vec<JsonFrame>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    let Some(frame) = stack.last() else {
        // TODO: Need separate error — comma outside any structure
        return Err(JsonParsingErrorV3::UnexpectedClosing)
    };

    match frame {
        JsonFrame::Object(o) => {
            // comma before any key-value pair: {, "a": 1}
            if o.items.is_empty() && o.pending_key.is_none() {
                return Err(JsonParsingErrorV3::MismatchedClosing)
            }
            // comma while waiting for value: {"a":, "b": 1}
            if o.pending_key.is_some() {
                return Err(JsonParsingErrorV3::MismatchedClosing)
            }
        },
        JsonFrame::Array(a) => {
            if a.items.is_empty() {
                // TODO: New error for comma needed
                return Err(JsonParsingErrorV3::MismatchedClosing)
            }
        }
    }
    return Ok(())
}


fn peek_token(tokens: &VecDeque<Token>) -> Option<&Token> {
    tokens.front()
}

fn next_token(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Option<Token> {
    let token = tokens.pop_front()?;
    *cursor = token.span.end;
    Some(token)
}
