use std::str::from_utf8;
use std::string::String;
use indexmap::IndexMap;

const HEX_ASCI_OFFSET_CAP: u8 = 0x41;
const HEX_ASCI_OFFSET_LOW: u8 = 0x61;
const HEX_ASCI_OFFSET_NUM: u8 = 0x30;

pub(crate) fn process_json_string(json_string: &str) -> Result<JsonValue, JsonParsingError> {
    if json_string.is_empty() {
        return Err(JsonParsingError::EmptyJsonFile);
    }

    let mut json_index: usize = 0;

    trim_spaces(json_string, &mut json_index);

    if json_index >= json_string.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let result = parse_json_value(json_string, &mut json_index)?;
    trim_spaces(json_string, &mut json_index);

    Ok(result)
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
        b'-' | b'0'..=b'9' => parse_number(json_string, current_index),
        b'\"' => parse_string(json_string, current_index),
        b'[' => parse_array(json_string, current_index),
        b'{' => parse_json_object(json_string, current_index),
        _ => Err(JsonParsingError::InvalidJsonFile),
    }
}

fn parse_json_object(
    json_string: &str,
    current_index: &mut usize,
) -> Result<JsonValue, JsonParsingError> {
    //consume { token
    *current_index+=1;
    let bytes = json_string.as_bytes();

    if *current_index >= bytes.len() {
        return Err(JsonParsingError::InvalidJsonObject);
    }
    let mut expecting_colum = false;
    let mut expecting_string = true;
    let mut expecting_value = false;
    let mut expecting_coma = false;
    let mut out_json_object: IndexMap<String, JsonValue> = IndexMap::new();
    let mut current_key_str: String = String::new();

    trim_spaces(json_string, current_index);

    // empty array: []
    if *current_index < bytes.len() && bytes[*current_index] == b'}' {
        *current_index += 1;
        return Ok(JsonValue::Object(out_json_object));
    }

    while *current_index < bytes.len() {
        trim_spaces(json_string, current_index);

        if *current_index >= bytes.len() {
            return Err(JsonParsingError::InvalidJsonObject);
        }

        match bytes[*current_index] {
            b'}' => {
                if  !expecting_coma {
                    return Err(JsonParsingError::InvalidJsonObject);
                }
                *current_index += 1;
                return Ok(JsonValue::Object(out_json_object));
            }
            b'\"' => {
                if !expecting_string  && !expecting_value{
                    return Err(JsonParsingError::InvalidJsonObject);
                }

                if expecting_string {
                    let key_value = parse_string(json_string, current_index)?;
                    let key = match key_value {
                        JsonValue::JsonString(s) => s,
                        _ => return Err(JsonParsingError::InvalidJsonObject),
                    };
                    current_key_str = key;
                    expecting_string = false;
                    expecting_colum = true;
                    continue;
                }

                if expecting_value {
                    let v = parse_json_value(json_string, current_index)?;
                    out_json_object.insert(current_key_str.to_string(),v);
                    expecting_value = false;
                    expecting_coma = true;
                }

            }
            b':' => {
                if !expecting_colum {
                    return Err(JsonParsingError::InvalidJsonObject);
                }
                *current_index += 1;
                expecting_value = true;
                expecting_colum = false;
            }
            b',' => {
                if !expecting_coma {
                    return Err(JsonParsingError::InvalidJsonObject);
                }
                *current_index += 1;
                expecting_string = true;
                expecting_coma = false;
            }
            _ => {
                if !expecting_value {
                    return Err(JsonParsingError::InvalidJsonObject);
                }
                let v = parse_json_value(json_string, current_index)?;
                out_json_object.insert(current_key_str.to_string(),v);
                expecting_value = false;
                expecting_coma = true;
            }
        }
    }


    Err(JsonParsingError::InvalidJsonObject)
}

