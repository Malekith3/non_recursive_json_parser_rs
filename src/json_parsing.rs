use std::string::String;
use std::collections::HashMap;
use std::env::current_exe;
use std::fs::read_to_string;
use std::str::from_utf8;

pub(crate) fn process_json_file(json_string: &str) -> Result<JsonValue, JsonParsingError> {
    if json_string.is_empty() {
        return Err(JsonParsingError::EmptyJsonFile);
    }

    let mut json_index: usize = 0;

    trim_spaces(json_string, &mut json_index);

    if json_index >= json_string.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let result = parse_json_value(json_string, &mut json_index);

    trim_spaces(json_string, &mut json_index);

    result
}
fn parse_json_value(
    json_string: &str,
    current_index: &mut usize,
) -> Result<JsonValue, JsonParsingError> {
    trim_spaces(json_string, current_index);

    if *current_index >= json_string.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }
    let json_string_bytes = json_string.as_bytes();
    match json_string_bytes[*current_index] {
        b'n' => parse_null(json_string, current_index),
        b't' | b'f' => parse_bool(json_string, current_index),
        b'-' | b'0'..= b'9' => parse_number(json_string, current_index),
        b'\"' => parse_string(json_string, current_index),
        _ => Err(JsonParsingError::InvalidJsonFile),
    }

}


