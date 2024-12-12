use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum TokenType {
    LeftBrace,
    RightBrace,
    String,
    Number,
    True,
    False,
    Null,
    Colon,
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
    Other
}

struct Token {
    token_type: TokenType,
    original_text: String,
}

impl Token {
    fn new(token_type: TokenType, original_text: String) -> Token {
        Token {
            token_type,
            original_text,
        }
    }
}

struct Lexer {
    buf_reader: Box<dyn BufRead>,
    tokens: Vec<Rc<Token>>,
    current_line: Option<String>,
    current_char: Option<char>,
    current_offset: usize,
    current_line_number: usize,
    start: usize,
    current_token: usize,
    keywords: HashMap<String, TokenType>
}

impl Lexer {
    fn new(mut buf_reader: Box<dyn BufRead>) -> Lexer {
        let line = &mut "".to_string();
        buf_reader.read_line(line).expect("Failed to read first line");

        let mut map = HashMap::new();
        map.insert("true".to_string(), TokenType::True);
        map.insert("false".to_string(), TokenType::False);
        map.insert("null".to_string(), TokenType::Null);

        Lexer {
            buf_reader,
            tokens: vec![],
            current_line: Some(line.clone()),
            current_char: None,
            current_offset: 0,
            current_line_number: 0,
            start: 0,
            current_token: 0,
            keywords: map
        }
    }

    fn next_character(&mut self) {
        if let Some(line) = &self.current_line {
            if self.current_offset >= line.chars().count() {
                let mut new_line = String::new();
                self.buf_reader.read_line(&mut new_line).expect("Failed to read line");
                self.current_line = Some(new_line.clone());
                self.current_char = Some('\n');
                self.current_line_number += 1;
                self.current_offset = 0;
            } else {
                self.current_char = line.chars().nth(self.current_offset);
                self.current_offset += 1;
            }
        } else {
            self.current_char = None;
        }
    }

    fn peek(&mut self) -> Option<char> {
        if let Some(line) = &self.current_line {
            if self.current_offset >= line.chars().count() {
                Some('\n')
            } else {
                line.chars().nth(self.current_offset)
            }
        } else {
            None
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        if let Some(line) = &self.current_line {
            self.tokens.push(Rc::new(Token::new(token_type, line[self.start..self.current_offset].to_string())))
        } else {
            panic!("Tried to add token but the current line is None");
        }
    }

    fn next_num(&mut self) {
        while let Some(_) = self.current_char {
            if self.peek().is_some_and(|x| !x.is_numeric()) { break; }
            self.next_character();
        }
    }

    fn number(&mut self) {
        self.next_num();

        if let Some(dot) = self.peek() {
            if dot == '.' {
                self.next_character();
                self.next_character();
                if let Some(n) = self.current_char {
                    if n.is_numeric() {
                        self.next_num();
                    }
                }
            }
        }

        self.add_token(TokenType::Number);
    }

    fn keyword(&mut self) {
        while let Some(_) = self.current_char {
            if self.peek().is_some_and(|x| !x.is_alphabetic()) { break; }
            self.next_character();
        }
        if let Some(line) = &self.current_line {
            if let Some(token) = self.keywords.get(&line[self.start..self.current_offset].to_string()) {
                self.add_token(token.clone());
            } else {
                self.add_token(TokenType::Other);
            }
        } else {
            self.add_token(TokenType::Other);
        }
    }

    fn scan_token(&mut self) {
        self.next_character();

        if let Some(c) = self.current_char {
            match c {
                '{' => { self.add_token(TokenType::LeftBrace); }
                '}' => { self.add_token(TokenType::RightBrace); }
                ':' => { self.add_token(TokenType::Colon); }
                ',' => { self.add_token(TokenType::Comma); }
                '[' => { self.add_token(TokenType::LeftSquareBracket); }
                ']' => { self.add_token(TokenType::RightSquareBracket); }
                '"' => {
                    self.next_character();
                    while let Some(ch) = self.current_char {
                        if ch == '"' { break; }
                        self.next_character();
                    }
                    self.add_token(TokenType::String)
                }
                '\n' | ' ' => { }
                _ => {
                    if c.is_numeric() {
                        self.number();
                    } else if c.is_alphabetic() {
                        self.keyword();
                    } else {
                        self.add_token(TokenType::Other);
                    }
                }
            }
        } else {
            panic!("Next character is none :o");
        }
    }

    fn at_end(&self) -> bool {
        if let Some(line) = &self.current_line {
            line.is_empty()
        } else {
            false
        }
    }

    fn scan_tokens(&mut self) {
        while !self.at_end() {
            self.start = self.current_offset;
            self.scan_token();
        }
    }

    fn next_token(&mut self) -> Option<Rc<Token>> {
        if self.current_token >= self.tokens.len() { return None; }

        let token = self.tokens[self.current_token].clone();
        self.current_token += 1;
        Some(token)
    }
}

struct SyntaxAnalyser {
    lexer: Lexer,
    next_token: Option<Rc<Token>>
}

impl SyntaxAnalyser {
    fn new (lexer: Lexer) -> SyntaxAnalyser {
        SyntaxAnalyser {
            lexer,
            next_token: None
        }
    }

    fn parse(&mut self) -> bool {
        self.lexer.scan_tokens();
        self.next_token = self.lexer.next_token();
        self.object()
    }

    fn object(&mut self) -> bool {
        if !self.match_token(TokenType::LeftBrace) { return false; }

        loop {
            if self.match_token(TokenType::String) {
                if !self.match_token(TokenType::Colon) { return false; }
                if !self.value() { return false; }
            }

            if !self.match_token(TokenType::Comma) { break; }
        }

        if !self.match_token(TokenType::RightBrace) { return false; }
        true
    }

    fn value(&mut self) -> bool {
        if self.match_token(TokenType::String) { return true; }
        if self.match_token(TokenType::Number) { return true; }
        if self.match_token(TokenType::True) { return true; }
        if self.match_token(TokenType::False) { return true; }
        if self.match_token(TokenType::Null) { return true; }
        if self.object() { return true; }
        if self.array() { return true; }
        false
    }

    fn array(&mut self) -> bool {
        if !self.match_token(TokenType::LeftSquareBracket) { return false; }

        if self.value() && self.match_token(TokenType::Comma) {
            loop {
                if !self.value() { return false; }
                if !self.match_token(TokenType::Comma) { break; }
            }
        }

        if !self.match_token(TokenType::RightSquareBracket) { return false; }
        true
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        match &self.next_token {
            None => { false }
            Some(token) => {
                if token.token_type == token_type {
                    self.next_token = self.lexer.next_token();
                    true
                } else {
                    false
                }
            }
        }
    }
}

fn main() -> std::io::Result<()>  {
    let args: Vec<String> = std::env::args().collect();

    let buffer: Box<dyn BufRead> = if args.len() == 1 {
        Box::new(BufReader::new(stdin()))
    } else {
        let file = File::open(&args[1])?;
        Box::new(BufReader::new(file))
    };

    let lexer = Lexer::new(buffer);
    let mut syntax_analyser = SyntaxAnalyser::new(lexer);

    let valid = syntax_analyser.parse();
    std::process::exit(!valid as i32);
}