use super::*;

fn char_from_hex(s: &str) -> Option<char> {
    let hex = s.trim_start_matches("0x");
    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
}

fn assert_ok_eq(input: &str, expected: JsonValue) {
    match process_json_string(input) {
        Ok(value) => assert_eq!(value, expected, "input was: {:?}", input),
        Err(err) => panic!("expected Ok, got Err: {:?} for input: {:?}", err, input),
    }
}

fn assert_err_eq(input: &str, expected: JsonParsingError) {
    match process_json_string(input) {
        Ok(value) => panic!("expected Err {:?}, got Ok: {:?} for input: {:?}", expected, value, input),
        Err(err) => assert_eq!(err, expected, "input was: {:?}, got: {:?}", input, err),
    }
}

#[test]
fn test_process_empty_json_file() {
    let json_string = "";
    assert_err_eq(json_string, JsonParsingError::EmptyJsonFile);
}

#[test]
fn test_parse_null() {
    let cases = [
        "null",
        "   null",
        "null   ",
        "\r  null",
        "\n  \r  \t null  \n \r\t",
    ];

    for case in cases {
        assert_ok_eq(case, JsonValue::Null);
    }
}

#[test]
fn parse_bool_values() {
    let mut json_string = "true";
    assert_ok_eq(json_string, JsonValue::Boolean(true));

    json_string = "false";
    assert_ok_eq(json_string, JsonValue::Boolean(false));
}

#[test]
fn parse_typos_bool_and_null() {
    let cases = ["nullx", "truex", "falsex", "fals", "tru", "nul"];

    for case in cases {
        assert_err_eq(case, JsonParsingError::InvalidJsonFile);
    }
}

#[test]
fn parse_numbers_ok() {
    let cases = [
        // integers
        ("-1", JsonValue::Number(-1.0)),
        ("1", JsonValue::Number(1.0)),
        ("0", JsonValue::Number(0.0)),
        ("124453", JsonValue::Number(124453.0)),
        ("-0", JsonValue::Number(0.0)),
        ("   42 \n", JsonValue::Number(42.0)),
        ("   4 \n   ", JsonValue::Number(4.0)),
        ("0.00000e+00000", JsonValue::Number(0.0)),
        // fractions
        ("1.0", JsonValue::Number(1.0)),
        ("0.435", JsonValue::Number(0.435)),
        ("-12.5", JsonValue::Number(-12.5)),
        // exponent
        ("1e3", JsonValue::Number(1000.0)),
        ("1E3", JsonValue::Number(1000.0)),
        ("1e-3", JsonValue::Number(0.001)),
        ("1e+3", JsonValue::Number(1000.0)),
        ("1.2e3", JsonValue::Number(1200.0)),
        ("-1.2E3", JsonValue::Number(-1200.0)),
        ("-1.2e00000003", JsonValue::Number(-1200.0)),
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }
}

#[test]
fn parse_numbers_fail() {
    let cases = [
        "-",   // sign with no digits
        "-x",  // sign then junk
        "1x",  // junk after digits
        "--1", // malformed sign (second '-' not allowed here)
        "1.",  // a digit must follow a dot
        "1.e2", // a digit must follow a dot
        ".1",  // the number must start with a digit (after optional '-')
        "-.1", // same
        "1e",  // exponent must have digits
        "1E",  // exponent must have digits
        "1e+", // digits must follow the exponent sign
        "1e-", // digits must follow the exponent sign
        "1e+x", // junk after exponent sign
        "1ex", // junk after exponent marker
        "1..2", // second dot isn't allowed
        "1e2e3", // second exponent not allowed
        "1e2.3", // dot not allowed after exponent
        "+1",  // leading '+' not allowed by your parser (and JSON)
    ];

    for case in cases {
        assert_err_eq(case, JsonParsingError::InvalidJsonFile);
    }
}

#[test]
fn leading_zero_fail() {
    let cases = ["00", "00000"];

    for case in cases {
        assert_err_eq(case, JsonParsingError::LeadingZero);
    }
}

