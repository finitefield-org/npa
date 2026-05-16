use crate::{
    lex, parse_machine_term, FileId, MachineLevel, MachineTerm, MachineTermAst,
    MachineTermSourceCanonical, Result, Span, Token,
};
use sha2::{Digest, Sha256};

const TERM_SOURCE_TAG: &str = "npa.phase3.machine-term-source.v1";
const MAX_CANONICAL_STRING_LEN: usize = 1 << 20;
const MAX_CANONICAL_LIST_LEN: usize = 100_000;
const MAX_CANONICAL_NODES: usize = 100_000;
const MAX_CANONICAL_DEPTH: usize = 256;

pub type MachineSurfaceToken = Token;

pub fn canonicalize_machine_term_source(source: &str) -> Result<MachineTermSourceCanonical> {
    let term = parse_machine_term(FileId(0), source)?;
    let mut canonical_bytes = Vec::new();
    encode_string_to(&mut canonical_bytes, TERM_SOURCE_TAG);
    encode_term_to(&mut canonical_bytes, &term);
    let canonical_hash = hash_bytes(&canonical_bytes);

    Ok(MachineTermSourceCanonical {
        source: source.to_owned(),
        canonical_bytes,
        canonical_hash,
    })
}

pub fn lex_machine_surface_tokens(source: &str) -> Result<Vec<MachineSurfaceToken>> {
    lex(FileId(0), source)
}

pub fn decode_machine_term_source_canonical(canonical_bytes: &[u8]) -> Result<MachineTermAst> {
    let mut decoder = Decoder::new(canonical_bytes);
    let tag = decoder.string()?;
    if tag != TERM_SOURCE_TAG {
        return Err(crate::MachineDiagnostic::parse(
            Span::empty(FileId(0)),
            "unexpected Machine Surface term-source canonical tag",
        ));
    }
    let term = decoder.term()?;
    decoder.finish()?;
    Ok(MachineTermAst { term })
}

fn hash_bytes(bytes: &[u8]) -> npa_cert::Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

fn is_machine_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'')
}

fn is_machine_keyword(value: &str) -> bool {
    matches!(
        value,
        "import"
            | "def"
            | "theorem"
            | "fun"
            | "forall"
            | "let"
            | "in"
            | "Prop"
            | "Type"
            | "Sort"
            | "open"
            | "namespace"
            | "notation"
            | "infix"
            | "infixl"
            | "infixr"
            | "axiom"
            | "inductive"
    )
}

fn is_level_operator(value: &str) -> bool {
    matches!(value, "succ" | "max" | "imax")
}

fn encode_term_to(out: &mut Vec<u8>, term: &MachineTerm) {
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            ..
        } => {
            out.push(0x00);
            out.push(u8::from(*explicit_mode));
            encode_name_to(out, &name.parts);
            encode_option_levels_to(out, universe_args.as_deref());
        }
        MachineTerm::Local { .. } => {
            unreachable!("canonical Machine Surface source is encoded before local resolution")
        }
        MachineTerm::Sort { level, .. } => {
            out.push(0x02);
            encode_machine_level_to(out, level);
        }
        MachineTerm::App { func, arg, .. } => {
            out.push(0x03);
            encode_term_to(out, func);
            encode_term_to(out, arg);
        }
        MachineTerm::Lam { binders, body, .. } => {
            out.push(0x04);
            encode_binders_to(out, binders);
            encode_term_to(out, body);
        }
        MachineTerm::Pi { binders, body, .. } => {
            out.push(0x05);
            encode_binders_to(out, binders);
            encode_term_to(out, body);
        }
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            ..
        } => {
            out.push(0x06);
            encode_string_to(out, name);
            encode_term_to(out, ty);
            encode_term_to(out, value);
            encode_term_to(out, body);
        }
        MachineTerm::Annot { expr, ty, .. } => {
            out.push(0x07);
            encode_term_to(out, expr);
            encode_term_to(out, ty);
        }
    }
}

fn encode_binders_to(out: &mut Vec<u8>, binders: &[crate::MachineBinder]) {
    encode_uvar_to(out, binders.len() as u64);
    for binder in binders {
        encode_string_to(out, &binder.name);
        encode_term_to(out, &binder.ty);
    }
}

fn encode_option_levels_to(out: &mut Vec<u8>, levels: Option<&[MachineLevel]>) {
    match levels {
        Some(levels) => {
            out.push(0x01);
            encode_uvar_to(out, levels.len() as u64);
            for level in levels {
                encode_machine_level_to(out, level);
            }
        }
        None => out.push(0x00),
    }
}