fn consume_escape_char(bytes_stream: &[u8], current_index: &mut usize) -> Result<([u8; 4], usize),JsonParsingError>{
    //check if we have scape char even if it double the work
    if *current_index >= bytes_stream.len() || bytes_stream[*current_index] != b'\\' {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    //consume escape char
    *current_index+=1;

    // Need one more byte for the escape kind
    if *current_index >= bytes_stream.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let mut out = [0u8; 4];

    let n = match bytes_stream[*current_index] {
        b'"'  => { out[0] = b'"'; 1 }
        b'\\' => { out[0] = b'\\'; 1 }
        b'/'  => { out[0] = b'/'; 1 }
        b'b'  => { out[0] = 0x08; 1 }
        b'f'  => { out[0] = 0x0C; 1 }
        b'n'  => { out[0] = b'\n'; 1 }
        b'r'  => { out[0] = b'\r'; 1 }
        b't'  => { out[0] = b'\t'; 1 }
        // TODO need to add handling unicodes
        _ => return Err(JsonParsingError::InvalidJsonFile),
    };

    *current_index += 1;
    Ok((out, n))

}

fn parse_string(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError>{

    let bytes = json_string.as_bytes();

    // Must start on opening quote
    if *current_index >= bytes.len() || bytes[*current_index] != b'"' {
        return Err(JsonParsingError::InvalidJsonFile);
    }
    *current_index += 1;

    let mut out: Vec<u8> = Vec::new();
    let mut run_start = *current_index;
    let mut closed = false;


    while *current_index < bytes.len() {
        match bytes[*current_index] {
            b'\\' => {
                out.extend_from_slice(&bytes[run_start..*current_index]);

                let (buf, n) = consume_escape_char(bytes, current_index)?;
                out.extend_from_slice(&buf[..n]);

                run_start = *current_index;
            }

            b'"' => {
                out.extend_from_slice(&bytes[run_start..*current_index]);

                *current_index += 1;
                closed = true;
                break;
            }

            // JSON: forbid raw control chars inside strings
            0x00..=0x1F => return Err(JsonParsingError::InvalidJsonFile),
            _ => {
                *current_index += 1;
            }
        }
    }

    if !closed {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    if !ensure_delimiter_or_eof(bytes, *current_index) {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let s = from_utf8(&out).map_err(|_| JsonParsingError::InvalidJsonFile)?.to_owned();
    Ok(JsonValue::JsonString(s))
}

fn parse_number(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError>{
    let bytes = json_string.as_bytes();
    let negative = if bytes[*current_index] == b'-' { *current_index+=1; -1.0 } else { 1.0 };

    if *current_index >= bytes.len() || !bytes[*current_index].is_ascii_digit() {
        return Err(JsonParsingError::InvalidJsonFile)
    }

    if bytes[*current_index] == b'0' {
        let next = *current_index + 1;
        if next < bytes.len() && bytes[next].is_ascii_digit() {
            return Err(JsonParsingError::LeadingZero)
        }
    }

    let start_index = *current_index;

    let mut seen_dot = false;
    let mut seen_exp = false;

    let mut need_frac_digit = false;
    let mut need_exp_digit = false;

    let mut exp_sign_allowed = false;

    while *current_index < bytes.len() {
        let b = bytes[*current_index];

        match b {
            b'0'..=b'9' => {
                if need_frac_digit {
                    need_frac_digit = false;
                }
                if need_exp_digit {
                    need_exp_digit = false;
                }
                exp_sign_allowed = false;
                *current_index += 1;
            }

            b'.' => {
                if seen_dot || seen_exp {
                    return Err(JsonParsingError::InvalidJsonFile);
                }
                seen_dot = true;
                need_frac_digit = true;
                exp_sign_allowed = false;
                *current_index += 1;
            }

            b'e' | b'E' => {
                if seen_exp || need_frac_digit {
                    return Err(JsonParsingError::InvalidJsonFile);
                }
                seen_exp = true;
                need_exp_digit = true;     // must see digits in exponent
                exp_sign_allowed = true;   // may accept +/-
                *current_index += 1;
            }

            b'+' | b'-' => {
                // '+' only valid in exponent; '-' here only valid if it's exponent sign (not leading; that's already handled)
                if !exp_sign_allowed {
                    return Err(JsonParsingError::InvalidJsonFile);
                }
                exp_sign_allowed = false;
                *current_index += 1;
            }

            _ => break, // stop scanning the number token; delimiter check comes next
        }
    }

    if(need_frac_digit || need_exp_digit){
        return Err(JsonParsingError::InvalidJsonFile)
    }

    if !ensure_delimiter_or_eof(bytes,*current_index) {
        return Err(JsonParsingError::InvalidJsonFile)
    }

    let string_number_rep = &json_string[start_index .. *current_index];
    let num: f64 = string_number_rep.parse().map_err(|_| JsonParsingError::InvalidJsonFile)?;


    Ok(JsonValue::Number(num * negative))
}

fn parse_null(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError> {
    const NULL_BYTES: &[u8] = b"null";
    let bytes = json_string.as_bytes();

    if *current_index + NULL_BYTES.len() > bytes.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    if &bytes[*current_index..*current_index + NULL_BYTES.len()] != NULL_BYTES {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let next = *current_index + NULL_BYTES.len();
    if next < bytes.len() && !is_delimiter(bytes[next]) {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    *current_index = next;

    Ok(JsonValue::Null)
}

fn is_delimiter(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b',' | b']' | b'}')
}
fn parse_bool(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError> {
    const TRUE_BYTES: &[u8] = b"true";
    const FALSE_BYTES: &[u8] = b"false";
    let bytes = json_string.as_bytes();

    if *current_index >= bytes.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    match bytes[*current_index] {
        b't' => {
            if !consume_literal(bytes, current_index, TRUE_BYTES) {
                return Err(JsonParsingError::InvalidJsonFile);
            }
            if !ensure_delimiter_or_eof(bytes, *current_index) {
                return Err(JsonParsingError::InvalidJsonFile);
            }
            Ok(JsonValue::Boolean(true))
        }
        b'f' => {
            if !consume_literal(bytes, current_index, FALSE_BYTES) {
                return Err(JsonParsingError::InvalidJsonFile);
            }
            if !ensure_delimiter_or_eof(bytes, *current_index) {
                return Err(JsonParsingError::InvalidJsonFile);
            }
            Ok(JsonValue::Boolean(false))
        }
        _ => Err(JsonParsingError::InvalidJsonFile),
    }
}

fn consume_literal(bytes: &[u8], index: &mut usize, lit: &[u8]) -> bool {
    if *index + lit.len() > bytes.len() {
        return false;
    }
    if &bytes[*index..*index + lit.len()] == lit {
        *index += lit.len();
        true
    } else {
        false
    }
}

fn ensure_delimiter_or_eof(bytes: &[u8], index: usize) -> bool {
    index == bytes.len() || is_delimiter(bytes[index])
}

fn trim_spaces(json_string: &str, current_index: &mut usize) {
    while *current_index < json_string.len() {
        let current_char = json_string.as_bytes()[*current_index];
        if current_char == b' '
            || current_char == b'\n'
            || current_char == b'\r'
            || current_char == b'\t'
        {
            *current_index += 1;
        } else {
            break;
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
enum JsonParsingError {
    EmptyJsonFile,
    InvalidJsonFile,
    LeadingZero
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    JsonString(String),
    Number(f64),
    Boolean(bool),
    Null,
}


///-----------------------TESTS-----------------------///
#[cfg(test)]
mod tests {
    use super::*;
    fn char_from_hex(s: &str) -> Option<char> {
        let hex = s.trim_start_matches("0x");
        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
    }
    #[test]
    fn test_process_empty_json_file() {
        let json_string = "";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap_err(), JsonParsingError::EmptyJsonFile);
    }

    #[test]
    fn test_invalid_inputs() {
        let cases = ["{", "}", "{{", "[]}"];

        for case in cases {
            let res = process_json_file(case);
            assert_eq!(
                res.unwrap_err(),
                JsonParsingError::InvalidJsonFile,
                "input was: {:?}",
                case
            );
        }
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
            let res = process_json_file(case);
            assert_eq!(res.unwrap(), JsonValue::Null)
        }
    }

    #[test]
    fn parse_bool_values() {
        let mut json_string = "true";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Boolean(true));

        json_string = "false";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Boolean(false));
    }
    #[test]
    fn parse_typos_bool_and_null() {
        let cases = ["nullx", "truex", "falsex", "fals", "tru", "nul"];

        for case in cases {
            let res = process_json_file(case);
            assert_eq!(
                res.unwrap_err(),
                JsonParsingError::InvalidJsonFile,
                "input was: {:?}",
                case
            );
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
            ("-1.2e00000003", JsonValue::Number(-1200.0))
        ];

        for (case, expected) in cases {
            let res = process_json_file(case);
            assert_eq!(res.unwrap(), expected, "input was: {:?}", case);
        }
    }

    #[test]
    fn parse_numbers_fail() {
        let cases = [
            "-",      // sign with no digits
            "-x",     // sign then junk
            "1x",     // junk after digits
            "--1",    // malformed sign (second '-' not allowed here)

            "1.",     // a digit must follow a dot
            "1.e2",   // a digit must follow a dot
            ".1",     // the number must start with a digit (after optional '-')
            "-.1",    // same

            "1e",     // exponent must have digits
            "1E",     // exponent must have digits
            "1e+",    // digits must follow the exponent sign
            "1e-",    // digits must follow the exponent sign
            "1e+x",   // junk after exponent sign
            "1ex",    // junk after exponent marker

            "1..2",   // second dot isn't allowed
            "1e2e3",  // second exponent not allowed
            "1e2.3",  // dot not allowed after exponent

            "+1",     // leading '+' not allowed by your parser (and JSON)
        ];

        for case in cases {
            let res = process_json_file(case);
            assert_eq!(
                res.unwrap_err(),
                JsonParsingError::InvalidJsonFile,
                "input was: {:?}",
                case
            );
        }
    }
    #[test]
    fn leading_zero_fail(){
        let cases = [
            "00",
            "00000"
        ];

        for case in cases {
            let res = process_json_file(case);
            assert_eq!(
                res.unwrap_err(),
                JsonParsingError::LeadingZero,
                "input was: {:?}",
                case
            );
        }
    }

    #[test]
    fn simple_strinng(){
        let cases = [
            (r#""hello""#, JsonValue::JsonString("hello".to_string())),
            (r#""hé""#, JsonValue::JsonString("hé".to_string())),
            (r#""1033""#, JsonValue::JsonString("1033".to_string())),

        ];

        for (case, expected) in cases {
            let res = process_json_file(case);
            assert_eq!(res.unwrap(), expected, "input was: {:?}", case);
        }
    }

    #[test]
    fn escape_string() {
        let cases = [
            // quote
            (r#"" \" \" ""#, JsonValue::JsonString(r#" " " "#.to_string())),
            (r#"" \" jdfjgkh \" djkdsjfkfjg ""#, JsonValue::JsonString(r#" " jdfjgkh " djkdsjfkfjg "#.to_string())),

            // backslash
            (r#"" \\ ""#, JsonValue::JsonString(r#" \ "#.to_string())),
            (r#"" path\\to\\file ""#, JsonValue::JsonString(r#" path\to\file "#.to_string())),

            // slash
            (r#"" \/ ""#, JsonValue::JsonString(" / ".to_string())),

            // control escapes
            (r#"" \n \r \t ""#, JsonValue::JsonString(" \n \r \t ".to_string())),
            (r#"" \b \f ""#, JsonValue::JsonString(format!(" {} {} ", '\u{0008}', '\u{000C}'))),

            // mixed
            (r#"" a\\b\"c\/d ""#, JsonValue::JsonString(r#" a\b"c/d "#.to_string())),
        ];

        for (case, expected) in cases {
            let value = process_json_file(case)
                .unwrap_or_else(|e| panic!("unexpected error for input {:?}: {:?}", case, e));
            assert_eq!(value, expected, "input was: {:?}", case);
        }
    }

    #[test]
    fn escape_string_fail() {
        let cases = [
            r#""\q""#,        // invalid escape
            r#""\""#,         // unterminated: ends right after an escape quote start
            r#""\\ \""#,        // unterminated: backslash escapes closing quote, no final quote
            "\"\n\"",         // raw newline inside string (not escaped)
            "\"\r\"",         // raw CR inside string (not escaped)
            "\"\t\"",         // raw tab inside string (not escaped) -- strict JSON forbids raw control chars
            r#""\u1234""#,    // unicode not supported yet (if you currently error on \u)
        ];

        for case in cases {
            let res = process_json_file(case);
            assert_eq!(
                res.unwrap_err(),
                JsonParsingError::InvalidJsonFile,
                "input was: {:?}",
                case
            );
        }
    }
}
