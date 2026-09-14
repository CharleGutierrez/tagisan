//! Recursive descent parser and lexer for the Gleam language.
//!
//! Hand-crafted for ultra-fast parsing, robust error recovery, and exact AST generation.

use std::fmt;
use crate::gleam::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Pub,
    Type,
    Opaque,
    Fn,
    Case,
    Let,
    Assert,
    Use,
    Import,
    As,
    Const,
    Panic,
    Todo,
    True,
    False,
    Nil,
    If,

    // Identifiers
    LowerIdent(String), // variables, functions, type variables, labels
    UpperIdent(String), // types, constructors

    // Literals
    Int(i64),
    Float(f64),
    String(String),

    // Delimiters
    LParen,        // (
    RParen,        // )
    LBrace,        // {
    RBrace,        // }
    LBracket,      // [
    RBracket,      // ]
    HashLParen,    // #(
    Comma,         // ,
    Colon,         // :
    ColonColon,    // ::
    Dot,           // .
    DotDot,        // ..
    Arrow,         // ->
    LeftArrow,     // <-
    Pipe,          // |
    PipeOp,        // |>
    Underscore,    // _
    Equal,         // =
    
    // Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    PlusDot,       // +.
    MinusDot,      // -.
    StarDot,       // *.
    SlashDot,      // /.
    EqualEqual,    // ==
    NotEqual,      // !=
    Less,          // <
    LessEqual,     // <=
    Greater,       // >
    GreaterEqual,  // >=
    AndAnd,        // &&
    OrOr,          // ||
    LtGt,          // <> (string concatenation)
    Bang,          // !

    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Pub => write!(f, "pub"),
            Token::Type => write!(f, "type"),
            Token::Opaque => write!(f, "opaque"),
            Token::Fn => write!(f, "fn"),
            Token::Case => write!(f, "case"),
            Token::Let => write!(f, "let"),
            Token::Assert => write!(f, "assert"),
            Token::Use => write!(f, "use"),
            Token::Import => write!(f, "import"),
            Token::As => write!(f, "as"),
            Token::Const => write!(f, "const"),
            Token::Panic => write!(f, "panic"),
            Token::Todo => write!(f, "todo"),
            Token::True => write!(f, "True"),
            Token::False => write!(f, "False"),
            Token::Nil => write!(f, "Nil"),
            Token::If => write!(f, "if"),
            Token::LowerIdent(s) => write!(f, "{}", s),
            Token::UpperIdent(s) => write!(f, "{}", s),
            Token::Int(i) => write!(f, "{}", i),
            Token::Float(fl) => write!(f, "{}", fl),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::HashLParen => write!(f, "#("),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::ColonColon => write!(f, "::"),
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::Arrow => write!(f, "->"),
            Token::LeftArrow => write!(f, "<-"),
            Token::Pipe => write!(f, "|"),
            Token::PipeOp => write!(f, "|>"),
            Token::Underscore => write!(f, "_"),
            Token::Equal => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::PlusDot => write!(f, "+."),
            Token::MinusDot => write!(f, "-."),
            Token::StarDot => write!(f, "*."),
            Token::SlashDot => write!(f, "/."),
            Token::EqualEqual => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::AndAnd => write!(f, "&&"),
            Token::OrOr => write!(f, "||"),
            Token::LtGt => write!(f, "<>"),
            Token::Bang => write!(f, "!"),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}

/// Token with source position
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token at {line}:{column}: expected {expected}, got {got}")]
    UnexpectedToken {
        line: usize,
        column: usize,
        expected: String,
        got: String,
    },
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Lexer error at {line}:{column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("Invalid syntax at {line}:{column}: {message}")]
    InvalidSyntax {
        line: usize,
        column: usize,
        message: String,
    },
}

// -----------------------------------------------------------------------------
// Lexer
// -----------------------------------------------------------------------------