fn encode_machine_level_to(out: &mut Vec<u8>, level: &MachineLevel) {
    match level {
        MachineLevel::Nat { value, .. } => {
            out.push(0x00);
            encode_uvar_to(out, *value);
        }
        MachineLevel::Param { name, .. } => {
            out.push(0x01);
            encode_string_to(out, name);
        }
        MachineLevel::Succ { level, .. } => {
            out.push(0x02);
            encode_machine_level_to(out, level);
        }
        MachineLevel::Max { lhs, rhs, .. } => {
            out.push(0x03);
            encode_machine_level_to(out, lhs);
            encode_machine_level_to(out, rhs);
        }
        MachineLevel::IMax { lhs, rhs, .. } => {
            out.push(0x04);
            encode_machine_level_to(out, lhs);
            encode_machine_level_to(out, rhs);
        }
    }
}

fn encode_name_to(out: &mut Vec<u8>, parts: &[String]) {
    encode_uvar_to(out, parts.len() as u64);
    for part in parts {
        encode_string_to(out, part);
    }
}

fn encode_string_to(out: &mut Vec<u8>, value: &str) {
    encode_uvar_to(out, value.len() as u64);
    out.extend(value.as_bytes());
}

fn encode_uvar_to(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
    remaining_nodes: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            remaining_nodes: MAX_CANONICAL_NODES,
        }
    }

    fn finish(&self) -> Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(crate::MachineDiagnostic::parse(
                Span::empty(FileId(0)),
                "trailing bytes in Machine Surface term-source canonical bytes",
            ))
        }
    }

    fn term(&mut self) -> Result<MachineTerm> {
        self.term_at_depth(0)
    }

    fn term_at_depth(&mut self, depth: usize) -> Result<MachineTerm> {
        self.ensure_depth(depth)?;
        self.consume_node()?;
        let tag = self.byte()?;
        let span = Span::empty(FileId(0));
        match tag {
            0x00 => {
                let explicit_mode = match self.byte()? {
                    0x00 => false,
                    0x01 => true,
                    _ => return Err(self.decode_error("invalid explicit-mode byte")),
                };
                let name = crate::MachineName::new(self.name()?, span);
                let universe_args = self.option_levels(self.child_depth(depth)?)?;
                Ok(MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span,
                })
            }
            0x01 => {
                Err(self.decode_error("local term tag is not canonical Machine Surface source"))
            }
            0x02 => Ok(MachineTerm::Sort {
                level: self.machine_level_at_depth(self.child_depth(depth)?)?,
                span,
            }),
            0x03 => Ok(MachineTerm::App {
                func: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                arg: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            0x04 => Ok(MachineTerm::Lam {
                binders: self.binders(self.child_depth(depth)?)?,
                body: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            0x05 => Ok(MachineTerm::Pi {
                binders: self.binders(self.child_depth(depth)?)?,
                body: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            0x06 => Ok(MachineTerm::Let {
                name: self.identifier("let name")?,
                ty: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                value: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                body: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            0x07 => Ok(MachineTerm::Annot {
                expr: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                ty: Box::new(self.term_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            _ => Err(self.decode_error("unknown Machine Surface term canonical tag")),
        }
    }

    fn binders(&mut self, depth: usize) -> Result<Vec<crate::MachineBinder>> {
        let len = self.usize()?;
        if len == 0 {
            return Err(self.decode_error("empty canonical binder list"));
        }
        self.ensure_list_len(len, "binder list")?;
        let mut binders = Vec::with_capacity(len);
        let span = Span::empty(FileId(0));
        for _ in 0..len {
            binders.push(crate::MachineBinder {
                name: self.identifier("binder name")?,
                ty: self.term_at_depth(depth)?,
                span,
            });
        }
        Ok(binders)
    }

    fn option_levels(&mut self, depth: usize) -> Result<Option<Vec<MachineLevel>>> {
        match self.byte()? {
            0x00 => Ok(None),
            0x01 => {
                let len = self.usize()?;
                if len == 0 {
                    return Err(self.decode_error("empty explicit universe argument list"));
                }
                self.ensure_list_len(len, "explicit universe argument list")?;
                let mut levels = Vec::with_capacity(len);
                for _ in 0..len {
                    levels.push(self.machine_level_at_depth(depth)?);
                }
                Ok(Some(levels))
            }
            _ => Err(self.decode_error("invalid optional level-list tag")),
        }
    }

    fn machine_level_at_depth(&mut self, depth: usize) -> Result<MachineLevel> {
        self.ensure_depth(depth)?;
        self.consume_node()?;
        let tag = self.byte()?;
        let span = Span::empty(FileId(0));
        match tag {
            0x00 => Ok(MachineLevel::Nat {
                value: self.uvar()?,
                span,
            }),
            0x01 => Ok(MachineLevel::Param {
                name: self.level_param_identifier()?,
                span,
            }),
            0x02 => Ok(MachineLevel::Succ {
                level: Box::new(self.machine_level_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            0x03 => Ok(MachineLevel::Max {
                lhs: Box::new(self.machine_level_at_depth(self.child_depth(depth)?)?),
                rhs: Box::new(self.machine_level_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            0x04 => Ok(MachineLevel::IMax {
                lhs: Box::new(self.machine_level_at_depth(self.child_depth(depth)?)?),
                rhs: Box::new(self.machine_level_at_depth(self.child_depth(depth)?)?),
                span,
            }),
            _ => Err(self.decode_error("unknown Machine Surface level canonical tag")),
        }
    }

    fn name(&mut self) -> Result<Vec<String>> {
        let len = self.usize()?;
        if len == 0 {
            return Err(self.decode_error("empty canonical name"));
        }
        self.ensure_list_len(len, "name component list")?;
        let mut parts = Vec::with_capacity(len);
        for _ in 0..len {
            parts.push(self.identifier("name component")?);
        }
        Ok(parts)
    }

    fn identifier(&mut self, what: &'static str) -> Result<String> {
        let value = self.string()?;
        if !is_machine_identifier(&value) || is_machine_keyword(&value) {
            return Err(self.decode_error(format!("invalid canonical {what}")));
        }
        Ok(value)
    }

    fn level_param_identifier(&mut self) -> Result<String> {
        let value = self.identifier("universe level parameter")?;
        if is_level_operator(&value) {
            return Err(self.decode_error("invalid canonical universe level parameter"));
        }
        Ok(value)
    }

    fn string(&mut self) -> Result<String> {
        let len = self.usize()?;
        if len > MAX_CANONICAL_STRING_LEN {
            return Err(self.decode_error("canonical string is too large"));
        }
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| self.decode_error("invalid UTF-8 string"))
    }

    fn usize(&mut self) -> Result<usize> {
        usize::try_from(self.uvar()?).map_err(|_| self.decode_error("length is too large"))
    }

    fn uvar(&mut self) -> Result<u64> {
        let start = self.offset;
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                let mut canonical = Vec::new();
                encode_uvar_to(&mut canonical, value);
                if canonical != self.bytes[start..self.offset] {
                    return Err(self.decode_error("non-canonical unsigned integer"));
                }
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(self.decode_error("unsigned integer is too large"));
            }
        }
    }

    fn byte(&mut self) -> Result<u8> {
        let Some(byte) = self.bytes.get(self.offset).copied() else {
            return Err(self.decode_error("unexpected end of canonical bytes"));
        };
        self.offset += 1;
        Ok(byte)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| self.decode_error("length overflow"))?;
        if end > self.bytes.len() {
            return Err(self.decode_error("unexpected end of canonical bytes"));
        }
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn consume_node(&mut self) -> Result<()> {
        self.remaining_nodes = self
            .remaining_nodes
            .checked_sub(1)
            .ok_or_else(|| self.decode_error("canonical term is too large"))?;
        Ok(())
    }

    fn ensure_depth(&self, depth: usize) -> Result<()> {
        if depth > MAX_CANONICAL_DEPTH {
            return Err(self.decode_error("canonical term nesting is too deep"));
        }
        Ok(())
    }

    fn child_depth(&self, depth: usize) -> Result<usize> {
        depth
            .checked_add(1)
            .ok_or_else(|| self.decode_error("canonical term nesting is too deep"))
    }

    fn ensure_list_len(&self, len: usize, what: &'static str) -> Result<()> {
        if len > MAX_CANONICAL_LIST_LEN {
            return Err(self.decode_error(format!("canonical {what} is too large")));
        }
        Ok(())
    }

    fn decode_error(&self, message: impl Into<String>) -> crate::MachineDiagnostic {
        crate::MachineDiagnostic::parse(Span::empty(FileId(0)), message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_bytes_with_term(body: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_string_to(&mut bytes, TERM_SOURCE_TAG);
        body(&mut bytes);
        bytes
    }

    fn encode_simple_ident(bytes: &mut Vec<u8>, name: &str) {
        bytes.push(0x00);
        bytes.push(0x00);
        encode_name_to(bytes, &[name.to_owned()]);
        bytes.push(0x00);
    }

    #[test]
    fn canonical_term_source_ignores_whitespace_and_spans() {
        let first = canonicalize_machine_term_source("@Eq.refl.{1} Nat n")
            .expect("term should canonicalize");
        let second = canonicalize_machine_term_source("  @Eq.refl.{1}\n Nat   n  ")
            .expect("term should canonicalize");

        assert_eq!(first.canonical_bytes, second.canonical_bytes);
        assert_eq!(first.canonical_hash, second.canonical_hash);
    }

    #[test]
    fn canonical_term_source_round_trips_through_ast() {
        let canonical = canonicalize_machine_term_source("@Eq.refl.{1} Nat n")
            .expect("term should canonicalize");
        let ast = decode_machine_term_source_canonical(&canonical.canonical_bytes)
            .expect("canonical bytes should decode");

        let mut bytes = Vec::new();
        encode_string_to(&mut bytes, TERM_SOURCE_TAG);
        encode_term_to(&mut bytes, &ast.term);
        assert_eq!(bytes, canonical.canonical_bytes);
    }

    #[test]
    fn decoder_rejects_empty_canonical_name() {
        let bytes = canonical_bytes_with_term(|bytes| {
            bytes.push(0x00);
            bytes.push(0x00);
            encode_uvar_to(bytes, 0);
            bytes.push(0x00);
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("empty names cannot be produced by the parser");
    }

    #[test]
    fn decoder_rejects_oversized_canonical_lists_before_allocation() {
        let bytes = canonical_bytes_with_term(|bytes| {
            bytes.push(0x00);
            bytes.push(0x00);
            encode_uvar_to(bytes, MAX_CANONICAL_LIST_LEN as u64 + 1);
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("oversized canonical lists should be rejected before allocation");
    }

    #[test]
    fn decoder_rejects_empty_or_keyword_identifiers() {
        let empty_name_component = canonical_bytes_with_term(|bytes| {
            bytes.push(0x00);
            bytes.push(0x00);
            encode_uvar_to(bytes, 1);
            encode_string_to(bytes, "");
            bytes.push(0x00);
        });
        decode_machine_term_source_canonical(&empty_name_component)
            .expect_err("empty name components cannot be produced by the parser");

        let keyword_name_component = canonical_bytes_with_term(|bytes| {
            bytes.push(0x00);
            bytes.push(0x00);
            encode_uvar_to(bytes, 1);
            encode_string_to(bytes, "let");
            bytes.push(0x00);
        });
        decode_machine_term_source_canonical(&keyword_name_component)
            .expect_err("keyword name components cannot be produced by the parser");
    }

    #[test]
    fn decoder_rejects_resolved_local_term_tag() {
        let bytes = canonical_bytes_with_term(|bytes| {
            bytes.push(0x01);
            encode_string_to(bytes, "n");
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("resolved locals cannot be produced by the parser");
    }

    #[test]
    fn decoder_rejects_empty_binder_lists() {
        let bytes = canonical_bytes_with_term(|bytes| {
            bytes.push(0x04);
            encode_uvar_to(bytes, 0);
            bytes.push(0x02);
            bytes.push(0x00);
            encode_uvar_to(bytes, 0);
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("empty lambda binder lists cannot be produced by the parser");
    }

    #[test]
    fn decoder_rejects_empty_explicit_universe_args() {
        let bytes = canonical_bytes_with_term(|bytes| {
            bytes.push(0x00);
            bytes.push(0x00);
            encode_name_to(bytes, &["Nat".to_owned()]);
            bytes.push(0x01);
            encode_uvar_to(bytes, 0);
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("empty explicit universe args cannot be produced by the parser");
    }

    #[test]
    fn decoder_rejects_level_operator_as_level_param() {
        let bytes = canonical_bytes_with_term(|bytes| {
            bytes.push(0x02);
            bytes.push(0x01);
            encode_string_to(bytes, "succ");
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("level operators cannot decode as level parameters");
    }

    #[test]
    fn decoder_rejects_excessive_canonical_depth() {
        let bytes = canonical_bytes_with_term(|bytes| {
            for _ in 0..=MAX_CANONICAL_DEPTH {
                bytes.push(0x03);
                encode_simple_ident(bytes, "f");
            }
            encode_simple_ident(bytes, "x");
        });

        decode_machine_term_source_canonical(&bytes)
            .expect_err("deep canonical terms should be rejected before overflowing the stack");
    }
}
