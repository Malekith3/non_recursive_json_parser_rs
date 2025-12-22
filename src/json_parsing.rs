use std::collections::HashMap;

pub(crate) fn process_json_file(json_string: &str) -> Result<JsonValue, JsonParsingError>  {
    if json_string.is_empty() {
        return Err(JsonParsingError::EmptyJsonFile)
    }

    if     json_string.starts_with("{") && !json_string.ends_with("}")
        || !json_string.starts_with("{") && json_string.ends_with("}")
    {
        return Err(JsonParsingError::InvalidJsonFile)
    }

    if json_string.starts_with("{") && json_string.ends_with("}") {
        return Ok(JsonValue::Object(HashMap::new()))
    }

    if json_string.starts_with("[") && json_string.ends_with("]") {
        return Ok(JsonValue::Array(Vec::new()))
    }

    Ok(JsonValue::Null)
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
    fn test_process_valid_json_file(){
        let json_string = "{}";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Object(HashMap::new()));
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
    fn test_process_json_file_with_empty_array(){
        let json_string = "[]";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Array(Vec::new()));
    }
    #[test]
    fn test_process_json_empty_json_value(){
        let json_string = " ";
        let res = process_json_file(json_string);
        assert_eq!(res.unwrap(), JsonValue::Null);
    }
}