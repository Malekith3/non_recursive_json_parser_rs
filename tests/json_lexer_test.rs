use json_parser_rust::json_lexer::{
    lex_all, NumberError, StringError, TokenKind,
};

use json_parser_rust::json_definitions::LexerError;

mod punctuation {
    use super::{lex_all, LexerError, TokenKind};

    mod pos {
        use super::{lex_all, TokenKind};

        #[test]
        fn sequence() {
            let input = b"{}[]:,";
            let tokens = lex_all(input).expect("lex_all should succeed for punctuation");

            let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
            assert_eq!(
                kinds,
                vec![
                    TokenKind::LBrace,
                    TokenKind::RBrace,
                    TokenKind::LBracket,
                    TokenKind::RBracket,
                    TokenKind::Colon,
                    TokenKind::Comma,
                    TokenKind::Eof,
                ]
            );
        }

        #[test]
        fn with_whitespace() {
            let input = b" { } [ ] : , \n\t";
            let tokens = lex_all(input).expect("lex_all should succeed with whitespace");

            let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
            assert_eq!(
                kinds,
                vec![
                    TokenKind::LBrace,
                    TokenKind::RBrace,
                    TokenKind::LBracket,
                    TokenKind::RBracket,
                    TokenKind::Colon,
                    TokenKind::Comma,
                    TokenKind::Eof,
                ]
            );
        }
    }

    mod neg {
        use super::{lex_all, LexerError};

        #[test]
        fn unknown_punctuation() {
            let input = b";";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::UnexpectedByte {
                    at: 0,
                    found: b';',
                    expected: "token"
                }
            );
        }

        #[test]
        fn punctuation_followed_by_unknown_byte() {
            let input = b"{}%";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::UnexpectedByte {
                    at: 2,
                    found: b'%',
                    expected: "token"
                }
            );
        }
    }
}

mod literals {
    use super::{lex_all, LexerError, TokenKind};

    mod pos {
        use super::{lex_all, TokenKind};

        #[test]
        fn keywords_with_whitespace() {
            let input = b" \n\ttrue\r ";
            let tokens = lex_all(input).expect("lex_all should succeed with surrounding whitespace");

            let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::Bool(true), TokenKind::Eof]);
        }

        #[test]
        fn multiple_keywords_separated_by_whitespace() {
            let input = b"true false null";
            let tokens = lex_all(input).expect("lex_all should succeed for multiple keywords");

            let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
            assert_eq!(
                kinds,
                vec![TokenKind::Bool(true), TokenKind::Bool(false), TokenKind::Null, TokenKind::Eof]
            );
        }

        #[test]
        fn keywords_adjacent_to_punctuation() {
            let input = b"[true,false,null]";
            let tokens = lex_all(input).expect("lex_all should succeed for keywords with punctuation");

            let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
            assert_eq!(
                kinds,
                vec![
                    TokenKind::LBracket,
                    TokenKind::Bool(true),
                    TokenKind::Comma,
                    TokenKind::Bool(false),
                    TokenKind::Comma,
                    TokenKind::Null,
                    TokenKind::RBracket,
                    TokenKind::Eof
                ]
            );
        }

        #[test]
        fn numbers_without_commas_are_lexed() {
            let input = b"[1 2]";
            let tokens = lex_all(input).expect("lex_all should succeed for lexer-only test");

            let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
            assert_eq!(
                kinds,
                vec![
                    TokenKind::LBracket,
                    TokenKind::Number(1.0),
                    TokenKind::Number(2.0),
                    TokenKind::RBracket,
                    TokenKind::Eof
                ]
            );
        }
    }

    mod neg {
        use super::{lex_all, LexerError};

        #[test]
        fn rejects_partial_true() {
            let input = b"tru";
            let err = lex_all(input).unwrap_err();
            assert_eq!(err, LexerError::UnexpectedEof { at: 3, expected: "true" });
        }

        #[test]
        fn rejects_partial_false() {
            let input = b"fals";
            let err = lex_all(input).unwrap_err();
            assert_eq!(err, LexerError::UnexpectedEof { at: 4, expected: "false" });
        }

        #[test]
        fn rejects_partial_null() {
            let input = b"nul";
            let err = lex_all(input).unwrap_err();
            assert_eq!(err, LexerError::UnexpectedEof { at: 3, expected: "null" });
        }

        #[test]
        fn rejects_true_with_extra_trailing_char() {
            let input = b"truee";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::UnexpectedByte { at: 4, found: b'e', expected: "token" }
            );
        }

        #[test]
        fn rejects_false_with_extra_trailing_char() {
            let input = b"falsex";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::UnexpectedByte { at: 5, found: b'x', expected: "token" }
            );
        }

        #[test]
        fn rejects_null_with_extra_trailing_char() {
            let input = b"nullx";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::UnexpectedByte { at: 4, found: b'x', expected: "token" }
            );
        }
    }
}

mod strings {
    use super::{lex_all, LexerError, StringError, TokenKind};

