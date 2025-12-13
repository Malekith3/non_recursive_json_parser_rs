pub(crate) fn process_json_file(json_string: &str) {
    println!("{}", json_string);
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_process_json_file(){
        let json_string = r#"{"name":"John","age":30,"city":"New York"}"#;
        process_json_file(json_string);
    }
}