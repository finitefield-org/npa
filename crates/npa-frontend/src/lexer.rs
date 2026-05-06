use crate::{MachineDiagnostic, Result, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Ident(String),
    Number(u64),
    Import,
    Def,
    Theorem,
    Fun,
    Forall,
    Let,
    In,
    Prop,
    Type,
    Sort,
    Open,
    Namespace,
    Notation,
    Infix,
    Infixl,
    Infixr,
    Axiom,
    Inductive,
    Dot,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    ColonEq,
    Comma,
    FatArrow,
    At,
    Hole,
    NamedHole(String),
    Eof,
    Unsupported(char),
}

pub fn lex(file_id: crate::FileId, source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((offset, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        let start = offset as u32;
        let token = match ch {
            '.' => Token {
                kind: TokenKind::Dot,
                span: Span::new(file_id, start, start + 1),
            },
            '{' => Token {
                kind: TokenKind::LBrace,
                span: Span::new(file_id, start, start + 1),
            },
            '}' => Token {
                kind: TokenKind::RBrace,
                span: Span::new(file_id, start, start + 1),
            },
            '(' => Token {
                kind: TokenKind::LParen,
                span: Span::new(file_id, start, start + 1),
            },
            ')' => Token {
                kind: TokenKind::RParen,
                span: Span::new(file_id, start, start + 1),
            },
            ',' => Token {
                kind: TokenKind::Comma,
                span: Span::new(file_id, start, start + 1),
            },
            '@' => Token {
                kind: TokenKind::At,
                span: Span::new(file_id, start, start + 1),
            },
            ':' => {
                if let Some((next_offset, '=')) = chars.peek().copied() {
                    chars.next();
                    Token {
                        kind: TokenKind::ColonEq,
                        span: Span::new(file_id, start, (next_offset + 1) as u32),
                    }
                } else {
                    Token {
                        kind: TokenKind::Colon,
                        span: Span::new(file_id, start, start + 1),
                    }
                }
            }
            '=' => {
                if let Some((next_offset, '>')) = chars.peek().copied() {
                    chars.next();
                    Token {
                        kind: TokenKind::FatArrow,
                        span: Span::new(file_id, start, (next_offset + 1) as u32),
                    }
                } else {
                    Token {
                        kind: TokenKind::Unsupported(ch),
                        span: Span::new(file_id, start, start + 1),
                    }
                }
            }
            '_' => Token {
                kind: TokenKind::Hole,
                span: Span::new(file_id, start, start + 1),
            },
            '?' => lex_named_hole(file_id, source, start, &mut chars),
            '0'..='9' => lex_number(file_id, source, start, offset, ch, &mut chars)?,
            ch if is_ident_start(ch) => lex_ident(file_id, source, start, offset, &mut chars),
            ch => Token {
                kind: TokenKind::Unsupported(ch),
                span: Span::new(file_id, start, start + ch.len_utf8() as u32),
            },
        };

        if matches!(token.kind, TokenKind::Unsupported(_)) {
            return Err(MachineDiagnostic::unsupported_syntax(
                token.span,
                "character is not part of Machine Surface syntax",
            ));
        }

        tokens.push(token);
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(file_id, source.len() as u32, source.len() as u32),
    });

    Ok(tokens)
}

fn lex_ident(
    file_id: crate::FileId,
    source: &str,
    start: u32,
    first_offset: usize,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Token {
    let mut end = first_offset;

    while let Some((offset, ch)) = chars.peek().copied() {
        if !is_ident_continue(ch) {
            break;
        }

        chars.next();
        end = offset;
    }

    let end = end
        + source[end..]
            .chars()
            .next()
            .expect("identifier has a character")
            .len_utf8();
    let text = &source[start as usize..end];
    let kind = match text {
        "import" => TokenKind::Import,
        "def" => TokenKind::Def,
        "theorem" => TokenKind::Theorem,
        "fun" => TokenKind::Fun,
        "forall" => TokenKind::Forall,
        "let" => TokenKind::Let,
        "in" => TokenKind::In,
        "Prop" => TokenKind::Prop,
        "Type" => TokenKind::Type,
        "Sort" => TokenKind::Sort,
        "open" => TokenKind::Open,
        "namespace" => TokenKind::Namespace,
        "notation" => TokenKind::Notation,
        "infix" => TokenKind::Infix,
        "infixl" => TokenKind::Infixl,
        "infixr" => TokenKind::Infixr,
        "axiom" => TokenKind::Axiom,
        "inductive" => TokenKind::Inductive,
        _ => TokenKind::Ident(text.to_owned()),
    };

    Token {
        kind,
        span: Span::new(file_id, start, end as u32),
    }
}

fn lex_number(
    file_id: crate::FileId,
    source: &str,
    start: u32,
    first_offset: usize,
    first: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Result<Token> {
    let mut end = first_offset + first.len_utf8();

    while let Some((offset, ch)) = chars.peek().copied() {
        if !ch.is_ascii_digit() {
            break;
        }

        chars.next();
        end = offset + ch.len_utf8();
    }

    let span = Span::new(file_id, start, end as u32);
    let value = source[start as usize..end]
        .parse::<u64>()
        .map_err(|_| MachineDiagnostic::parse(span, "universe level numeral is too large"))?;

    Ok(Token {
        kind: TokenKind::Number(value),
        span,
    })
}

fn lex_named_hole(
    file_id: crate::FileId,
    source: &str,
    start: u32,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Token {
    let Some((first_offset, first)) = chars.peek().copied() else {
        return Token {
            kind: TokenKind::Hole,
            span: Span::new(file_id, start, start + 1),
        };
    };

    if !is_ident_start(first) {
        return Token {
            kind: TokenKind::Hole,
            span: Span::new(file_id, start, start + 1),
        };
    }

    chars.next();
    let mut end = first_offset + first.len_utf8();

    while let Some((offset, ch)) = chars.peek().copied() {
        if !is_ident_continue(ch) {
            break;
        }

        chars.next();
        end = offset + ch.len_utf8();
    }

    Token {
        kind: TokenKind::NamedHole(source[(start as usize + 1)..end].to_owned()),
        span: Span::new(file_id, start, end as u32),
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''
}
