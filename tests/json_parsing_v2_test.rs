use indexmap::IndexMap;

use json_parser_rust::json_definitions::{JsonParsingError, JsonValue, LexerError};
use json_parser_rust::json_lexer::{NumberError, StringError};
use json_parser_rust::json_lexer_parser::process_json_string_v2;

fn assert_ok_eq(input: &str, expected: JsonValue) {
    match process_json_string_v2(input) {
        Ok(value) => assert_eq!(value, expected, "input was: {:?}", input),
        Err(err) => panic!("expected Ok, got Err: {:?} for input: {:?}", err, input),
    }
}

fn assert_err_matches<F>(input: &str, check: F)
where
    F: FnOnce(JsonParsingError) -> bool,
{
    match process_json_string_v2(input) {
        Ok(value) => panic!("expected Err, got Ok: {:?} for input: {:?}", value, input),
        Err(err) => assert!(check(err), "input was: {:?}", input),
    }
}

mod nulls {
    use super::{assert_err_matches, assert_ok_eq, JsonParsingError, JsonValue, LexerError};

    mod pos {
        use super::{assert_ok_eq, JsonValue};

        #[test]
        fn parse_null_ok() {
            let cases = ["null", "   null", "null   ", "\r  null", "\n  \r  \t null  \n \r\t"];
            for case in cases {
                assert_ok_eq(case, JsonValue::Null);
            }
        }
    }

    mod neg {
        use super::{assert_err_matches, JsonParsingError, LexerError};

        #[test]
        fn parse_null_typos_fail() {
            assert_err_matches("nul", |e| {
                matches!(e, JsonParsingError::LexError(LexerError::UnexpectedEof { expected: "null", .. }))
            });
            assert_err_matches("nullx", |e| {
                matches!(e, JsonParsingError::LexError(LexerError::UnexpectedByte { expected: "token", .. }))
            });
        }
    }
}

mod bools {
    use super::{assert_err_matches, assert_ok_eq, JsonParsingError, JsonValue, LexerError};

    mod pos {
        use super::{assert_ok_eq, JsonValue};

        #[test]
        fn parse_bool_ok() {
            assert_ok_eq("true", JsonValue::Boolean(true));
            assert_ok_eq("false", JsonValue::Boolean(false));
        }
    }

    mod neg {
        use super::{assert_err_matches, JsonParsingError, LexerError};

        #[test]
        fn parse_bool_typos_fail() {
            assert_err_matches("tru", |e| {
                matches!(e, JsonParsingError::LexError(LexerError::UnexpectedEof { expected: "true", .. }))
            });
            assert_err_matches("fals", |e| {
                matches!(e, JsonParsingError::LexError(LexerError::UnexpectedEof { expected: "false", .. }))
            });
            assert_err_matches("truex", |e| {
                matches!(e, JsonParsingError::LexError(LexerError::UnexpectedByte { expected: "token", .. }))
            });
        }
    }
}

mod strings {
    use super::{
        assert_err_matches, assert_ok_eq, JsonParsingError, JsonValue, LexerError, StringError,
    };

    mod pos {
        use super::{assert_ok_eq, JsonValue};

        #[test]
        fn parse_string_ok() {
            let cases = [
                (r#""hello""#, JsonValue::JsonString("hello".to_string())),
                (r#""with spaces""#, JsonValue::JsonString("with spaces".to_string())),
                (r#""\n\r\t""#, JsonValue::JsonString("\n\r\t".to_string())),
                (r#""\u0041\u0042""#, JsonValue::JsonString("AB".to_string())),
            ];
            for (case, expected) in cases {
                assert_ok_eq(case, expected);
            }
        }
    }

    mod neg {
        use super::{assert_err_matches, JsonParsingError, LexerError, StringError};

        #[test]
        fn parse_string_fail() {
            assert_err_matches(r#""\q""#, |e| {
                matches!(
                    e,
                    JsonParsingError::LexError(LexerError::InvalidString {
                        reason: StringError::InvalidEscape { .. },
                        ..
                    })
                )
            });
            assert_err_matches(r#""\u12X4""#, |e| {
                matches!(
                    e,
                    JsonParsingError::LexError(LexerError::InvalidString {
                        reason: StringError::InvalidUnicodeEscape,
                        ..
                    })
                )
            });
            assert_err_matches(r#""abc"#, |e| {
                matches!(
                    e,
                    JsonParsingError::LexError(LexerError::InvalidString {
                        reason: StringError::Unterminated,
                        ..
                    })
                )
            });
        }
    }
}

mod numbers {
    use super::{
        assert_err_matches, assert_ok_eq, JsonParsingError, JsonValue, LexerError, NumberError,
    };

    mod pos {
        use super::{assert_ok_eq, JsonValue};

        #[test]
        fn parse_number_ok() {
            let cases = [
                ("0", JsonValue::Number(0.0)),
                ("-1", JsonValue::Number(-1.0)),
                ("42", JsonValue::Number(42.0)),
                ("3.14", JsonValue::Number(3.14)),
                ("1e3", JsonValue::Number(1000.0)),
                ("-1.2E3", JsonValue::Number(-1200.0)),
            ];
            for (case, expected) in cases {
                assert_ok_eq(case, expected);
            }
        }
    }

    mod neg {
        use super::{assert_err_matches, JsonParsingError, LexerError, NumberError};