#[test]
fn simple_strinng() {
    let cases = [
        (r#""hello""#, JsonValue::JsonString("hello".to_string())),
        (r#""hé""#, JsonValue::JsonString("hé".to_string())),
        (r#""1033""#, JsonValue::JsonString("1033".to_string())),
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }
}

#[test]
fn escape_string() {
    let cases = [
        // quote
        (
            r#"" \" \" ""#,
            JsonValue::JsonString(r#" " " "#.to_string()),
        ),
        (
            r#"" \" jdfjgkh \" djkdsjfkfjg ""#,
            JsonValue::JsonString(r#" " jdfjgkh " djkdsjfkfjg "#.to_string()),
        ),
        // backslash
        (r#"" \\ ""#, JsonValue::JsonString(r#" \ "#.to_string())),
        (
            r#"" path\\to\\file ""#,
            JsonValue::JsonString(r#" path\to\file "#.to_string()),
        ),
        // slash
        (r#"" \/ ""#, JsonValue::JsonString(" / ".to_string())),
        // control escapes
        (
            r#"" \n \r \t ""#,
            JsonValue::JsonString(" \n \r \t ".to_string()),
        ),
        (
            r#"" \b \f ""#,
            JsonValue::JsonString(format!(" {} {} ", '\u{0008}', '\u{000C}')),
        ),
        // mixed
        (
            r#"" a\\b\"c\/d ""#,
            JsonValue::JsonString(r#" a\b"c/d "#.to_string()),
        ),
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }
}

#[test]
fn escape_string_fail() {
    let cases = [
        r#""\q""#,     // invalid escape
        r#""\""#,      // unterminated: ends right after an escape quote start
        r#""\\ \""#,   // unterminated: backslash escapes closing quote, no final quote
        "\"\n\"",      // raw newline inside string (not escaped)
        "\"\r\"",      // raw CR inside string (not escaped)
        "\"\t\"",  // raw tab inside string (not escaped) -- strict JSON forbids raw control chars
    ];

    for case in cases {
        assert_err_eq(case, JsonParsingError::InvalidJsonFile);
    }
}

#[test]
fn unicode_escape_ok() {
    let cases = [
        (r#""\u0041""#, JsonValue::JsonString("A".to_string())), // A
        (r#""\u0061""#, JsonValue::JsonString("a".to_string())), // a
        (r#""\u03B1""#, JsonValue::JsonString("α".to_string())), // Greek alpha
        (r#""x\u0041y""#, JsonValue::JsonString("xAy".to_string())), // mixed
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }
}

#[test]
fn unicode_escape_fail() {
    let cases = [
        r#""\u""#,        // no digits
        r#""\u0""#,       // too short
        r#""\u000""#,     // too short
        r#""\u12G4""#,    // non-hex
        r#""\uD800""#,    // surrogate range (high)
        r#""\uDFFF""#,    // surrogate range (low)
        r#""\uDE00""#,    // surrogate range (low)
    ];

    for case in cases {
        assert_err_eq(case, JsonParsingError::InvalidUnicodeInString);
    }
}

#[test]
fn unicode_escape_multiple_in_one_string_ok() {
    let cases = [
        // A B C
        (
            r#""\u0041\u0042\u0043""#,
            JsonValue::JsonString("ABC".to_string()),
        ),
        // x A y α z
        (
            r#""x\u0041y\u03B1z""#,
            JsonValue::JsonString("xAyαz".to_string()),
        ),
        // a-b-c with hyphens
        (
            r#""\u0061-\u0062-\u0063""#,
            JsonValue::JsonString("a-b-c".to_string()),
        ),
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }
}

#[test]
fn unicode_escape_multiple_in_one_string_fail() {
    let cases = [
        r#""\u0041\u12G4\u0043""#, // bad hex in the middle
        r#""\u0041\uD800\u0043""#, // surrogate in the middle (your v1 rule: error)
        r#""\u0041\u0042\u000""#,  // truncated at end
    ];

    for case in cases {
        let res = process_json_string(case);
        assert!(
            matches!(
                res,
                Err(JsonParsingError::InvalidUnicodeInString)
                    | Err(JsonParsingError::InvalidJsonFile)
            ),
            "input was: {:?}, got: {:?}",
            case,
            res
        );
    }
}
#[test]
fn simple_arrays() {
    let cases = [
        // single element
        (
            r#"[10]"#,
            JsonValue::Array(vec![JsonValue::Number(10.0)]),
        ),
        // multiple ints + whitespace
        (
            r#"[10, 15, 0, -10]"#,
            JsonValue::Array(vec![
                JsonValue::Number(10.0),
                JsonValue::Number(15.0),
                JsonValue::Number(0.0),
                JsonValue::Number(-10.0),
            ]),
        ),
        // nested empty arrays
        (
            r#"[[], []]"#,
            JsonValue::Array(vec![
                JsonValue::Array(vec![]),
                JsonValue::Array(vec![]),
            ]),
        ),
        // mixed types (assuming you already have strings/bools/null working)
        (
            r#"[null, true, false, "hé", "1033"]"#,
            JsonValue::Array(vec![
                JsonValue::Null,
                JsonValue::Boolean(true),
                JsonValue::Boolean(false),
                JsonValue::JsonString("hé".to_string()),
                JsonValue::JsonString("1033".to_string()),
            ]),
        ),
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }

}

#[test]
fn arrays_malformed_should_fail() {
    let cases = [
        r#"[,1]"#,        // leading comma
        r#"[1,,2]"#,      // double comma
        r#"[1,]"#,        // trailing comma
        r#"[1 2]"#,       // missing comma
        r#"[ , ]"#,       // comma with no values (after trimming)
        r#"[true false]"#, // missing comma between values
        r#"["#,           // unterminated
        r#"[1"#,          // unterminated after value
        r#"[1,"#,         // unterminated after comma
        r#"[1, 2"#,       // unterminated after second value
        r#"[1, ]"#,       // comma then whitespace then close
    ];

    for case in cases {
        let res = process_json_string(case);
        assert!(
            matches!(res, Err(JsonParsingError::InvalidArray) | Err(JsonParsingError::InvalidJsonFile)),
            "input was: {:?}, got: {:?}",
            case,
            res
        );
    }
}

#[test]
fn object_simple_ok() {
    use indexmap::IndexMap;

    // {}
    let expected_empty = JsonValue::Object(IndexMap::new());

    // {"a":1}
    let mut m1 = IndexMap::new();
    m1.insert("a".to_string(), JsonValue::Number(1.0));
    let expected_one = JsonValue::Object(m1);

    // {"a":1,"b":true,"c":null,"d":"hé","e":[1,2]}
    let mut m2 = IndexMap::new();
    m2.insert("a".to_string(), JsonValue::Number(1.0));
    m2.insert("b".to_string(), JsonValue::Boolean(true));
    m2.insert("c".to_string(), JsonValue::Null);
    m2.insert("d".to_string(), JsonValue::JsonString("hé".to_string()));
    m2.insert(
        "e".to_string(),
        JsonValue::Array({
            let mut v = Vec::new();
            v.push(JsonValue::Number(1.0));
            v.push(JsonValue::Number(2.0));
            v
        }),
    );
    let expected_mixed = JsonValue::Object(m2);

    let cases = [
        (r#"{}"#, expected_empty),
        (r#"{ "a" : 1 }"#, expected_one),
        (r#"{ "a":1, "b":true, "c":null, "d":"hé", "e":[1,2] }"#, expected_mixed),
    ];

    for (case, expected) in cases {
        assert_ok_eq(case, expected);
    }
}

#[test]
fn object_degenerate_should_fail() {
    let cases = [
        r#"{"#,                 // unterminated
        r#"}"#,                 // stray close
        r#"{{}}"#,              // extra open
        r#"{,}"#,               // leading comma
        r#"{,"a":1}"#,          // leading comma (alt)
        r#"{"a":1,}"#,          // trailing comma
        r#"{"a":}"#,            // missing value
        r#"{"a" 1}"#,           // missing colon
        r#"{"a":1 "b":2}"#,     // missing comma between pairs
        r#"{"a":1,, "b":2}"#,   // double comma
        r#"{a:1}"#,             // key not quoted
        r#"{"a":1, b:2}"#,      // key not quoted (second)
        r#"{"a": [1,]}"#,       // invalid array value (trailing comma)
        r#"{"a": "x"#,          // unterminated string
        r#"{"a":1"#,            // missing closing brace
        r#"{"a":1, "b":2"#,     // missing closing brace
        r#"{"a":1:2}"#,         // extra colon
        r#"{"a""b":1}"#,        // missing colon between key and key
    ];

    for case in cases {
        let res = process_json_string(case);
        assert!(
            matches!(res, Err(JsonParsingError::InvalidJsonFile) | Err(JsonParsingError::InvalidJsonObject) | Err(JsonParsingError::InvalidArray)),
            "input was: {:?}, got: {:?}",
            case,
            res
        );
    }
}
