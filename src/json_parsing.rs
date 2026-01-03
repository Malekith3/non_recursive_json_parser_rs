use std::collections::HashMap;

pub(crate) fn process_json_file(json_string: &str) -> Result<JsonValue, JsonParsingError>  {

    if json_string.is_empty() {
        return Err(JsonParsingError::EmptyJsonFile)
    }

    let mut json_index : usize = 0;

    trim_spaces(json_string, &mut json_index);

    if json_index >= json_string.len(){
        return Err(JsonParsingError::InvalidJsonFile)
    }

    let result = parse_json_value(json_string, &mut json_index);

    trim_spaces(json_string, &mut json_index);

    if json_index != json_string.len() {
        return Err(JsonParsingError::InvalidJsonFile)
    }

    result
}
fn parse_json_value(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError> {
    trim_spaces(json_string, current_index);

    if *current_index >= json_string.len(){
        return Err(JsonParsingError::InvalidJsonFile);
    }
    let json_string_bytes = json_string.as_bytes();

    match json_string_bytes[*current_index] {
        b'n' => parse_null(json_string, current_index),
        b't' | b'f' => parse_bool(json_string, current_index),
        _ => Err(JsonParsingError::InvalidJsonFile)
    }
}

fn parse_null(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError>{
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
fn parse_bool(json_string: &str, current_index: &mut usize) -> Result<JsonValue, JsonParsingError>{
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

fn trim_spaces(json_string : &str, current_index: &mut usize){

    while *current_index < json_string.len() {
        let current_char = json_string.as_bytes()[*current_index];
    if     current_char == b' '
        || current_char == b'\n'
        || current_char == b'\r'
        || current_char == b'\t' {
            *current_index += 1;
        } else {
            break;
        }
    }
}
#[derive(Debug, Clone)]
#[derive(PartialEq)]
enum JsonParsingError {
    EmptyJsonFile,
    InvalidJsonFile
}

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}


///-----------------------TESTS-----------------------///
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_process_empty_json_file(){
        let json_string = "";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap_err(), JsonParsingError::EmptyJsonFile);
    }

    #[test]
    fn test_invalid_inputs() {
        let cases = [
            "{",
            "}",
            "{{",
            "[]}",
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
    fn test_parse_null(){
        let cases = [
            "null",
            "   null",
            "null   ",
            "\r  null",
            "\n  \r  \t null  \n \r\t"
        ];

       for case in cases {
           let res = process_json_file(case);
           assert_eq!(res.unwrap(), JsonValue::Null)
       }

    }

    #[test]
    fn parse_bool_values(){
        let mut json_string = "true";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Boolean(true));

        json_string = "false";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Boolean(false));
    }
    #[test]
    fn parse_typos_bool_and_null(){
        let cases = [
            "nullx",
            "truex",
            "falsex",
            "fals",
            "tru",
            "nul"
        ];

        for case in cases {
            let res = process_json_file(case);
            assert_eq!(res.unwrap_err(), JsonParsingError::InvalidJsonFile , "input was: {:?}", case);
        }
    }
}