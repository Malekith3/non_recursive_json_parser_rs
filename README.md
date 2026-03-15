# JSON Parser — Learning Project

This is a personal learning project. The goal was to take a simple,
well-defined format, read the [JSON specification](https://www.json.org/json-en.html)
directly, and implement a parser for it from scratch — progressively in three
versions, each adding more structure. JSON was chosen because it is popular
enough to be familiar but small enough to implement without getting lost in
complexity. Rust was chosen as an additional opportunity to learn the language
alongside the parsing concepts.

> This is not a production parser. Some edge cases are deliberately out of scope.
 
---

## Aim

The goal of this project was to answer a few concrete questions by doing:

- How difficult is it to implement a parser from scratch?
- What does lexing actually involve and where does the complexity hide?
- What should you expect when parsing strings in particular?
- How do you model and propagate errors in a parser?
- How does a memory-safe language like Rust differ from C/C++ in practice?

---

## What's in the repo

```
src/          — all parser implementations and shared definitions
tests/        — unit tests for each version (lexer, v1, v2, v3)
examples/     — CLI applications to run each parser against a JSON file
assets/       — sample JSON files for manual testing
```

### Source structure

| File | Description |
|------|-------------|
| `json_lexer.rs` | Tokenizer — converts raw bytes into a token stream |
| `json_definitions.rs` | Shared types — `JsonValue`, error types, token tags |
| `json_parsing_naive.rs` | v1 — naive recursive parser, no lexer |
| `json_lexer_parser.rs` | v2 — lexer + recursive descent parser |
| `json_non_recursive.rs` | v3 — lexer + stack-based non-recursive parser (WIP) |
 
---

## Learning Progression

### v1 — Naive Recursive Parser (`json_parsing_naive.rs`)

The first version was a direct read-and-parse implementation with no lexer —
cursor manipulation, whitespace trimming, and structural parsing all mixed
together. The goal was to understand the raw difficulty of parsing a string
by hand and see where the pain points are.

The main lesson: manual cursor management is error-prone. Forgetting to
advance the cursor, forgetting to trim whitespace, forgetting to validate
a character — all of these are a class of bugs that have to be handled
everywhere, scattered across the entire parser.

### v2 — Lexer + Recursive Descent (`json_lexer_parser.rs`)

The second version introduced a lexer to separate tokenisation from parsing.
The goal was to understand why lexing exists in practice, not just in theory.

The result was immediate: all the manual cursor bugs from v1 were centralised
in one place. The parser became significantly simpler — it operates on a clean
token stream and never thinks about bytes, whitespace, or cursor advancement.
Most of the complexity in a JSON parser lives in the lexer, not the parser.

### v3 — Stack-Based Non-Recursive Parser (`json_non_recursive.rs`) — WIP

The third version eliminates recursion by replacing the call stack with an
explicit `Vec<Frame>`. The goals were:

- Eliminate a whole class of stack-overflow bugs caused by deeply nested input
- Remove the performance cost of recursive calls emulating something the
  hardware does not have natively
- Simplify debugging — no need to unwind a huge call stack to understand
  parser state

---

## API

The parser exposes one entry point per version:

```rust
// v1 — naive recursive
pub fn process_json_string_v1(json_string: &str) -> Result<JsonValue, JsonParsingError>
 
// v2 — lexer + recursive descent
pub fn process_json_string_v2(json_string: &str) -> Result<JsonValue, JsonParsingErrorV2>
 
// v3 — stack-based non-recursive (WIP)
pub fn process_json_string_v3(json_string: &str) -> Result<JsonValue, JsonParsingErrorV3>
```

All versions produce the same output type:

```rust
pub enum JsonValue {
    Object(IndexMap<String, JsonValue>),  // insertion order preserved
    Array(Vec<JsonValue>),
    JsonString(String),
    Number(f64),
    Boolean(bool),
    Null,
}
```

There is no serialisation — the parser produces a `JsonValue` tree only.
See the `examples/` directory for CLI usage.

### Dependencies

One external dependency: [`indexmap`](https://crates.io/crates/indexmap) —
used for `Object` to preserve insertion order. Writing a custom ordered map
was out of scope for this project. See `Cargo.toml` for the exact version.
 
---

## Limitations

These are deliberate gaps — areas that were out of scope for the learning
goals of this project and not worth the investment to close.

- **UTF-8 surrogate pairs** — surrogate pairs in `\uXXXX` escape sequences
  are detected and rejected with an explicit error. Full surrogate pair
  handling (e.g. `\uD800\uDC00`) is not implemented. Note: rejecting
  surrogates is actually correct per the JSON spec — this is a known
  scope decision, not a bug.

- **Number precision** — all numbers are represented as `f64`. The maximum
  safe integer is 2^53 (same limitation as JavaScript). Large integers
  beyond this range will lose precision silently.

- **Non-finite numbers** — `+Infinity`, `-Infinity` and `NaN` are rejected.
  JSON does not include them in the spec, but some parsers accept them as
  an extension. This one does not.

- **No serialisation** — the parser produces a `JsonValue` tree but does
  not convert back to a JSON string. Serialisation was not relevant to the
  parsing lessons this project was built around.

- **Performance** — performance was never a concern for this project.
  The goal was to understand parsing concepts, not to optimise them.
  No benchmarking or profiling was done.

---

## Planned Experiments

- **Profiling** — once v3 is complete, profile all three versions against
  each other to measure the real cost of recursion vs explicit stack,
  and where time is actually spent in the parsing pipeline.

---

## Conclusions

### How difficult is it to implement a parser from scratch?

Laborious — and humbling. The code itself is not the hard part.
Error handling, testing, and chasing edge cases is where the time goes.
JSON is about as simple as grammars get, and it still demanded significant
effort to get right. Pick a more complex grammar and this compounds fast.

### What does lexing actually involve and where does the complexity hide?

Lexing is not magic — it is just operating on bytes with discipline.
The complexity does not disappear, it centralises. Strings are where
most of it lives: UTF-8, escape sequences, unicode codepoints, control
characters — all of it lands in the lexer and none of it is fun.

The non-obvious lesson was the design decision hiding inside lexing:
how much parsing work belongs in the lexer vs the parser? There is no
universal answer. Fully decoding numbers and strings in the lexer turned
out to be the right call here — it left the parser with a clean structural
problem and nothing else.

### What should you expect when parsing strings in particular?

A rabbit hole with no visible bottom. Strings look trivial until you
start pulling on the thread: what is a string, exactly? The answer
depends on the encoding, the language, and what era the standard was
written in. UTF-8, UTF-32, surrogate pairs, codepoints vs code units —
and none of this is theoretical once you support any language beyond English.
Budget two to three times more time than you think strings will take.

### How do you model and propagate errors in a parser?

`Result<>` based propagation is the answer, and it is a significant
quality of life improvement over alternatives. The key insight is that
errors need *context* — position, what was found, what was expected —
and that context should be captured at the source, not reconstructed later.
Separate error types per layer (lexer errors stay in the lexer, parser
errors stay in the parser) keeps this clean.

This pattern is not Rust-specific. The same approach in C++ via
[Outcome](https://ned14.github.io/outcome/) feels immediately superior
to error codes or exceptions for this kind of work.

### How does a memory-safe language like Rust differ from C/C++ in practice?

C++ with guardrails is the most honest description. The mental model is
identical — ownership, lifetimes, memory layout all require the same
thinking. Rust just enforces what C++ leaves to discipline and convention.
For a project like this the borrow checker was never a serious obstacle,
just an occasional reminder to be explicit about something that should
have been explicit anyway.
 