        #[test]
        fn parse_number_fail() {
            assert_err_matches("01", |e| {
                matches!(
                    e,
                    JsonParsingError::LexError(LexerError::InvalidNumber {
                        reason: NumberError::LeadingZero,
                        ..
                    })
                )
            });
            assert_err_matches("1.", |e| {
                matches!(
                    e,
                    JsonParsingError::LexError(LexerError::InvalidNumber {
                        reason: NumberError::MissingFracDigit,
                        ..
                    })
                )
            });
            assert_err_matches("1e", |e| {
                matches!(
                    e,
                    JsonParsingError::LexError(LexerError::InvalidNumber {
                        reason: NumberError::MissingExpDigit,
                        ..
                    })
                )
            });
        }
    }
}

mod arrays {
    use super::{assert_err_matches, assert_ok_eq, JsonParsingError, JsonValue};

    mod pos {
        use super::{assert_ok_eq, JsonValue};

        #[test]
        fn parse_array_ok_more() {
            let cases = [
                (r#"[1]"#, JsonValue::Array(vec![JsonValue::Number(1.0)])),
                (r#"[  ]"#, JsonValue::Array(vec![])),
                (
                    r#"[[1,2],[3],[[]]]"#,
                    JsonValue::Array(vec![
                        JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
                        JsonValue::Array(vec![JsonValue::Number(3.0)]),
                        JsonValue::Array(vec![JsonValue::Array(vec![])]),
                    ]),
                ),
                (
                    r#"[1, [2, 3], 4]"#,
                    JsonValue::Array(vec![
                        JsonValue::Number(1.0),
                        JsonValue::Array(vec![JsonValue::Number(2.0), JsonValue::Number(3.0)]),
                        JsonValue::Number(4.0),
                    ]),
                ),
            ];

            for (case, expected) in cases {
                assert_ok_eq(case, expected);
            }
        }

        #[test]
        fn parse_array_ok_deep_nested() {
            let input = r#"[1,[2,[3,[4,[5]]]]]"#;
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Array(vec![
                    JsonValue::Number(2.0),
                    JsonValue::Array(vec![
                        JsonValue::Number(3.0),
                        JsonValue::Array(vec![
                            JsonValue::Number(4.0),
                            JsonValue::Array(vec![JsonValue::Number(5.0)]),
                        ]),
                    ]),
                ]),
            ]);

            assert_ok_eq(input, expected);
        }
    }

    mod neg {
        use super::{assert_err_matches, JsonParsingError};

        #[test]
        fn parse_array_fail_more() {
            let cases = [
                r#"[1 2]"#,   // missing comma
                r#"[1,]"#,    // trailing comma
                r#"[,1]"#,    // leading comma
                r#"[1,,2]"#,  // double comma
                r#"[1"#,      // unterminated
                r#"[1]]"#,    // extra closing bracket
                r#"[""#,      // unterminated string inside array (lexer/parser may surface different error)
            ];

            for case in cases {
                assert_err_matches(case, |e| {
                    matches!(
                        e,
                        JsonParsingError::InvalidJsonFile
                            | JsonParsingError::InvalidArray
                            | JsonParsingError::UnexpectedEOF
                            | JsonParsingError::LexError(_)
                    )
                });
            }
        }

        #[test]
        fn parse_array_fail_deep_nested() {
            let cases = [
                r#"[1,[2,[3,[4,[5]]]]"#,   // missing final closing bracket
                r#"[1,[2,[3,[4,[5,]]]]]"#, // trailing comma in deepest array
                r#"[1,[2,[3,[4,[5]]]]]]"#, // extra closing bracket
            ];

            for case in cases {
                assert_err_matches(case, |e| {
                    matches!(
                        e,
                        JsonParsingError::InvalidJsonFile
                            | JsonParsingError::InvalidArray
                            | JsonParsingError::UnexpectedEOF
                            | JsonParsingError::LexError(_)
                    )
                });
            }
        }
    }
}

mod objects {
    use super::{assert_err_matches, assert_ok_eq, IndexMap, JsonParsingError, JsonValue};

    mod pos {
        use super::{assert_ok_eq, IndexMap, JsonValue};

        #[test]
        fn parse_object_ok() {
            let mut m1 = IndexMap::new();
            m1.insert("a".to_string(), JsonValue::Number(1.0));

            let mut m2 = IndexMap::new();
            m2.insert("a".to_string(), JsonValue::Number(1.0));
            m2.insert("b".to_string(), JsonValue::Boolean(true));
            m2.insert("c".to_string(), JsonValue::Null);
            m2.insert("d".to_string(), JsonValue::JsonString("x".to_string()));

            let cases = [
                (r#"{}"#, JsonValue::Object(IndexMap::new())),
                (r#"{ "a": 1 }"#, JsonValue::Object(m1)),
                (
                    r#"{ "a":1, "b":true, "c":null, "d":"x" }"#,
                    JsonValue::Object(m2),
                ),
            ];
            for (case, expected) in cases {
                assert_ok_eq(case, expected);
            }
        }
    }

    mod neg {
        use super::{assert_err_matches, JsonParsingError};

        #[test]
        fn parse_object_fail() {
            let cases = [
                r#"{"#,             // unterminated
                r#"}"#,             // stray close
                r#"{,}"#,           // leading comma
                r#"{"a":}"#,        // missing value
                r#"{"a" 1}"#,       // missing colon
                r#"{"a":1,}"#,      // trailing comma
                r#"{a:1}"#,         // key not quoted
            ];
            for case in cases {
                assert_err_matches(case, |e| {
                    matches!(
                        e,
                        JsonParsingError::InvalidJsonFile
                            | JsonParsingError::InvalidJsonObject
                            | JsonParsingError::UnexpectedEOF
                            | JsonParsingError::LexError(_)
                    )
                });
            }
        }
    }
}
