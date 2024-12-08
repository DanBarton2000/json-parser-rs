use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum TokenType {
    LeftBrace,
    RightBrace,
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
    buf_reader: BufReader<File>,
    tokens: Vec<Rc<Token>>,
    current_line: Option<String>,
    current_char: Option<char>,
    current_offset: usize,
    current_line_number: usize,
    start: usize,
    current_token: usize
}

impl<'a> Lexer {
    fn new(mut buf_reader: BufReader<File>) -> Lexer {
        let line = &mut "".to_string();
        buf_reader.read_line(line).expect("Failed to read first line");
        Lexer {
            buf_reader,
            tokens: vec![],
            current_line: Some(line.clone()),
            current_char: None,
            current_offset: 0,
            current_line_number: 0,
            start: 0,
            current_token: 0
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
            } else {
                self.current_char = line.chars().nth(self.current_offset);
                self.current_offset += 1;
            }
        } else {
            self.current_char = None;
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        if let Some(line) = &self.current_line {
            self.tokens.push(Rc::new(Token::new(token_type, line[self.start..self.current_offset].to_string())))
        } else {
            panic!("Tried to add token but the current line is None");
        }
    }

    fn scan_token(&mut self) {
        self.next_character();

        if let Some(c) = self.current_char {
            match c {
                '{' => { self.add_token(TokenType::LeftBrace); }
                '}' => { self.add_token(TokenType::RightBrace); }
                '\n' => { return; }
                _ => { self.add_token(TokenType::Other); }
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
        if &self.current_token >= &self.tokens.len() { return None; }

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
        if !self.match_token(TokenType::RightBrace) { return false; }
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
    let file = File::open(".\\tests\\step1\\valid.json")?;
    let reader = BufReader::new(file);
    let lexer = Lexer::new(reader);
    let mut syntax_analyser = SyntaxAnalyser::new(lexer);

    println!("{:?}", syntax_analyser.parse());

    Ok(())
}