fn parse_array(
    json_string: &str,
    current_index: &mut usize,
) -> Result<JsonValue, JsonParsingError> {
    // consume '['
    *current_index += 1;
    let bytes = json_string.as_bytes();

    if *current_index >= bytes.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let mut out_array = Vec::new();
    let mut expect_value = true;

    trim_spaces(json_string, current_index);

    // empty array: []
    if *current_index < bytes.len() && bytes[*current_index] == b']' {
        *current_index += 1;
        return Ok(JsonValue::Array(out_array));
    }

    while *current_index < bytes.len() {
        trim_spaces(json_string, current_index);

        if *current_index >= bytes.len() {
            return Err(JsonParsingError::InvalidArray);
        }

        match bytes[*current_index] {
            b']' => {
                if expect_value {
                    return Err(JsonParsingError::InvalidArray);
                }
                *current_index += 1;
                return Ok(JsonValue::Array(out_array));
            }

            b',' => {
                if expect_value {
                    return Err(JsonParsingError::InvalidArray);
                }
                *current_index += 1;
                expect_value = true;
            }

            _ => {
                if !expect_value {
                    return Err(JsonParsingError::InvalidArray);
                }
                let v = parse_json_value(json_string, current_index)?;
                out_array.push(v);
                expect_value = false;
            }
        }
    }

    Err(JsonParsingError::InvalidArray)
}
fn consume_unicode(bytes_stream: &[u8],
                   current_index: &mut usize,
) -> Result<([u8; 4], usize), JsonParsingError> {

    // Move after seen u separator
    *current_index += 1;

    let mut out_buffer = [0u8; 4];

    // Gather
    for i in 0..4 {
        if *current_index >= bytes_stream.len() {
            return Err(JsonParsingError::InvalidUnicodeInString);
        }

        match bytes_stream[*current_index] {
            b'0'..=b'9' => {
                out_buffer[i] = bytes_stream[*current_index] - HEX_ASCI_OFFSET_NUM;
            }
            b'A'..=b'F' => {
                out_buffer[i] = bytes_stream[*current_index] - HEX_ASCI_OFFSET_CAP + 0xA
            }
            b'a'..=b'f' => {
                out_buffer[i] = bytes_stream[*current_index] - HEX_ASCI_OFFSET_LOW + 0xA
            }

            _ => { return Err(JsonParsingError::InvalidUnicodeInString) }
        }
        *current_index += 1;
    }

    let code: u32 =
        ((out_buffer[0] as u32) << 12) |
        ((out_buffer[1] as u32) << 8)  |
        ((out_buffer[2] as u32) << 4)  |
        ((out_buffer[3] as u32) << 0);

    if code >= 0xD800 && code <= 0xDFFF {
        return Err(JsonParsingError::InvalidUnicodeInString);
    }

    let  unicode_char  = char::from_u32(code).ok_or(JsonParsingError::InvalidUnicodeInString)?;

    let mut buf = [0u8; 4];
    let len = unicode_char.encode_utf8(&mut buf).len();

    Ok((buf,len))
}

fn consume_escape_char(
    bytes_stream: &[u8],
    current_index: &mut usize,
) -> Result<([u8; 4], usize), JsonParsingError> {
    //check if we have scape char even if it double the work
    if *current_index >= bytes_stream.len() || bytes_stream[*current_index] != b'\\' {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    //consume escape char
    *current_index += 1;

    // Need one more byte for the escape kind
    if *current_index >= bytes_stream.len() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let mut out = [0u8; 4];
    match bytes_stream[*current_index] {
        b'"' => {
            out[0] = b'"';
        }
        b'\\' => {
            out[0] = b'\\';
        }
        b'/' => {
            out[0] = b'/';
        }
        b'b' => {
            out[0] = 0x08;
        }
        b'f' => {
            out[0] = 0x0C;
        }
        b'n' => {
            out[0] = b'\n';
        }
        b'r' => {
            out[0] = b'\r';
        }
        b't' => {
            out[0] = b'\t';
        }
        b'u' => {
             return consume_unicode(bytes_stream, current_index);
        }
        _ => return Err(JsonParsingError::InvalidJsonFile),
    };

    *current_index += 1;
    Ok((out, 1))
}

fn parse_string(
    json_string: &str,
    current_index: &mut usize,
) -> Result<JsonValue, JsonParsingError> {
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

    let s = from_utf8(&out)
        .map_err(|_| JsonParsingError::InvalidJsonFile)?
        .to_owned();
    Ok(JsonValue::JsonString(s))
}

fn parse_number(
    json_string: &str,
    current_index: &mut usize,
) -> Result<JsonValue, JsonParsingError> {
    let bytes = json_string.as_bytes();
    let negative = if bytes[*current_index] == b'-' {
        *current_index += 1;
        -1.0
    } else {
        1.0
    };

    if *current_index >= bytes.len() || !bytes[*current_index].is_ascii_digit() {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    if bytes[*current_index] == b'0' {
        let next = *current_index + 1;
        if next < bytes.len() && bytes[next].is_ascii_digit() {
            return Err(JsonParsingError::LeadingZero);
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
                need_exp_digit = true; // must see digits in exponent
                exp_sign_allowed = true; // may accept +/-
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

    if need_frac_digit || need_exp_digit {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    if !ensure_delimiter_or_eof(bytes, *current_index) {
        return Err(JsonParsingError::InvalidJsonFile);
    }

    let string_number_rep = &json_string[start_index..*current_index];
    let num: f64 = string_number_rep
        .parse()
        .map_err(|_| JsonParsingError::InvalidJsonFile)?;

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
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b',' | b']' | b'}' | b':')
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
pub(crate) enum JsonParsingError {
    EmptyJsonFile,
    InvalidJsonFile,
    LeadingZero,
    InvalidUnicodeInString,
    InvalidArray,
    InvalidJsonObject
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

#[cfg(test)]
mod json_parsing_tests;
