use crate::{Diagnostic, FileId, Result, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Ident(String),
    Number(u64),
    String(String),
    Symbol(String),
    Import,
    Open,
    Namespace,
    End,
    Def,
    Theorem,
    Axiom,
    Inductive,
    Where,
    Fun,
    Forall,
    Pi,
    Let,
    In,
    Prop,
    Type,
    Sort,
    Prefix,
    Postfix,
    Infix,
    Infixl,
    Infixr,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    ColonEq,
    Arrow,
    FatArrow,
    Pipe,
    Dot,
    DotLBrace,
    At,
    Underscore,
    Question,
    Eof,
}

impl TokenKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ident(_) => "identifier",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Symbol(_) => "operator",
            Self::Import => "`import`",
            Self::Open => "`open`",
            Self::Namespace => "`namespace`",
            Self::End => "`end`",
            Self::Def => "`def`",
            Self::Theorem => "`theorem`",
            Self::Axiom => "`axiom`",
            Self::Inductive => "`inductive`",
            Self::Where => "`where`",
            Self::Fun => "`fun`",
            Self::Forall => "`forall`",
            Self::Pi => "`Pi`",
            Self::Let => "`let`",
            Self::In => "`in`",
            Self::Prop => "`Prop`",
            Self::Type => "`Type`",
            Self::Sort => "`Sort`",
            Self::Prefix => "`prefix`",
            Self::Postfix => "`postfix`",
            Self::Infix => "`infix`",
            Self::Infixl => "`infixl`",
            Self::Infixr => "`infixr`",
            Self::LParen => "`(`",
            Self::RParen => "`)`",
            Self::LBrace => "`{`",
            Self::RBrace => "`}`",
            Self::Comma => "`,`",
            Self::Colon => "`:`",
            Self::ColonEq => "`:=`",
            Self::Arrow => "`->`",
            Self::FatArrow => "`=>`",
            Self::Pipe => "`|`",
            Self::Dot => "`.`",
            Self::DotLBrace => "`.{`",
            Self::At => "`@`",
            Self::Underscore => "`_`",
            Self::Question => "`?`",
            Self::Eof => "end of file",
        }
    }
}

pub fn lex(file_id: FileId, source: &str) -> Result<Vec<Token>> {
    Lexer::new(file_id, source).lex()
}

