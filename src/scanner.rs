use crate::common::{error::*, token::*};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    pub raw_source: Vec<String>,
    pub line: i32,
    pub column: i32,
    keywords: HashMap<&'a str, TokenType>,
    err: bool,
}
impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source: source.chars().peekable(),
            raw_source: source
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            line: 1,
            column: 1,
            err: false,
            keywords: HashMap::from([
                ("void", TokenType::Void),
                ("int", TokenType::Int),
                ("long", TokenType::Long),
                ("char", TokenType::Char),
                ("if", TokenType::If),
                ("else", TokenType::Else),
                ("for", TokenType::For),
                ("while", TokenType::While),
                ("return", TokenType::Return),
            ]),
        }
    }

    fn match_next(&mut self, expected: char, if_match: TokenType, if_not: TokenType) -> TokenType {
        match self.source.next_if_eq(&expected) {
            Some(_v) => if_match,
            None => if_not,
        }
    }
    fn add_token(&mut self, tokens: &mut Vec<Token>, current_token: TokenType) {
        tokens.push(Token {
            token: current_token.clone(),
            line_index: self.line,
            column: self.column,
            line_string: self.raw_source[(self.line - 1) as usize].clone(),
        });
        self.column += Self::get_token_len(current_token);
    }
    fn get_token_len(token: TokenType) -> i32 {
        match token {
            TokenType::BangEqual
            | TokenType::EqualEqual
            | TokenType::GreaterEqual
            | TokenType::LessEqual => 2,
            TokenType::String(s) => (s.len() + 2) as i32,
            TokenType::Ident(s) => s.len() as i32,
            TokenType::Int | TokenType::For => 3,
            TokenType::Char | TokenType::Else | TokenType::Long => 4,
            TokenType::While => 5,
            TokenType::If => 2,
            TokenType::Return => 6,
            TokenType::Number(n) => n.to_string().len() as i32,
            _ => 1,
        }
    }
    pub fn scan_token(&mut self) -> Result<Vec<Token>, Vec<Error>> {
        let mut errors: Vec<Error> = Vec::new();
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(c) = self.source.next() {
            match c {
                '[' => self.add_token(&mut tokens, TokenType::LeftBracket),
                ']' => self.add_token(&mut tokens, TokenType::RightBracket),
                '(' => self.add_token(&mut tokens, TokenType::LeftParen),
                ')' => self.add_token(&mut tokens, TokenType::RightParen),
                '{' => self.add_token(&mut tokens, TokenType::LeftBrace),
                '}' => self.add_token(&mut tokens, TokenType::RightBrace),
                ',' => self.add_token(&mut tokens, TokenType::Comma),
                '.' => self.add_token(&mut tokens, TokenType::Dot),
                ';' => self.add_token(&mut tokens, TokenType::Semicolon),
                '~' => self.add_token(&mut tokens, TokenType::Tilde),
                '-' => {
                    let mut token = TokenType::Minus;
                    if self.matches('-') {
                        token = TokenType::MinusMinus;
                    } else if self.matches('=') {
                        token = TokenType::MinusEqual;
                    }
                    self.add_token(&mut tokens, token);
                }
                '+' => {
                    let mut token = TokenType::Plus;
                    if self.matches('+') {
                        token = TokenType::PlusPlus;
                    } else if self.matches('=') {
                        token = TokenType::PlusEqual;
                    }
                    self.add_token(&mut tokens, token);
                }
                '|' => {
                    let mut token = TokenType::Pipe;
                    if self.matches('|') {
                        token = TokenType::PipePipe;
                    } else if self.matches('=') {
                        token = TokenType::PipeEqual;
                    }
                    self.add_token(&mut tokens, token);
                }
                '&' => {
                    let mut token = TokenType::Amp;
                    if self.matches('&') {
                        token = TokenType::AmpAmp;
                    } else if self.matches('=') {
                        token = TokenType::AmpEqual;
                    }
                    self.add_token(&mut tokens, token);
                }
                '<' => {
                    let mut token = TokenType::Less;
                    if self.matches('<') {
                        token = self.match_next('=', TokenType::LessLessEqual, TokenType::LessLess);
                    } else if self.matches('=') {
                        token = TokenType::LessEqual;
                    }
                    self.add_token(&mut tokens, token);
                }
                '>' => {
                    let mut token = TokenType::Greater;
                    if self.matches('>') {
                        token = self.match_next(
                            '=',
                            TokenType::GreaterGreaterEqual,
                            TokenType::GreaterGreater,
                        );
                    } else if self.matches('=') {
                        token = TokenType::GreaterEqual;
                    }
                    self.add_token(&mut tokens, token);
                }
                '^' => {
                    let token = self.match_next('=', TokenType::XorEqual, TokenType::Xor);
                    self.add_token(&mut tokens, token);
                }
                '*' => {
                    let token = self.match_next('=', TokenType::StarEqual, TokenType::Star);
                    self.add_token(&mut tokens, token);
                }
                '%' => {
                    let token = self.match_next('=', TokenType::ModEqual, TokenType::Mod);
                    self.add_token(&mut tokens, token);
                }

                '!' => {
                    let token = self.match_next('=', TokenType::BangEqual, TokenType::Bang);
                    self.add_token(&mut tokens, token);
                }
                '=' => {
                    let token = self.match_next('=', TokenType::EqualEqual, TokenType::Equal);
                    self.add_token(&mut tokens, token);
                }

                '/' => {
                    if self.matches('/') {
                        // there has to be a better way to consume the iter without the first \n
                        while self
                            .source
                            .by_ref()
                            .next_if(|&c| c != '\n' && c != '\0')
                            .is_some()
                        {}
                    } else {
                        let token = self.match_next('=', TokenType::SlashEqual, TokenType::Slash);
                        self.add_token(&mut tokens, token);
                    }
                }
                ' ' | '\r' | '\t' => self.column += 1,
                '\n' => {
                    self.line += 1;
                    self.column = 1
                }

                '"' => match self.string() {
                    Ok(string) => self.add_token(&mut tokens, TokenType::String(string.clone())),
                    Err(e) => {
                        self.err = true;
                        errors.push(e)
                    }
                },
                '\'' => match self.char_lit() {
                    Ok(char) => self.add_token(&mut tokens, TokenType::CharLit(char as i8)),
                    Err(e) => {
                        self.err = true;
                        errors.push(e)
                    }
                },

                _ => {
                    if c.is_ascii_digit() {
                        // Number
                        let mut num = String::new();
                        // have to prepend already consumned char
                        num.push(c);

                        while let Some(digit) = self.source.by_ref().next_if(|c| c.is_ascii_digit())
                        {
                            num.push(digit);
                        }
                        self.add_token(&mut tokens, TokenType::Number(num.parse::<i32>().unwrap()));
                    } else if c.is_alphabetic() || c == '_' {
                        // Identifier
                        let mut value = String::new();
                        value.push(c);
                        while let Some(v) = self
                            .source
                            .by_ref()
                            .next_if(|c| c.is_alphabetic() || *c == '_' || c.is_ascii_digit())
                        {
                            value.push(v);
                        }
                        if self.keywords.contains_key(&value as &str) {
                            self.add_token(
                                &mut tokens,
                                self.keywords.get(&value as &str).unwrap().clone(),
                            );
                        } else {
                            self.add_token(&mut tokens, TokenType::Ident(value.to_string()))
                        }
                    } else {
                        self.err = true;
                        errors.push(Error::new_scan_error(
                            self,
                            &format!("Unexpected character: {c}").to_string(),
                        ));
                        self.column += 1;
                    }
                }
            }
        }
        match self.err {
            true => Err(errors),
            false => Ok(tokens),
        }
    }

    fn matches(&mut self, expected: char) -> bool {
        match self.source.peek() {
            Some(v) => {
                if *v != expected {
                    return false;
                }
            }
            None => return false,
        }
        self.source.next();
        true
    }
    fn char_lit(&mut self) -> Result<char, Error> {
        let mut last_char = '\0';
        let result = self
            .source
            .by_ref()
            .take_while(|c| {
                last_char = *c;
                *c != '\''
            })
            .collect::<String>();
        if last_char != '\'' {
            return Err(Error::new_scan_error(self, "unterminated char literal"));
        } else if result.len() != 1 {
            return Err(Error::new_scan_error(
                self,
                "char literal must contain single character",
            ));
        } else if !result.is_ascii() {
            return Err(Error::new_scan_error(
                self,
                "char literal must be valid ascii value",
            ));
        }

        Ok(result.chars().next().unwrap())
    }

    fn string(&mut self) -> Result<String, Error> {
        let mut last_char = '\0';
        let result = self
            .source
            .by_ref()
            .take_while(|c| {
                last_char = *c;
                *c != '"'
            })
            .collect::<String>();
        if last_char != '"' {
            return Err(Error {
                line_index: self.line,
                line_string: self.raw_source[(self.line - 1) as usize].clone(),
                column: self.column,
                msg: "Unterminated string".to_string(),
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;

    #[test]
    fn basic_single_and_double_tokens() {
        let source = "!= = > == \n\n    ;";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!("test"),
        };
        let expected = vec![
            Token::new(TokenType::BangEqual, 1, 1, "!= = > == ".to_string()),
            Token::new(TokenType::Equal, 1, 4, "!= = > == ".to_string()),
            Token::new(TokenType::Greater, 1, 6, "!= = > == ".to_string()),
            Token::new(TokenType::EqualEqual, 1, 8, "!= = > == ".to_string()),
            Token::new(TokenType::Semicolon, 3, 5, "    ;".to_string()),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn ignores_comments() {
        let source = "// this is a    comment\n\n!this";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!("test"),
        };
        let expected = vec![
            Token::new(TokenType::Bang, 3, 1, "!this".to_string()),
            Token::new(
                TokenType::Ident("this".to_string()),
                3,
                2,
                "!this".to_string(),
            ),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn token_basic_math_expression() {
        let source = "3 + 1 / 4";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!("test"),
        };
        let expected = vec![
            Token::new(TokenType::Number(3), 1, 1, "3 + 1 / 4".to_string()),
            Token::new(TokenType::Plus, 1, 3, "3 + 1 / 4".to_string()),
            Token::new(TokenType::Number(1), 1, 5, "3 + 1 / 4".to_string()),
            Token::new(TokenType::Slash, 1, 7, "3 + 1 / 4".to_string()),
            Token::new(TokenType::Number(4), 1, 9, "3 + 1 / 4".to_string()),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn basic_math_double_digit_nums() {
        let source = "300 - 11 * 41";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!("test"),
        };
        let expected = vec![
            Token::new(TokenType::Number(300), 1, 1, "300 - 11 * 41".to_string()),
            Token::new(TokenType::Minus, 1, 5, "300 - 11 * 41".to_string()),
            Token::new(TokenType::Number(11), 1, 7, "300 - 11 * 41".to_string()),
            Token::new(TokenType::Star, 1, 10, "300 - 11 * 41".to_string()),
            Token::new(TokenType::Number(41), 1, 12, "300 - 11 * 41".to_string()),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn matches_keywords_and_strings() {
        let source = "int some = \"this is a string\"";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!("test"),
        };
        let expected = vec![
            Token::new(
                TokenType::Int,
                1,
                1,
                "int some = \"this is a string\"".to_string(),
            ),
            Token::new(
                TokenType::Ident("some".to_string()),
                1,
                5,
                "int some = \"this is a string\"".to_string(),
            ),
            Token::new(
                TokenType::Equal,
                1,
                10,
                "int some = \"this is a string\"".to_string(),
            ),
            Token::new(
                TokenType::String("this is a string".to_string()),
                1,
                12,
                "int some = \"this is a string\"".to_string(),
            ),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn errors_on_unterminated_string() {
        let source = "int some = \"this is a string";
        let mut scanner = Scanner::new(source);

        let result = match scanner.scan_token() {
            Ok(v) => panic!(),
            Err(e) => e,
        };
        let expected = vec![Error {
            line_index: 1,
            line_string: "int some = \"this is a string".to_string(),
            column: 12,
            msg: "Unterminated string".to_string(),
        }];
        assert_eq!(result, expected);
    }
    #[test]
    fn matches_complex_keywords() {
        let source = "int some_long;\nwhile (val >= 12) {*p = val}";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!("test"),
        };
        let expected = vec![
            Token::new(TokenType::Int, 1, 1, "int some_long;".to_string()),
            Token::new(
                TokenType::Ident("some_long".to_string()),
                1,
                5,
                "int some_long;".to_string(),
            ),
            Token::new(TokenType::Semicolon, 1, 14, "int some_long;".to_string()),
            Token::new(
                TokenType::While,
                2,
                1,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::LeftParen,
                2,
                7,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::Ident("val".to_string()),
                2,
                8,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::GreaterEqual,
                2,
                12,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::Number(12),
                2,
                15,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::RightParen,
                2,
                17,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::LeftBrace,
                2,
                19,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::Star,
                2,
                20,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::Ident("p".to_string()),
                2,
                21,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::Equal,
                2,
                23,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::Ident("val".to_string()),
                2,
                25,
                "while (val >= 12) {*p = val}".to_string(),
            ),
            Token::new(
                TokenType::RightBrace,
                2,
                28,
                "while (val >= 12) {*p = val}".to_string(),
            ),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn detects_single_on_invalid_char() {
        let source = "int c = 0$";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(_v) => panic!(),
            Err(e) => e,
        };
        let expected = vec![Error {
            line_index: 1,
            column: 10,
            line_string: "int c = 0$".to_string(),
            msg: "Unexpected character: $".to_string(),
        }];
        assert_eq!(result, expected);
    }
    #[test]
    fn detects_mutliple_on_invalid_chars() {
        let source = "int c = 0$\n\n??? ???";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => panic!(),
            Err(e) => e,
        };
        let expected = vec![
            Error {
                line_index: 1,
                column: 10,
                line_string: "int c = 0$".to_string(),
                msg: "Unexpected character: $".to_string(),
            },
            Error {
                line_index: 3,
                column: 1,
                line_string: "??? ???".to_string(),
                msg: "Unexpected character: ???".to_string(),
            },
            Error {
                line_index: 3,
                column: 3,
                line_string: "??? ???".to_string(),
                msg: "Unexpected character: ???".to_string(),
            },
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn can_handle_non_ascii_alphabet() {
        let source = "\nint ?? = 123";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!(),
        };
        let expected = vec![
            Token::new(TokenType::Int, 2, 1, "int ?? = 123".to_string()),
            Token::new(
                TokenType::Ident("??".to_string()),
                2,
                5,
                "int ?? = 123".to_string(),
            ),
            Token::new(TokenType::Equal, 2, 8, "int ?? = 123".to_string()), // ?? len is 2 but thats fine because its the same when indexing
            Token::new(TokenType::Number(123), 2, 10, "int ?? = 123".to_string()),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn errors_on_non_ascii_non_letters() {
        let source = "\nint ?? @ = 123";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => panic!(),
            Err(e) => e,
        };
        let expected = vec![Error {
            line_index: 2,
            column: 8,
            line_string: "int ?? @ = 123".to_string(),
            msg: "Unexpected character: @".to_string(),
        }];
        assert_eq!(result, expected);
    }
    #[test]
    fn char_literal() {
        let source = "char some = '1'";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => v,
            Err(e) => panic!(),
        };
        let expected = vec![
            Token::new(TokenType::Char, 1, 1, "char some = '1'".to_string()),
            Token::new(
                TokenType::Ident("some".to_string()),
                1,
                6,
                "char some = '1'".to_string(),
            ),
            Token::new(TokenType::Equal, 1, 11, "char some = '1'".to_string()),
            Token::new(
                TokenType::CharLit('1' as i8),
                1,
                13,
                "char some = '1'".to_string(),
            ),
        ];
        assert_eq!(result, expected);
    }
    #[test]
    fn char_literal_len_greater_1() {
        let source = "char some = '12'";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => panic!(),
            Err(e) => e,
        };
        let expected = vec![Error {
            line_index: 1,
            column: 13,
            line_string: "char some = '12'".to_string(),
            msg: "char literal must contain single character".to_string(),
        }];
        assert_eq!(result, expected);
    }
    #[test]
    fn char_literal_empty() {
        let source = "char some = ''";
        let mut scanner = Scanner::new(source);
        let result = match scanner.scan_token() {
            Ok(v) => panic!(),
            Err(e) => e,
        };
        let expected = vec![Error {
            line_index: 1,
            column: 13,
            line_string: "char some = ''".to_string(),
            msg: "char literal must contain single character".to_string(),
        }];
        assert_eq!(result, expected);
    }
}