pub struct Lexer<'a> {
    _source: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: source,
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 < self.chars.len() {
            Some(self.chars[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, ParseError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            // Whitespace
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            let start_line = self.line;
            let start_col = self.col;

            // Comments (skip `// ...`)
            if ch == '/' && self.peek_next() == Some('/') {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // String literals
            if ch == '"' {
                self.advance(); // skip opening quote
                let mut content = String::new();
                let mut closed = false;
                while let Some(c) = self.advance() {
                    if c == '"' {
                        closed = true;
                        break;
                    }
                    if c == '\\' {
                        if let Some(escaped) = self.advance() {
                            match escaped {
                                'n' => content.push('\n'),
                                't' => content.push('\t'),
                                'r' => content.push('\r'),
                                '\\' => content.push('\\'),
                                '"' => content.push('"'),
                                other => content.push(other),
                            }
                        }
                    } else {
                        content.push(c);
                    }
                }
                if !closed {
                    return Err(ParseError::LexerError {
                        line: start_line,
                        column: start_col,
                        message: "Unterminated string literal".to_string(),
                    });
                }
                tokens.push(SpannedToken {
                    token: Token::String(content),
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Tuple start `#(`
            if ch == '#' && self.peek_next() == Some('(') {
                self.advance(); // '#'
                self.advance(); // '('
                tokens.push(SpannedToken {
                    token: Token::HashLParen,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Two-character operators / delimiters
            if ch == '-' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::Arrow,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '<' && self.peek_next() == Some('-') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::LeftArrow,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '|' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::PipeOp,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '|' && self.peek_next() == Some('|') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::OrOr,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '&' && self.peek_next() == Some('&') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::AndAnd,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '=' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::EqualEqual,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '!' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::NotEqual,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '<' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::LessEqual,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '>' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::GreaterEqual,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '<' && self.peek_next() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::LtGt,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '+' && self.peek_next() == Some('.') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::PlusDot,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '-' && self.peek_next() == Some('.') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::MinusDot,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '*' && self.peek_next() == Some('.') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::StarDot,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '/' && self.peek_next() == Some('.') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::SlashDot,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == ':' && self.peek_next() == Some(':') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::ColonColon,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            if ch == '.' && self.peek_next() == Some('.') {
                self.advance();
                self.advance();
                tokens.push(SpannedToken {
                    token: Token::DotDot,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Single characters
            let single = match ch {
                '(' => Some(Token::LParen),
                ')' => Some(Token::RParen),
                '{' => Some(Token::LBrace),
                '}' => Some(Token::RBrace),
                '[' => Some(Token::LBracket),
                ']' => Some(Token::RBracket),
                ',' => Some(Token::Comma),
                ':' => Some(Token::Colon),
                '.' => Some(Token::Dot),
                '|' => Some(Token::Pipe),
                '=' => Some(Token::Equal),
                '+' => Some(Token::Plus),
                '-' => Some(Token::Minus),
                '*' => Some(Token::Star),
                '/' => Some(Token::Slash),
                '%' => Some(Token::Percent),
                '<' => Some(Token::Less),
                '>' => Some(Token::Greater),
                '!' => Some(Token::Bang),
                _ => None,
            };

            if let Some(tok) = single {
                self.advance();
                tokens.push(SpannedToken {
                    token: tok,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Numbers (integers or floats)
            if ch.is_ascii_digit() {
                let mut num_str = String::new();
                let mut is_float = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() || c == '_' {
                        if c != '_' {
                            num_str.push(c);
                        }
                        self.advance();
                    } else if c == '.' && self.peek_next().map(|n| n.is_ascii_digit()).unwrap_or(false) {
                        is_float = true;
                        num_str.push('.');
                        self.advance();
                    } else {
                        break;
                    }
                }

                if is_float {
                    let val = num_str.parse::<f64>().map_err(|e| ParseError::LexerError {
                        line: start_line,
                        column: start_col,
                        message: format!("Malformed float '{}': {}", num_str, e),
                    })?;
                    tokens.push(SpannedToken {
                        token: Token::Float(val),
                        line: start_line,
                        column: start_col,
                    });
                } else {
                    let val = num_str.parse::<i64>().map_err(|e| ParseError::LexerError {
                        line: start_line,
                        column: start_col,
                        message: format!("Malformed integer '{}': {}", num_str, e),
                    })?;
                    tokens.push(SpannedToken {
                        token: Token::Int(val),
                        line: start_line,
                        column: start_col,
                    });
                }
                continue;
            }

            // Identifiers, keywords, or discard `_`
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if ident == "_" {
                    tokens.push(SpannedToken {
                        token: Token::Underscore,
                        line: start_line,
                        column: start_col,
                    });
                    continue;
                }

                let tok = match ident.as_str() {
                    "pub" => Token::Pub,
                    "type" => Token::Type,
                    "opaque" => Token::Opaque,
                    "fn" => Token::Fn,
                    "case" => Token::Case,
                    "let" => Token::Let,
                    "assert" => Token::Assert,
                    "use" => Token::Use,
                    "import" => Token::Import,
                    "as" => Token::As,
                    "const" => Token::Const,
                    "panic" => Token::Panic,
                    "todo" => Token::Todo,
                    "True" => Token::True,
                    "False" => Token::False,
                    "Nil" => Token::Nil,
                    "if" => Token::If,
                    _ => {
                        if ident.chars().next().unwrap().is_uppercase() {
                            Token::UpperIdent(ident)
                        } else {
                            Token::LowerIdent(ident)
                        }
                    }
                };

                tokens.push(SpannedToken {
                    token: tok,
                    line: start_line,
                    column: start_col,
                });
                continue;
            }

            // Unknown character
            return Err(ParseError::LexerError {
                line: start_line,
                column: start_col,
                message: format!("Unexpected character: {:?}", ch),
            });
        }

        tokens.push(SpannedToken {
            token: Token::Eof,
            line: self.line,
            column: self.col,
        });

        Ok(tokens)
    }
}

// -----------------------------------------------------------------------------
// Parser
// -----------------------------------------------------------------------------

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn current(&self) -> &SpannedToken {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            self.tokens.last().unwrap()
        }
    }

    fn peek(&self) -> &Token {
        &self.current().token
    }

    fn advance(&mut self) -> &SpannedToken {
        let idx = self.cursor;
        if self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
        &self.tokens[idx]
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<&SpannedToken, ParseError> {
        if self.check(expected) {
            Ok(self.advance())
        } else {
            let cur = self.current();
            Err(ParseError::UnexpectedToken {
                line: cur.line,
                column: cur.column,
                expected: expected.to_string(),
                got: cur.token.to_string(),
            })
        }
    }

    fn expect_lower_ident(&mut self) -> Result<String, ParseError> {
        let cur = self.current().clone();
        match cur.token {
            Token::LowerIdent(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(ParseError::UnexpectedToken {
                line: cur.line,
                column: cur.column,
                expected: "lowercase identifier".to_string(),
                got: cur.token.to_string(),
            }),
        }
    }

    fn expect_upper_ident(&mut self) -> Result<String, ParseError> {
        let cur = self.current().clone();
        match cur.token {
            Token::UpperIdent(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(ParseError::UnexpectedToken {
                line: cur.line,
                column: cur.column,
                expected: "uppercase identifier (TypeName or Constructor)".to_string(),
                got: cur.token.to_string(),
            }),
        }
    }

    // -------------------------------------------------------------------------
    // Module Parsing
    // -------------------------------------------------------------------------

    pub fn parse_module(&mut self, name: &str) -> Result<GleamModule, ParseError> {
        let mut module = GleamModule::new(name);

        while !self.check(&Token::Eof) {
            match self.peek() {
                Token::Import => {
                    let imp = self.parse_import()?;
                    module.imports.push(imp);
                }
                Token::Pub => {
                    self.advance();
                    match self.peek() {
                        Token::Type | Token::Opaque => {
                            let type_def = self.parse_type_definition(true)?;
                            module.add_type(type_def);
                        }
                        Token::Fn => {
                            let func_def = self.parse_function(true)?;
                            module.add_function(func_def);
                        }
                        Token::Const => {
                            let const_def = self.parse_constant(true)?;
                            module.constants.push(const_def);
                        }
                        _ => {
                            let cur = self.current();
                            return Err(ParseError::UnexpectedToken {
                                line: cur.line,
                                column: cur.column,
                                expected: "'type', 'fn', or 'const' after 'pub'".to_string(),
                                got: cur.token.to_string(),
                            });
                        }
                    }
                }
                Token::Type | Token::Opaque => {
                    let type_def = self.parse_type_definition(false)?;
                    module.add_type(type_def);
                }
                Token::Fn => {
                    let func_def = self.parse_function(false)?;
                    module.add_function(func_def);
                }
                Token::Const => {
                    let const_def = self.parse_constant(false)?;
                    module.constants.push(const_def);
                }
                _ => {
                    let cur = self.current();
                    return Err(ParseError::UnexpectedToken {
                        line: cur.line,
                        column: cur.column,
                        expected: "top-level declaration (import, type, fn, const)".to_string(),
                        got: cur.token.to_string(),
                    });
                }
            }
        }

        Ok(module)
    }

    // -------------------------------------------------------------------------
    // Import Parsing
    // -------------------------------------------------------------------------

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.expect(&Token::Import)?;

        let mut path_parts = Vec::new();
        path_parts.push(self.expect_lower_ident()?);

        while self.match_token(&Token::Slash) {
            path_parts.push(self.expect_lower_ident()?);
        }

        let module_path = path_parts.join("/");
        let mut alias = None;
        let mut unqualified = Vec::new();

        if self.match_token(&Token::Dot) {
            self.expect(&Token::LBrace)?;
            while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
                // Could be `type Subject` or `send`
                if self.match_token(&Token::Type) {
                    let name = self.expect_upper_ident()?;
                    unqualified.push(format!("type {}", name));
                } else if let Ok(s) = self.expect_lower_ident() {
                    unqualified.push(s);
                } else if let Ok(s) = self.expect_upper_ident() {
                    unqualified.push(s);
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::RBrace)?;
        }

        if self.match_token(&Token::As) {
            alias = Some(self.expect_lower_ident()?);
        }

        let mut imp = Import::new(module_path);
        imp.alias = alias;
        imp.unqualified = unqualified;
        Ok(imp)
    }

    // -------------------------------------------------------------------------
    // Type Definition Parsing
    // -------------------------------------------------------------------------

    fn parse_type_definition(&mut self, is_pub: bool) -> Result<TypeDefinition, ParseError> {
        let is_opaque = self.match_token(&Token::Opaque);
        self.expect(&Token::Type)?;

        let type_name = self.expect_upper_ident()?;
        let mut parameters = Vec::new();

        // Generic parameters: `type Option(a)`
        if self.match_token(&Token::LParen) {
            while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                let param = self.expect_lower_ident()?;
                parameters.push(param);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::RParen)?;
        }

        let mut constructors = Vec::new();

        // Custom ADT body with constructors `{ Variant1(...) Variant2 }`
        if self.match_token(&Token::LBrace) {
            while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
                let constr_name = self.expect_upper_ident()?;
                let mut fields = Vec::new();

                if self.match_token(&Token::LParen) {
                    while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                        // Labeled field: `label: Type` or positional: `Type`
                        if let Token::LowerIdent(label_candidate) = self.peek().clone() {
                            if self.cursor + 1 < self.tokens.len()
                                && self.tokens[self.cursor + 1].token == Token::Colon
                            {
                                self.advance(); // label
                                self.advance(); // ':'
                                let f_type = self.parse_type_annotation()?;
                                fields.push(ConstructorField::labeled(label_candidate, f_type));
                            } else {
                                let f_type = self.parse_type_annotation()?;
                                fields.push(ConstructorField::positional(f_type));
                            }
                        } else {
                            let f_type = self.parse_type_annotation()?;
                            fields.push(ConstructorField::positional(f_type));
                        }

                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RParen)?;
                }

                constructors.push(Constructor::new(constr_name, fields));
            }
            self.expect(&Token::RBrace)?;
        } else if self.match_token(&Token::Equal) {
            // Type alias: `pub type Header = Dict(String, String)`
            let aliased_type = self.parse_type_annotation()?;
            constructors.push(Constructor::new(
                &type_name,
                vec![ConstructorField::positional(aliased_type)],
            ));
        }

        Ok(TypeDefinition {
            is_pub,
            is_opaque,
            name: type_name,
            parameters,
            constructors,
        })
    }

    // -------------------------------------------------------------------------
    // Type Annotation Parsing
    // -------------------------------------------------------------------------

    pub fn parse_type_annotation(&mut self) -> Result<GleamType, ParseError> {
        let cur = self.current().clone();
        match cur.token {
            Token::UpperIdent(name) => {
                self.advance();
                let mut module = None;
                let mut type_name = name;

                if self.match_token(&Token::Dot) {
                    module = Some(type_name);
                    type_name = self.expect_upper_ident()?;
                }

                // Built-in checks
                if module.is_none() {
                    match type_name.as_str() {
                        "Int" => return Ok(GleamType::Int),
                        "Float" => return Ok(GleamType::Float),
                        "String" => return Ok(GleamType::String),
                        "Bool" => return Ok(GleamType::Bool),
                        "Nil" => return Ok(GleamType::Nil),
                        "List" => {
                            self.expect(&Token::LParen)?;
                            let inner = self.parse_type_annotation()?;
                            self.expect(&Token::RParen)?;
                            return Ok(GleamType::list(inner));
                        }
                        _ => {}
                    }
                }

                // Generic params `Subject(Msg)`
                let mut params = Vec::new();
                if self.match_token(&Token::LParen) {
                    while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                        params.push(self.parse_type_annotation()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RParen)?;
                }

                Ok(GleamType::Custom {
                    name: type_name,
                    module,
                    params,
                })
            }
            Token::LowerIdent(generic_name) => {
                self.advance();
                // Module qualified lowercase? e.g. `actor.Next(msg, state)`
                if self.match_token(&Token::Dot) {
                    let type_name = self.expect_upper_ident()?;
                    let mut params = Vec::new();
                    if self.match_token(&Token::LParen) {
                        while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                            params.push(self.parse_type_annotation()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                        self.expect(&Token::RParen)?;
                    }
                    Ok(GleamType::Custom {
                        name: type_name,
                        module: Some(generic_name),
                        params,
                    })
                } else {
                    Ok(GleamType::Generic(generic_name))
                }
            }
            Token::HashLParen => {
                // Tuple type `#(Int, String)`
                self.advance();
                let mut elements = Vec::new();
                while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                    elements.push(self.parse_type_annotation()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(GleamType::Tuple(elements))
            }
            Token::Fn => {
                // Function type `fn(Int, String) -> Bool`
                self.advance();
                self.expect(&Token::LParen)?;
                let mut params = Vec::new();
                while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                    params.push(self.parse_type_annotation()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                self.expect(&Token::Arrow)?;
                let return_type = self.parse_type_annotation()?;
                Ok(GleamType::function(params, return_type))
            }
            _ => Err(ParseError::UnexpectedToken {
                line: cur.line,
                column: cur.column,
                expected: "type annotation".to_string(),
                got: cur.token.to_string(),
            }),
        }
    }

    // -------------------------------------------------------------------------
    // Function Parsing
    // -------------------------------------------------------------------------

    fn parse_function(&mut self, is_pub: bool) -> Result<FunctionDefinition, ParseError> {
        self.expect(&Token::Fn)?;
        let name = self.expect_lower_ident()?;

        self.expect(&Token::LParen)?;
        let mut parameters = Vec::new();

        while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
            let mut label = None;
            let mut param_name = self.expect_lower_ident()?;

            // Check if label was provided: `label name: Type`
            if let Token::LowerIdent(second_ident) = self.peek().clone() {
                label = Some(param_name);
                param_name = second_ident;
                self.advance();
            }

            let mut type_ann = None;
            if self.match_token(&Token::Colon) {
                type_ann = Some(self.parse_type_annotation()?);
            }

            let mut p = FunctionParam::new(param_name, type_ann);
            p.label = label;
            parameters.push(p);

            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RParen)?;

        let mut return_type = None;
        if self.match_token(&Token::Arrow) {
            return_type = Some(self.parse_type_annotation()?);
        }

        self.expect(&Token::LBrace)?;
        let body = self.parse_block_statements()?;
        self.expect(&Token::RBrace)?;

        Ok(FunctionDefinition {
            is_pub,
            name,
            parameters,
            return_type,
            body,
        })
    }

    // -------------------------------------------------------------------------
    // Constant Parsing
    // -------------------------------------------------------------------------

    fn parse_constant(&mut self, is_pub: bool) -> Result<ConstantDefinition, ParseError> {
        self.expect(&Token::Const)?;
        let name = self.expect_lower_ident()?;

        let mut type_annotation = None;
        if self.match_token(&Token::Colon) {
            type_annotation = Some(self.parse_type_annotation()?);
        }

        self.expect(&Token::Equal)?;
        let value = self.parse_expression()?;

        Ok(ConstantDefinition {
            is_pub,
            name,
            type_annotation,
            value,
        })
    }

    // -------------------------------------------------------------------------
    // Statement Parsing
    // -------------------------------------------------------------------------

    fn parse_block_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            if self.match_token(&Token::Let) {
                let _is_assert = self.match_token(&Token::Assert);
                let pattern = self.parse_pattern()?;
                let mut type_annotation = None;
                if self.match_token(&Token::Colon) {
                    type_annotation = Some(self.parse_type_annotation()?);
                }
                self.expect(&Token::Equal)?;
                let value = self.parse_expression()?;
                statements.push(Statement::Let {
                    pattern,
                    type_annotation,
                    value,
                });
            } else if self.match_token(&Token::Use) {
                let mut patterns = Vec::new();
                patterns.push(self.parse_pattern()?);
                while self.match_token(&Token::Comma) {
                    patterns.push(self.parse_pattern()?);
                }
                self.expect(&Token::LeftArrow)?;
                let call = self.parse_expression()?;
                statements.push(Statement::Use { patterns, call });
            } else {
                let expr = self.parse_expression()?;
                statements.push(Statement::Expression(expr));
            }
        }

        Ok(statements)
    }

    // -------------------------------------------------------------------------
    // Pattern Parsing
    // -------------------------------------------------------------------------

    pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let left = self.parse_pattern_single()?;
        if self.match_token(&Token::Pipe) {
            let right = self.parse_pattern()?;
            Ok(Pattern::Or(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_pattern_single(&mut self) -> Result<Pattern, ParseError> {
        let cur = self.current().clone();
        match cur.token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Discard(None))
            }
            Token::Int(i) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Int(i)))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Float(f)))
            }
            Token::String(s) => {
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            Token::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            Token::Nil => {
                self.advance();
                Ok(Pattern::Literal(Literal::Nil))
            }
            Token::HashLParen => {
                // Tuple pattern `#(...)`
                self.advance();
                let mut elements = Vec::new();
                while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                    elements.push(self.parse_pattern()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Pattern::Tuple(elements))
            }
            Token::LBracket => {
                // List pattern `[x, ..rest]`
                self.advance();
                let mut elements = Vec::new();
                let mut tail = None;
                while !self.check(&Token::RBracket) && !self.check(&Token::Eof) {
                    if self.match_token(&Token::DotDot) {
                        tail = Some(Box::new(self.parse_pattern()?));
                        break;
                    }
                    elements.push(self.parse_pattern()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Pattern::List { elements, tail })
            }
            Token::UpperIdent(name) => {
                self.advance();
                let mut module = None;
                let mut constr_name = name;

                if self.match_token(&Token::Dot) {
                    module = Some(constr_name);
                    constr_name = self.expect_upper_ident()?;
                }

                let mut args = Vec::new();
                if self.match_token(&Token::LParen) {
                    while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                        let mut label = None;
                        if let Token::LowerIdent(label_cand) = self.peek().clone() {
                            if self.cursor + 1 < self.tokens.len()
                                && self.tokens[self.cursor + 1].token == Token::Colon
                            {
                                self.advance(); // label
                                self.advance(); // ':'
                                label = Some(label_cand);
                            }
                        }
                        let pat = self.parse_pattern()?;
                        args.push(PatternArg::new(label, pat));

                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RParen)?;
                }

                Ok(Pattern::Constructor {
                    name: constr_name,
                    module,
                    args,
                })
            }
            Token::LowerIdent(var_name) => {
                self.advance();
                if var_name.starts_with('_') {
                    Ok(Pattern::Discard(Some(var_name)))
                } else if self.match_token(&Token::Dot) {
                    // Qualified constructor e.g. `dict.Empty`
                    let constr_name = self.expect_upper_ident()?;
                    let mut args = Vec::new();
                    if self.match_token(&Token::LParen) {
                        while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                            let pat = self.parse_pattern()?;
                            args.push(PatternArg::positional(pat));
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                        self.expect(&Token::RParen)?;
                    }
                    Ok(Pattern::Constructor {
                        name: constr_name,
                        module: Some(var_name),
                        args,
                    })
                } else {
                    Ok(Pattern::Variable(var_name))
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                line: cur.line,
                column: cur.column,
                expected: "pattern".to_string(),
                got: cur.token.to_string(),
            }),
        }
    }

    // -------------------------------------------------------------------------
    // Expression Parsing (Precedence Climbing)
    // -------------------------------------------------------------------------

    pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_or()?;
        while self.match_token(&Token::PipeOp) {
            let right = self.parse_or()?;
            expr = Expression::BinOp {
                op: BinOp::Pipe,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_and()?;
        while self.match_token(&Token::OrOr) {
            let right = self.parse_and()?;
            expr = Expression::BinOp {
                op: BinOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_equality()?;
        while self.match_token(&Token::AndAnd) {
            let right = self.parse_equality()?;
            expr = Expression::BinOp {
                op: BinOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_relational()?;
        while let Some(op) = match self.peek() {
            Token::EqualEqual => Some(BinOp::Eq),
            Token::NotEqual => Some(BinOp::NotEq),
            _ => None,
        } {
            self.advance();
            let right = self.parse_relational()?;
            expr = Expression::BinOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_relational(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_additive()?;
        while let Some(op) = match self.peek() {
            Token::Less => Some(BinOp::Lt),
            Token::LessEqual => Some(BinOp::LtEq),
            Token::Greater => Some(BinOp::Gt),
            Token::GreaterEqual => Some(BinOp::GtEq),
            _ => None,
        } {
            self.advance();
            let right = self.parse_additive()?;
            expr = Expression::BinOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_multiplicative()?;
        while let Some(op) = match self.peek() {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            Token::PlusDot => Some(BinOp::AddFloat),
            Token::MinusDot => Some(BinOp::SubFloat),
            Token::LtGt => Some(BinOp::Concat),
            _ => None,
        } {
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expression::BinOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_unary()?;
        while let Some(op) = match self.peek() {
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            Token::Percent => Some(BinOp::Mod),
            Token::StarDot => Some(BinOp::MulFloat),
            Token::SlashDot => Some(BinOp::DivFloat),
            _ => None,
        } {
            self.advance();
            let right = self.parse_unary()?;
            expr = Expression::BinOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&Token::Minus) {
            let expr = self.parse_postfix()?;
            return Ok(Expression::BinOp {
                op: BinOp::Sub,
                left: Box::new(Expression::int(0)),
                right: Box::new(expr),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::Dot) {
                // Field access `record.field` or module function `module.func`
                if let Token::LowerIdent(field_name) = self.peek().clone() {
                    self.advance();
                    expr = Expression::FieldAccess {
                        record: Box::new(expr),
                        field: field_name,
                    };
                } else if let Token::UpperIdent(name) = self.peek().clone() {
                    self.advance();
                    expr = Expression::FieldAccess {
                        record: Box::new(expr),
                        field: name,
                    };
                } else if let Token::Int(idx) = self.peek().clone() {
                    // Tuple index `t.0`
                    self.advance();
                    expr = Expression::TupleAccess {
                        tuple: Box::new(expr),
                        index: idx as usize,
                    };
                } else {
                    let cur = self.current();
                    return Err(ParseError::UnexpectedToken {
                        line: cur.line,
                        column: cur.column,
                        expected: "field name or tuple index after '.'".to_string(),
                        got: cur.token.to_string(),
                    });
                }
            } else if self.match_token(&Token::LParen) {
                // Function call `f(arg1, label: arg2)`
                let mut arguments = Vec::new();
                while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                    let mut label = None;
                    if let Token::LowerIdent(label_cand) = self.peek().clone() {
                        if self.cursor + 1 < self.tokens.len()
                            && self.tokens[self.cursor + 1].token == Token::Colon
                        {
                            self.advance(); // label
                            self.advance(); // ':'
                            label = Some(label_cand);
                        }
                    }
                    let val = self.parse_expression()?;
                    arguments.push(CallArgument { label, value: val });

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                expr = Expression::FunctionCall {
                    function: Box::new(expr),
                    arguments,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let cur = self.current().clone();
        match cur.token {
            Token::Int(i) => {
                self.advance();
                Ok(Expression::Literal(Literal::Int(i)))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(f)))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            Token::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(false)))
            }
            Token::Nil => {
                self.advance();
                Ok(Expression::Literal(Literal::Nil))
            }
            Token::HashLParen => {
                // Tuple literal `#("a", 10)`
                self.advance();
                let mut elements = Vec::new();
                while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expression::Tuple(elements))
            }
            Token::LBracket => {
                // List literal `[1, 2, 3]`
                self.advance();
                let mut elements = Vec::new();
                while !self.check(&Token::RBracket) && !self.check(&Token::Eof) {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expression::List(elements))
            }
            Token::LParen => {
                // Grouped expression `(a + b)`
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(inner)
            }
            Token::LBrace => {
                // Block `{ let x = 1; x + 2 }`
                self.advance();
                let stmts = self.parse_block_statements()?;
                self.expect(&Token::RBrace)?;
                Ok(Expression::Block(stmts))
            }
            Token::Case => {
                // Case expression `case x { Pattern -> expr }` or multi-subject `case x, y { Pattern1, Pattern2 -> expr }`
                self.advance();
                let mut subjects = vec![self.parse_expression()?];
                while self.match_token(&Token::Comma) {
                    subjects.push(self.parse_expression()?);
                }
                let subject = if subjects.len() == 1 {
                    subjects.into_iter().next().unwrap()
                } else {
                    Expression::Tuple(subjects)
                };

                self.expect(&Token::LBrace)?;
                let mut clauses = Vec::new();

                while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
                    let mut patterns = vec![self.parse_pattern()?];
                    while self.match_token(&Token::Comma) {
                        patterns.push(self.parse_pattern()?);
                    }
                    let pattern = if patterns.len() == 1 {
                        patterns.into_iter().next().unwrap()
                    } else {
                        Pattern::Tuple(patterns)
                    };

                    let mut guard = None;
                    if self.match_token(&Token::If) {
                        guard = Some(self.parse_expression()?);
                    }
                    self.expect(&Token::Arrow)?;
                    let body = self.parse_expression()?;
                    clauses.push(CaseClause {
                        pattern,
                        guard,
                        body,
                    });
                }
                self.expect(&Token::RBrace)?;

                Ok(Expression::Case {
                    subject: Box::new(subject),
                    clauses,
                })
            }
            Token::Panic => {
                self.advance();
                let mut reason = None;
                if self.match_token(&Token::As) {
                    if let Token::String(s) = self.peek().clone() {
                        self.advance();
                        reason = Some(s);
                    }
                }
                Ok(Expression::Panic(reason))
            }
            Token::Todo => {
                self.advance();
                let mut reason = None;
                if self.match_token(&Token::As) {
                    if let Token::String(s) = self.peek().clone() {
                        self.advance();
                        reason = Some(s);
                    }
                }
                Ok(Expression::Todo(reason))
            }
            Token::Fn => {
                // Anonymous function `fn(x) { x + 1 }`
                self.advance();
                self.expect(&Token::LParen)?;
                let mut parameters = Vec::new();
                while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                    let name = self.expect_lower_ident()?;
                    let mut type_ann = None;
                    if self.match_token(&Token::Colon) {
                        type_ann = Some(self.parse_type_annotation()?);
                    }
                    parameters.push(FunctionParam::new(name, type_ann));
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RParen)?;

                let mut return_type = None;
                if self.match_token(&Token::Arrow) {
                    return_type = Some(self.parse_type_annotation()?);
                }

                self.expect(&Token::LBrace)?;
                let body = self.parse_block_statements()?;
                self.expect(&Token::RBrace)?;

                Ok(Expression::AnonymousFunction {
                    parameters,
                    return_type,
                    body,
                })
            }
            Token::UpperIdent(name) => {
                self.advance();
                let mut module = None;
                let mut constr_name = name;

                if self.match_token(&Token::Dot) {
                    module = Some(constr_name);
                    constr_name = self.expect_upper_ident()?;
                }

                // Record creation `Constructor(field: value, ...)`
                if self.match_token(&Token::LParen) {
                    let mut fields = Vec::new();
                    let mut pos_idx = 0;

                    while !self.check(&Token::RParen) && !self.check(&Token::Eof) {
                        let field_name = if let Token::LowerIdent(ref label) = self.peek().clone() {
                            if self.cursor + 1 < self.tokens.len()
                                && self.tokens[self.cursor + 1].token == Token::Colon
                            {
                                self.advance(); // label
                                self.advance(); // ':'
                                label.clone()
                            } else {
                                let name = format!("_{}", pos_idx);
                                pos_idx += 1;
                                name
                            }
                        } else {
                            let name = format!("_{}", pos_idx);
                            pos_idx += 1;
                            name
                        };

                        let val = self.parse_expression()?;
                        fields.push((field_name, val));

                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RParen)?;

                    Ok(Expression::RecordCreation {
                        constructor: constr_name,
                        module,
                        fields,
                    })
                } else {
                    // Unit constructor `Reset`
                    Ok(Expression::RecordCreation {
                        constructor: constr_name,
                        module,
                        fields: Vec::new(),
                    })
                }
            }
            Token::LowerIdent(name) => {
                self.advance();
                Ok(Expression::Variable(name))
            }
            _ => Err(ParseError::UnexpectedToken {
                line: cur.line,
                column: cur.column,
                expected: "expression".to_string(),
                got: cur.token.to_string(),
            }),
        }
    }
}

/// Convenience function to parse Gleam source code into an AST module
pub fn parse_gleam_source(source: &str, module_name: &str) -> Result<GleamModule, ParseError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_module(module_name)
}
