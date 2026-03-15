use std::collections::VecDeque;
use indexmap::IndexMap;
use crate::json_definitions::{JsonParsingErrorV2, JsonValue, token_tag_of, JsonFrame, JsonParsingErrorV3};
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
//      → consume and continue, no action needed
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

/// Parse a JSON string using an explicit stack instead of recursion.
/// Drop-in replacement for process_json_string_v2.
pub fn process_json_string_v3(json_string: &str) -> Result<JsonValue, JsonParsingErrorV3> {
    // Step 1: validate input (same as v2)
    // TODO: return EmptyJsonFile if empty

    // Step 2: lex (same as v2)
    // TODO: let mut tokens = lex_all(...)

    // Step 3: set up explicit stack and result slot
    // TODO: let mut stack: Vec<JsonFrame> = Vec::new();
    // TODO: let mut result: Option<JsonValue> = None;

    // Step 4: main loop — see parse_loop() below
    // TODO: parse_loop(&mut tokens, &mut cursor, &mut stack, &mut result)?;

    // Step 5: return result
    // TODO: result.ok_or(UnexpectedEOF) or however you want to handle no value
    todo!()
}

// =============================================================================
// MAIN PARSING LOOP
// =============================================================================

/// The core of v3. Iterates over tokens and drives the stack machine.
/// Replaces all recursive parse_* functions from v2.
fn parse_loop(
    tokens: &mut VecDeque<Token>,
    cursor: &mut usize,
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
) -> Result<(), JsonParsingErrorV3> {
    loop {
        // TODO: let token = next_token(tokens, cursor) — same helper as v2

        // TODO: match token.kind {

        // --- Leaf values ---
        // Null    → attach_value(stack, result, JsonValue::Null)?
        // Bool(b) → attach_value(stack, result, JsonValue::Boolean(b))?
        // String  → attach_value(stack, result, JsonValue::JsonString(s))?
        // Number  → attach_value(stack, result, JsonValue::Number(n))?
        //
        // NOTE: String is special for objects — attach_value handles the
        // pending_key logic, so no special casing needed here.

        // --- Open structures: push frame ---
        // LBracket → push ArrayFrame { items: vec![] }
        // LBrace   → push ObjectFrame { pending_key: None, items: IndexMap::new() }

        // --- Close structures: pop frame, attach completed value ---
        // RBracket → close_array(stack, result, cursor)?
        // RBrace   → close_object(stack, result, cursor)?

        // --- Separators: consume and continue ---
        // Colon → continue  (no action needed)
        // Comma → continue  (no action needed)

        // --- Done ---
        // Eof → validate_end(stack, result, cursor)?; break;

        // --- Anything else is unexpected ---
        // _ → return Err(UnexpectedToken)
        // }
    }
    todo!()
}

// =============================================================================
// ATTACH VALUE
// =============================================================================

/// Called whenever a JsonValue is completed — either a leaf token or a
/// just-popped frame. Attaches the value to the correct place.
///
/// Decision tree:
///   Stack empty              → store in result (this is the root value)
///   Top = ArrayFrame         → push to frame.items
///   Top = ObjectFrame
///     pending_key = None
///       value is JsonString  → set as pending_key (it's a key)
///       value is not String  → error: object keys must be strings
///     pending_key = Some(k)  → insert (k, value) into items, clear pending_key
fn attach_value(
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
    value: JsonValue,
) -> Result<(), JsonParsingErrorV3> {
    // TODO: match stack.last_mut() { ... }
    todo!()
}

// =============================================================================
// CLOSE ARRAY
// =============================================================================

/// Called when ] is encountered.
/// Pops the top frame (must be ArrayFrame), builds JsonValue::Array,
/// then calls attach_value to send it to the parent frame.
fn close_array(
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    // TODO: pop top frame
    // TODO: verify it is ArrayFrame (not ObjectFrame — that's a mismatch error)
    // TODO: build JsonValue::Array(frame.items)
    // TODO: attach_value(stack, result, completed_array)?
    todo!()
}

// =============================================================================
// CLOSE OBJECT
// =============================================================================

/// Called when } is encountered.
/// Pops the top frame (must be ObjectFrame), builds JsonValue::Object,
/// then calls attach_value to send it to the parent frame.
fn close_object(
    stack: &mut Vec<JsonFrame>,
    result: &mut Option<JsonValue>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    // TODO: pop top frame
    // TODO: verify it is ObjectFrame (not ArrayFrame — that's a mismatch error)
    // TODO: verify pending_key is None (Some means key without value — error)
    // TODO: build JsonValue::Object(frame.items)
    // TODO: attach_value(stack, result, completed_object)?
    todo!()
}

// =============================================================================
// VALIDATE END
// =============================================================================

/// Called when EOF is encountered.
/// At this point the stack must be empty and result must be Some.
fn validate_end(
    stack: &Vec<JsonFrame>,
    result: &Option<JsonValue>,
    cursor: &usize,
) -> Result<(), JsonParsingErrorV3> {
    // TODO: if stack is not empty → UnexpectedEOF (unclosed structure)
    // TODO: if result is None     → UnexpectedEOF (empty input, caught earlier but guard anyway)
    // TODO: otherwise Ok(())
    todo!()
}

// =============================================================================
// TOKEN HELPERS (same as v2, copy or reuse)
// =============================================================================

fn peek_token(tokens: &VecDeque<Token>) -> Option<&Token> {
    tokens.front()
}

fn next_token(tokens: &mut VecDeque<Token>, cursor: &mut usize) -> Option<Token> {
    let token = tokens.pop_front()?;
    *cursor = token.span.end;
    Some(token)
}