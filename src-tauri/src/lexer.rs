use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    Integer(i64), Float(f64), String(String), Color(String), Identifier(String),
    Script, Version, On, Start, Function, Call, Var,
    If, Else, Loop, While, Break, Restart,
    Wait, Until, Return, Not, Matches, Within, Timeout, Confidence, Pause, Human,
    LeftBrace, RightBrace, LeftParen, RightParen,
    Comma, Semicolon, Colon, Dot, DotDot,
    Plus, Minus, Star, Slash, Eq, EqEq, NotEq, Lt, Gt,
    Newline, Eof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self { source: source.chars().collect(), pos: 0, line: 1, col: 1 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof { break; }
        }
        tokens
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.pos];
        self.pos += 1;
        if ch == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        ch
    }

    #[allow(dead_code)]
    fn peek(&self) -> char {
        if self.pos < self.source.len() { self.source[self.pos] } else { '\0' }
    }

    fn peek_next(&self) -> char {
        if self.pos + 1 < self.source.len() { self.source[self.pos + 1] } else { '\0' }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token { kind, line: self.line, col: self.col.saturating_sub(1) }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        if self.pos >= self.source.len() { return self.make_token(TokenKind::Eof); }
        let ch = self.source[self.pos];
        if ch == '\n' { self.advance(); return self.make_token(TokenKind::Newline); }
        if ch == '"' { return self.read_string(); }
        if ch == '#' { return self.read_color(); }
        if ch.is_ascii_digit() { return self.read_number(); }
        if ch.is_alphabetic() || ch == '_' { return self.read_identifier(); }
        if self.pos + 1 < self.source.len() {
            let next = self.source[self.pos + 1];
            match (ch, next) {
                ('.', '.') => { self.advance(); self.advance(); return self.make_token(TokenKind::DotDot); }
                ('=', '=') => { self.advance(); self.advance(); return self.make_token(TokenKind::EqEq); }
                ('!', '=') => { self.advance(); self.advance(); return self.make_token(TokenKind::NotEq); }
                _ => {}
            }
        }
        self.advance();
        let kind = match ch {
            '{' => TokenKind::LeftBrace, '}' => TokenKind::RightBrace,
            '(' => TokenKind::LeftParen, ')' => TokenKind::RightParen,
            ',' => TokenKind::Comma, ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon, '.' => TokenKind::Dot,
            '+' => TokenKind::Plus, '-' => TokenKind::Minus,
            '*' => TokenKind::Star, '/' => TokenKind::Slash,
            '=' => TokenKind::Eq, '<' => TokenKind::Lt, '>' => TokenKind::Gt,
            _ => return self.make_token(TokenKind::Identifier(format!("Unexpected: '{}'", ch))),
        };
        self.make_token(kind)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.pos >= self.source.len() { return; }
            let ch = self.source[self.pos];
            if ch == ' ' || ch == '\t' || ch == '\r' { self.advance(); }
            else if ch == '/' && self.peek_next() == '/' {
                while self.pos < self.source.len() && self.source[self.pos] != '\n' { self.advance(); }
            } else { break; }
        }
    }

    fn read_string(&mut self) -> Token {
        let (line, col) = (self.line, self.col);
        self.advance();
        let mut value = String::new();
        loop {
            if self.pos >= self.source.len() {
                return Token { kind: TokenKind::Identifier("Unterminated string".into()), line, col };
            }
            let ch = self.advance();
            if ch == '"' { break; }
            if ch == '\\' {
                match self.advance() {
                    'n' => value.push('\n'), 't' => value.push('\t'),
                    '\\' => value.push('\\'), '"' => value.push('"'),
                    c => { value.push('\\'); value.push(c); }
                }
            } else { value.push(ch); }
        }
        Token { kind: TokenKind::String(value), line, col }
    }

    fn read_color(&mut self) -> Token {
        let (line, col) = (self.line, self.col);
        self.advance();
        let mut hex = String::from("#");
        while self.pos < self.source.len() && self.source[self.pos].is_ascii_hexdigit() {
            hex.push(self.advance());
        }
        Token { kind: TokenKind::Color(hex), line, col }
    }

    fn read_number(&mut self) -> Token {
        let (line, col) = (self.line, self.col);
        let mut num_str = String::new();
        let mut is_float = false;
        while self.pos < self.source.len() {
            let ch = self.source[self.pos];
            if ch.is_ascii_digit() { num_str.push(ch); self.advance(); }
            else if ch == '.' && self.peek_next().is_ascii_digit() { is_float = true; num_str.push(ch); self.advance(); }
            else { break; }
        }
        if is_float { Token { kind: TokenKind::Float(num_str.parse().unwrap_or(0.0)), line, col } }
        else { Token { kind: TokenKind::Integer(num_str.parse().unwrap_or(0)), line, col } }
    }

    fn read_identifier(&mut self) -> Token {
        let (line, col) = (self.line, self.col);
        let mut word = String::new();
        while self.pos < self.source.len() {
            let ch = self.source[self.pos];
            if ch.is_alphanumeric() || ch == '_' { word.push(ch); self.advance(); } else { break; }
        }
        let kind = match word.as_str() {
            "script" => TokenKind::Script, "version" => TokenKind::Version,
            "on" => TokenKind::On, "start" => TokenKind::Start,
            "function" => TokenKind::Function, "call" => TokenKind::Call,
            "var" => TokenKind::Var, "if" => TokenKind::If, "else" => TokenKind::Else,
            "loop" => TokenKind::Loop, "while" => TokenKind::While,
            "break" => TokenKind::Break, "restart" => TokenKind::Restart,
            "wait" => TokenKind::Wait, "until" => TokenKind::Until,
            "return" => TokenKind::Return, "not" => TokenKind::Not,
            "matches" => TokenKind::Matches, "within" => TokenKind::Within,
            "timeout" => TokenKind::Timeout, "confidence" => TokenKind::Confidence,
            "pause" => TokenKind::Pause, "human" => TokenKind::Human,
            _ => TokenKind::Identifier(word),
        };
        Token { kind, line, col }
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    Lexer::new(source).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_script() {
        let src = "script \"Example\" {\n    version: 1\n    on start {\n        var x = 10\n    }\n}";
        let tokens = tokenize(src);
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Script));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Version));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::On));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Start));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Var));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Eof));
    }

    #[test]
    fn test_colors_and_strings() {
        let tokens = tokenize("\"hello\" #FF00AA");
        assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Color("#FF00AA".to_string()));
    }

    #[test]
    fn test_numbers_and_operators() {
        let tokens = tokenize("42 3.14 100..500 = == !=");
        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
        assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
        assert_eq!(tokens[2].kind, TokenKind::Integer(100));
        assert_eq!(tokens[3].kind, TokenKind::DotDot);
        assert_eq!(tokens[4].kind, TokenKind::Integer(500));
        assert_eq!(tokens[5].kind, TokenKind::Eq);
        assert_eq!(tokens[6].kind, TokenKind::EqEq);
        assert_eq!(tokens[7].kind, TokenKind::NotEq);
    }

    #[test]
    fn test_keywords_vs_identifiers() {
        let tokens = tokenize("if pixel var myVar");
        assert_eq!(tokens[0].kind, TokenKind::If);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("pixel".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Var);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("myVar".to_string()));
    }

    #[test]
    fn test_comments_ignored() {
        let tokens = tokenize("x = 1 // this is a comment\ny = 2");
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert!(!kinds.contains(&TokenKind::Identifier("this".to_string())));
    }

    #[test]
    fn test_pauses_and_actions() {
        let tokens = tokenize("pause 400..700 pause human key.hold mouse.click");
        assert_eq!(tokens[0].kind, TokenKind::Pause);
        assert_eq!(tokens[1].kind, TokenKind::Integer(400));
        assert_eq!(tokens[2].kind, TokenKind::DotDot);
        assert_eq!(tokens[3].kind, TokenKind::Integer(700));
        assert_eq!(tokens[4].kind, TokenKind::Pause);
        assert_eq!(tokens[5].kind, TokenKind::Human);
        assert_eq!(tokens[6].kind, TokenKind::Identifier("key".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::Dot);
        assert_eq!(tokens[8].kind, TokenKind::Identifier("hold".to_string()));
    }
}