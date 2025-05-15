//! The Lexer implementation for the GraphViz file format.

#[derive(Debug, Clone)]
pub enum Token {
    EOF,
    Identifier(String),
    GraphKW,
    NodeKW,
    EdgeKW,
    DigraphKW,
    StrictKW,
    SubgraphKW,
    Equal,
    Colon,
    Comma,
    Semicolon,
    ArrowRight,
    ArrowLine,
    OpenBracket,
    CloseBracket,
    HtmlStart,
    HtmlEnd,
    OpenBrace,
    CloseBrace,
    Error(usize),
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    pub pos: usize,
    pub ch: char,
}

impl Lexer {
    pub fn from_string(input: &str) -> Self {
        let chars = input.chars().collect();
        Lexer::new(chars)
    }

    pub fn new(input: Vec<char>) -> Self {
        let mut l = Self {
            input,
            pos: 0,
            ch: '\0',
        };
        l.read_char();
        l
    }

    pub fn print_error(&self) {
        let mut found_loc = false;
        let mut since_last_line = 0;
        let mut idx = 0;
        // Print every char in the file.
        for ch in self.input.iter() {
            print!("{}", ch);
            idx += 1;
            if idx == self.pos {
                found_loc = true;
            }
            // Go until the end of the line, but keep track how many spaces we
            // need to print.
            if *ch == '\n' {
                if found_loc {
                    println!();
                    // Subtract 1, because 'pos' points one char after the error
                    // and another one because we print a '^' marker instead of
                    // the last space.
                    for _ in 2..since_last_line {
                        print!(" ");
                    }
                    println!("^");
                    return;
                }
                since_last_line = 0;
            }
            since_last_line += 1;
        }
    }

    pub fn has_next(&self) -> bool {
        self.pos < self.input.len()
    }

    pub fn read_char(&mut self) {
        if !self.has_next() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.pos];
            self.pos += 1;
        }
    }

    pub fn skip_whitespace(&mut self) -> bool {
        let mut changed = false;
        while self.ch.is_ascii_whitespace() {
            self.read_char();
            changed = true;
        }
        changed
    }

    pub fn skip_comment(&mut self) -> bool {
        let mut changed = false;
        if self.ch != '/' {
            return changed;
        }
        self.read_char();
        changed = true;

        if self.ch == '*' {
            let mut prev = '\0';
            while self.has_next() {
                changed = true;
                self.read_char();
                if prev == '*' && self.ch == '/' {
                    self.read_char();
                    return changed;
                }
                prev = self.ch;
            }
            return changed;
        }

        if self.ch == '/' {
            while self.has_next() {
                changed = true;
                self.read_char();
                if self.ch.is_ascii_control() {
                    self.read_char();
                    return changed;
                }
            }
        }
        changed
    }

    pub fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while self.ch.is_ascii_alphanumeric() || self.ch == '_' {
            result.push(self.ch);
            self.read_char();
        }
        // exception for POINT-SIZE
        // if result == "POINT" && self.ch == '-' {
        //     result.push(self.ch);
        //     self.read_char();
        //     while self.ch.is_ascii_alphanumeric() || self.ch == '_' {
        //         result.push(self.ch);
        //         self.read_char();
        //     }
        // }
        result
    }

    pub fn read_number(&mut self) -> String {
        let mut result = String::new();
        let mut period = false;
        while self.ch.is_numeric() || self.ch == '.' {
            // Only allow one period in each number.
            if self.ch == '.' {
                if !period {
                    period = true;
                } else {
                    break;
                }
            }
            result.push(self.ch);
            self.read_char();
        }
        result
    }

    pub fn read_string(&mut self) -> Token {
        let mut result = String::new();
        println!("Reading string");
        self.read_char();
        while self.ch != '"' {
            // Handle escaping
            if self.ch == '\\' {
                // Consume the escape character.
                self.read_char();
                self.ch = match self.ch {
                    'n' => '\n',
                    'l' => '\n',
                    _ => self.ch,
                }
            } else if self.ch == '\0' {
                // Reached EOF without completing the string
                return Token::Error(self.pos);
            }
            result.push(self.ch);
            self.read_char();
        }
        println!("Finished reading string: {}", result);
        Token::Identifier(result)
    }

    pub fn next_token_html(&mut self) -> Token {
        let mut result = String::new();
        let mut bracket_balance = 1;
        loop {
            // Handle escaping
            if self.ch == '\0' {
                // Reached EOF without completing the string
                return Token::Error(self.pos);
            }
            if self.ch == '<' {
                bracket_balance += 1;
            } else if self.ch == '>' {
                if bracket_balance == 1 {
                    break;
                }
                bracket_balance -= 1;
            }
            result.push(self.ch);
            self.read_char();
        }
        Token::Identifier(result)
    }

    pub fn next_token(&mut self) -> Token {
        let tok: Token;
        while self.skip_comment() || self.skip_whitespace() {}
        match self.ch {
            '=' => {
                tok = Token::Equal;
            }
            ';' => {
                tok = Token::Semicolon;
            }
            ':' => {
                tok = Token::Colon;
            }
            '[' => {
                tok = Token::OpenBracket;
            }
            ']' => {
                tok = Token::CloseBracket;
            }
            '{' => {
                tok = Token::OpenBrace;
            }
            '}' => {
                tok = Token::CloseBrace;
            }
            ',' => {
                tok = Token::Comma;
            }
            '"' => {
                tok = self.read_string();
            }
            '<' => {
                tok = Token::HtmlStart;
            }
            '>' => {
                tok = Token::HtmlEnd;
            }
            '-' => {
                self.read_char();
                match self.ch {
                    '>' => {
                        tok = Token::ArrowRight;
                    }
                    '-' => {
                        tok = Token::ArrowLine;
                    }
                    _ => {
                        if self.ch.is_ascii_digit() {
                            let mut num = String::new();
                            let res = self.read_number();
                            num.push('-');
                            num.push_str(&res[..]);
                            tok = Token::Identifier(num);
                        } else {
                            tok = Token::Error(self.pos);
                        }
                    }
                }
            }
            '\0' => {
                tok = Token::EOF;
            }
            _ => {
                if self.ch.is_ascii_alphabetic() {
                    let name = self.read_identifier();
                    match name.as_str() {
                        "graph" => {
                            return Token::GraphKW;
                        }
                        "node" => {
                            return Token::NodeKW;
                        }
                        "edge" => {
                            return Token::EdgeKW;
                        }
                        "digraph" => {
                            return Token::DigraphKW;
                        }
                        "strict" => {
                            return Token::StrictKW;
                        }
                        "subgraph" => {
                            return Token::SubgraphKW;
                        }
                        _ => {
                            return Token::Identifier(name);
                        }
                    }
                }
                if self.ch.is_ascii_digit() {
                    let num = self.read_number();
                    return Token::Identifier(num);
                }

                return Token::Error(self.pos);
            }
        }
        self.read_char();
        tok
    }
}
