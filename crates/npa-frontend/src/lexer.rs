use crate::{MachineDiagnostic, Result, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Unsupported(char),
}

pub fn lex(file_id: crate::FileId, source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    for (offset, ch) in source.char_indices() {
        if !ch.is_whitespace() {
            let span = Span::new(file_id, offset as u32, (offset + ch.len_utf8()) as u32);
            tokens.push(Token {
                kind: TokenKind::Unsupported(ch),
                span,
            });
            break;
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(file_id, source.len() as u32, source.len() as u32),
    });

    if let Some(token) = tokens
        .iter()
        .find(|token| matches!(token.kind, TokenKind::Unsupported(_)))
    {
        return Err(MachineDiagnostic::unsupported_syntax(
            token.span,
            "non-empty module input is implemented in M2",
        ));
    }

    Ok(tokens)
}
