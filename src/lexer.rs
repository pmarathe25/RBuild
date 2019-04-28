use std::str::Chars;
use std::iter::Peekable;

use crate::token::{Token, ESCAPE};

pub(crate) struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Lexer<'a> {
        return Lexer::<'a>{input: input.chars().peekable()};
    }

    fn read_char(&mut self) -> Option<char> {
        return self.input.next();
    }

    fn peek_char(&mut self) -> Option<&char> {
        return self.input.peek();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.read_char();
            } else {
                return;
            }
        }
    }

    // Reads until the closure returns true, including the last element.
    fn read_until(&mut self, until: impl Fn(char) -> bool) -> String {
        let mut ident = String::new();
        let mut prev = '\0';
        while let Some(curr) = self.read_char() {
            if prev != ESCAPE && until(curr) {
                return ident;
            }
            ident.push(curr);
            prev = curr;
        }
        return ident;
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(&curr) = self.peek_char() {
            let token: Token;
            match curr {
                // Always treat quoted strings as identifiers
                quote_kind @ '\'' | quote_kind @ '\"' => {
                    // Consume the open quote.
                    self.read_char();
                    token = Token::Ident(self.read_until(|c| c == quote_kind));
                },
                // Tags can be specified after paths to make indexing easier.
                '#' => {
                    // Consume the comment
                    self.read_until(|c| c.is_whitespace());
                    token = Token::Tag;
                }
                _ => token = Token::lookup(&self.read_until(|c| c.is_whitespace())),
            };
            return Some(token)
        }
        return None;
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        // Skip over comments.
        let mut token_opt = self.next_token();
        while let Some(Token::Tag) = token_opt {
            token_opt = self.next_token();
        };
        return token_opt;
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token};
    use std::collections::HashMap;
    use std::iter::FromIterator;

    fn make_lexer<'a>(inp: &'a str) -> Lexer<'a> {
        return Lexer::new(inp);
    }

    #[test]
    fn test_can_parse_single_keywords() {
        let keywords: HashMap<&str, Token> = HashMap::from_iter(vec![
            ("path", Token::Path),
            ("deps", Token::Deps),
            ("run", Token::Run),
        ]);
        for (raw, token) in keywords {
            let mut lexer = Lexer::new(&raw);
            assert_eq!(lexer.next(), Some(token));
        }
    }

    #[test]
    fn test_can_escape_spaces() {
        let mut lexer = make_lexer("my\\ string\\ with\\ spaces");
        assert_eq!(lexer.next(), Some(Token::Ident("my\\ string\\ with\\ spaces".to_string())));
    }

    #[test]
    fn test_single_quoted_string_is_single_token() {
        let mut lexer = make_lexer("'my quoted string with whitespaces'");
        assert_eq!(lexer.next(), Some(Token::Ident("my quoted string with whitespaces".to_string())));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_double_quoted_string_is_single_token() {
        let mut lexer = make_lexer("\"my quoted string with whitespaces\"");
        assert_eq!(lexer.next(), Some(Token::Ident("my quoted string with whitespaces".to_string())));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_quoted_keyword_is_ident() {
        let mut lexer = make_lexer("'path'");
        assert_eq!(lexer.next(), Some(Token::Ident("path".to_string())));
    }

    #[test]
    fn test_will_skip_comments() {
        let mut lexer = make_lexer("#0569");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_can_read_path_tag() {
        let mut lexer = make_lexer("path /my/test/path #0");
        assert_eq!(lexer.next(), Some(Token::Path));
        assert_eq!(lexer.next(), Some(Token::Ident("/my/test/path".to_string())));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_can_read_deps() {
        let mut lexer = make_lexer("deps 0 1 2");
        assert_eq!(lexer.next(), Some(Token::Deps));
        assert_eq!(lexer.next(), Some(Token::Num(0)));
        assert_eq!(lexer.next(), Some(Token::Num(1)));
        assert_eq!(lexer.next(), Some(Token::Num(2)));
    }

    #[test]
    fn test_can_read_run() {
        let mut lexer = make_lexer("run g++ '/my/path.cpp' '-omy output/path.o'");
        assert_eq!(lexer.next(), Some(Token::Run));
        assert_eq!(lexer.next(), Some(Token::Ident("g++".to_string())));
        assert_eq!(lexer.next(), Some(Token::Ident("/my/path.cpp".to_string())));
        assert_eq!(lexer.next(), Some(Token::Ident("-omy output/path.o".to_string())));
    }
}