    mod pos {
        use super::{lex_all, TokenKind};

        #[test]
        fn empty() {
            let input = br#""""#;
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::String("".to_string()), TokenKind::Eof]);
        }

        #[test]
        fn ascii() {
            let input = br#""hello""#;
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::String("hello".to_string()), TokenKind::Eof]);
        }

        #[test]
        fn with_spaces() {
            let input = br#""hello world""#;
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::String("hello world".to_string()), TokenKind::Eof]);
        }

        #[test]
        fn with_escaped_quote() {
            let input = br#""he\"llo""#;
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::String("he\"llo".to_string()), TokenKind::Eof]);
        }

        #[test]
        fn with_backslash_escape() {
            let input = br#""C:\\path""#;
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::String("C:\\path".to_string()), TokenKind::Eof]);
        }

        #[test]
        fn with_unicode_escape_ascii() {
            let input = br#""\u0041\u0042""#;
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::String("AB".to_string()), TokenKind::Eof]);
        }
    }

    mod neg {
        use super::{lex_all, LexerError, StringError};

        #[test]
        fn rejects_unterminated() {
            let input = br#""abc"#; // bytes: [0]='"', [1]='a', [2]='b', [3]='c', EOF at index 4
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::InvalidString { at: 4, reason: StringError::Unterminated }
            );
        }

        #[test]
        fn rejects_raw_control_char() {
            let input: &[u8] = b"\"a\x01b\""; // '"' 'a' 0x01 'b' '"'
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::InvalidString { at: 2, reason: StringError::ControlChar { found: 0x01 } }
            );
        }

        #[test]
        fn rejects_invalid_escape() {
            let input = br#""\q""#;
            let err = lex_all(input).unwrap_err();
            assert!(matches!(
                err,
                LexerError::InvalidString { reason: StringError::InvalidEscape { .. }, .. }
            ));
        }

        #[test]
        fn rejects_trailing_backslash() {
            let input = br#""abc\"#;
            let err = lex_all(input).unwrap_err();
            assert!(matches!(
                err,
                LexerError::InvalidString { reason: StringError::TrailingBackslash, .. }
            ));
        }

        #[test]
        fn rejects_invalid_unicode_escape() {
            let input = br#""\u12X4""#;
            let err = lex_all(input).unwrap_err();
            assert!(matches!(
                err,
                LexerError::InvalidString { reason: StringError::InvalidUnicodeEscape, .. }
            ));
        }

        #[test]
        fn rejects_surrogate_unicode_escape() {
            let input = br#""\uD800""#;
            let err = lex_all(input).unwrap_err();
            assert!(matches!(
                err,
                LexerError::InvalidString { reason: StringError::SurrogateNotAllowed, .. }
            ));
        }
    }
}

mod numbers {
    use super::{lex_all, LexerError, TokenKind};

    mod pos {
        use super::{lex_all, TokenKind};

        #[test]
        fn zero() {
            let input = b"0";
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::Number(0.0), TokenKind::Eof]);
        }

        #[test]
        fn integer() {
            let input = b"42";
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::Number(42.0), TokenKind::Eof]);
        }

        #[test]
        fn fraction() {
            let input = b"3.14";
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::Number(3.14), TokenKind::Eof]);
        }

        #[test]
        fn exponent() {
            let input = b"1e3";
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::Number(1000.0), TokenKind::Eof]);
        }

        #[test]
        fn negative() {
            let input = b"-12";
            let kinds: Vec<TokenKind> = lex_all(input).unwrap().into_iter().map(|t| t.kind).collect();
            assert_eq!(kinds, vec![TokenKind::Number(-12.0), TokenKind::Eof]);
        }
    }

    mod neg {
        use super::{lex_all, LexerError};
        use super::super::NumberError;

        #[test]
        fn rejects_leading_zero() {
            let input = b"01";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::InvalidNumber { at: 0, reason: NumberError::LeadingZero }
            );
        }

        #[test]
        fn rejects_missing_fraction_digits() {
            let input = b"1.";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::InvalidNumber { at: 2, reason: NumberError::MissingFracDigit }
            );
        }

        #[test]
        fn rejects_missing_exponent_digits() {
            let input = b"1e";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::InvalidNumber { at: 2, reason: NumberError::MissingExpDigit }
            );
        }

        #[test]
        fn rejects_plus_sign() {
            let input = b"+1";
            let err = lex_all(input).unwrap_err();
            assert_eq!(
                err,
                LexerError::UnexpectedByte {
                    at: 0,
                    found: b'+',
                    expected: "token"
                }
            );
        }
    }
}

mod complex_exceptions {
    use super::{lex_all, LexerError};

    #[test]
    fn unquoted_object_key_should_fail() {
        let input = br#"{a:1}"#;
        let err = lex_all(input).unwrap_err();
        assert_eq!(
            err,
            LexerError::UnexpectedByte {
                at: 1,
                found: b'a',
                expected: "token"
            }
        );
    }
}