struct Lexer<'a> {
    file_id: FileId,
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(file_id: FileId, source: &'a str) -> Self {
        Self {
            file_id,
            source,
            chars: source.char_indices().peekable(),
            tokens: Vec::new(),
        }
    }

    fn lex(mut self) -> Result<Vec<Token>> {
        while let Some((start, ch)) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
                continue;
            }

            if ch == '-' && self.next_char_is('-') {
                self.skip_line_comment();
                continue;
            }

            if is_ident_start(ch) {
                self.lex_ident_or_keyword()?;
                continue;
            }

            if ch.is_ascii_digit() {
                self.lex_number()?;
                continue;
            }

            match ch {
                '"' => self.lex_string()?,
                '(' => self.push_single(TokenKind::LParen),
                ')' => self.push_single(TokenKind::RParen),
                '{' => self.push_single(TokenKind::LBrace),
                '}' => self.push_single(TokenKind::RBrace),
                ',' => self.push_single(TokenKind::Comma),
                '|' => self.push_single(TokenKind::Pipe),
                '@' => self.push_single(TokenKind::At),
                '?' => self.push_single(TokenKind::Question),
                ':' => {
                    self.bump();
                    if self.consume_char('=') {
                        self.push(TokenKind::ColonEq, start, start + 2);
                    } else {
                        self.push(TokenKind::Colon, start, start + 1);
                    }
                }
                '=' => {
                    self.bump();
                    if self.consume_char('>') {
                        self.push(TokenKind::FatArrow, start, start + 2);
                    } else {
                        self.lex_symbol_from(start, '=');
                    }
                }
                '-' => {
                    self.bump();
                    if self.consume_char('>') {
                        self.push(TokenKind::Arrow, start, start + 2);
                    } else {
                        self.lex_symbol_from(start, '-');
                    }
                }
                '→' => {
                    self.bump();
                    self.push(TokenKind::Arrow, start, start + ch.len_utf8());
                }
                '.' => {
                    self.bump();
                    if self.consume_char('{') {
                        self.push(TokenKind::DotLBrace, start, start + 2);
                    } else {
                        self.push(TokenKind::Dot, start, start + 1);
                    }
                }
                _ if is_symbol_char(ch) => self.lex_symbol()?,
                _ => {
                    return Err(self.error(
                        start,
                        start + ch.len_utf8(),
                        format!("unexpected character `{ch}`"),
                    ));
                }
            }
        }

        let eof = self.source.len();
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.file_id, eof, eof),
        });
        Ok(self.tokens)
    }

    fn lex_ident_or_keyword(&mut self) -> Result<()> {
        let (start, first) = self.bump().expect("peeked character must exist");
        let mut end = start + first.len_utf8();
        while let Some((idx, ch)) = self.peek() {
            if is_ident_continue(ch) {
                self.bump();
                end = idx + ch.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.source[start..end];
        let kind = match text {
            "_" => TokenKind::Underscore,
            "import" => TokenKind::Import,
            "open" => TokenKind::Open,
            "namespace" => TokenKind::Namespace,
            "end" => TokenKind::End,
            "def" => TokenKind::Def,
            "theorem" => TokenKind::Theorem,
            "axiom" => TokenKind::Axiom,
            "inductive" => TokenKind::Inductive,
            "where" => TokenKind::Where,
            "fun" => TokenKind::Fun,
            "forall" => TokenKind::Forall,
            "Pi" => TokenKind::Pi,
            "let" => TokenKind::Let,
            "in" => TokenKind::In,
            "Prop" => TokenKind::Prop,
            "Type" => TokenKind::Type,
            "Sort" => TokenKind::Sort,
            "prefix" => TokenKind::Prefix,
            "postfix" => TokenKind::Postfix,
            "infix" => TokenKind::Infix,
            "infixl" => TokenKind::Infixl,
            "infixr" => TokenKind::Infixr,
            _ => TokenKind::Ident(text.to_owned()),
        };
        self.push(kind, start, end);
        Ok(())
    }

    fn lex_number(&mut self) -> Result<()> {
        let (start, first) = self.bump().expect("peeked character must exist");
        let mut end = start + first.len_utf8();
        while let Some((idx, ch)) = self.peek() {
            if ch.is_ascii_digit() {
                self.bump();
                end = idx + ch.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.source[start..end];
        let value = text
            .parse::<u64>()
            .map_err(|_| self.error(start, end, "number literal is too large"))?;
        self.push(TokenKind::Number(value), start, end);
        Ok(())
    }

    fn lex_string(&mut self) -> Result<()> {
        let (start, _) = self.bump().expect("peeked character must exist");
        let mut value = String::new();

        while let Some((idx, ch)) = self.bump() {
            match ch {
                '"' => {
                    self.push(TokenKind::String(value), start, idx + ch.len_utf8());
                    return Ok(());
                }
                '\\' => {
                    let Some((_, escaped)) = self.bump() else {
                        return Err(self.error(start, self.source.len(), "unterminated string"));
                    };
                    match escaped {
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        _ => {
                            return Err(self.error(
                                idx,
                                idx + escaped.len_utf8(),
                                format!("unknown string escape `\\{escaped}`"),
                            ));
                        }
                    }
                }
                '\n' | '\r' => {
                    return Err(self.error(start, idx + ch.len_utf8(), "unterminated string"));
                }
                _ => value.push(ch),
            }
        }

        Err(self.error(start, self.source.len(), "unterminated string"))
    }

    fn lex_symbol(&mut self) -> Result<()> {
        let (start, first) = self.bump().expect("peeked character must exist");
        self.lex_symbol_from(start, first);
        Ok(())
    }

    fn lex_symbol_from(&mut self, start: usize, first: char) {
        let mut end = start + first.len_utf8();
        while let Some((idx, ch)) = self.peek() {
            if !is_symbol_char(ch) {
                break;
            }
            if ch == '-' && self.next_char_is('-') {
                break;
            }
            self.bump();
            end = idx + ch.len_utf8();
        }
        self.push(
            TokenKind::Symbol(self.source[start..end].to_owned()),
            start,
            end,
        );
    }

    fn skip_line_comment(&mut self) {
        while let Some((_, ch)) = self.bump() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn push_single(&mut self, kind: TokenKind) {
        let (start, ch) = self.bump().expect("peeked character must exist");
        self.push(kind, start, start + ch.len_utf8());
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(self.file_id, start, end),
        });
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek().is_some_and(|(_, ch)| ch == expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn next_char_is(&mut self, expected: char) -> bool {
        let mut chars = self.chars.clone();
        chars.next();
        chars.peek().is_some_and(|(_, ch)| *ch == expected)
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn bump(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn error(&self, start: usize, end: usize, message: impl Into<String>) -> Diagnostic {
        Diagnostic::parser(Span::new(self.file_id, start, end), message)
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '\'' || ch.is_ascii_alphanumeric()
}

fn is_symbol_char(ch: char) -> bool {
    !ch.is_whitespace()
        && !is_ident_continue(ch)
        && !ch.is_ascii_digit()
        && !matches!(
            ch,
            '"' | '(' | ')' | '{' | '}' | ',' | ':' | '|' | '@' | '?' | '.'
        )
        && ch != '→'
}
