use std::collections::{BTreeMap, BTreeSet};

use crate::{
    lex, BinderInfo, Diagnostic, DiagnosticKind, FileId, ImplicitMode, NotationDecl, NotationKind,
    Result, Span, SurfaceBinder, SurfaceBinderKind, SurfaceCtorDecl, SurfaceDecl, SurfaceExpr,
    SurfaceItem, SurfaceLevel, SurfaceModule, SurfaceName, SurfaceUniverseParam, Token, TokenKind,
};

pub fn parse_module(file_id: FileId, source: &str) -> Result<SurfaceModule> {
    parse_module_with_imports(file_id, source, &[])
}

pub(crate) fn parse_module_with_imports(
    file_id: FileId,
    source: &str,
    imports: &[ParserImport],
) -> Result<SurfaceModule> {
    let tokens = lex(file_id, source)?;
    Parser::new(file_id, tokens, imports).parse_module()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NotationSyntaxEntry {
    kind: NotationKind,
    associativity: Associativity,
    precedence: u32,
    symbol: String,
}

impl NotationSyntaxEntry {
    fn from_decl(decl: &NotationDecl) -> Self {
        Self {
            kind: decl.kind.clone(),
            associativity: associativity_for_kind(&decl.kind),
            precedence: decl.precedence,
            symbol: decl.symbol.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParserImport {
    pub module: Vec<String>,
    pub notations: Vec<ParserImportedNotation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParserImportedNotation {
    pub kind: NotationKind,
    pub precedence: u32,
    pub symbol: String,
    pub namespace: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopedNotationSyntaxEntry {
    namespace: Option<Vec<String>>,
    entry: NotationSyntaxEntry,
}

impl ScopedNotationSyntaxEntry {
    fn from_imported(notation: &ParserImportedNotation) -> Self {
        Self {
            namespace: notation.namespace.clone(),
            entry: NotationSyntaxEntry {
                kind: notation.kind.clone(),
                associativity: associativity_for_kind(&notation.kind),
                precedence: notation.precedence,
                symbol: notation.symbol.clone(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
    NonAssoc,
}

struct Parser {
    file_id: FileId,
    tokens: Vec<Token>,
    pos: usize,
    namespace_stack: Vec<String>,
    open_scopes: Vec<Vec<Vec<String>>>,
    notation_scopes: Vec<Vec<NotationSyntaxEntry>>,
    namespace_notations: BTreeMap<Vec<String>, Vec<NotationSyntaxEntry>>,
    available_imports: BTreeMap<Vec<String>, Vec<ScopedNotationSyntaxEntry>>,
    imported_modules: BTreeSet<Vec<String>>,
}

impl Parser {
    fn new(file_id: FileId, tokens: Vec<Token>, imports: &[ParserImport]) -> Self {
        Self {
            file_id,
            tokens,
            pos: 0,
            namespace_stack: Vec::new(),
            open_scopes: vec![Vec::new()],
            notation_scopes: vec![Vec::new()],
            namespace_notations: BTreeMap::new(),
            available_imports: parser_import_map(imports),
            imported_modules: BTreeSet::new(),
        }
    }

    fn parse_module(&mut self) -> Result<SurfaceModule> {
        let start = self.peek().span.start;
        let mut items = Vec::new();
        let mut seen_non_import = false;

        while !self.at_eof() {
            if self.at(TokenTag::Import) && seen_non_import {
                return Err(Diagnostic::import_after_item(self.peek().span));
            }

            let item = self.parse_item()?;
            seen_non_import |= !matches!(item, SurfaceItem::Import { .. });
            self.update_notation_scope_after_item(&item)?;
            items.push(item);
        }

        let end = self.peek().span.end;
        Ok(SurfaceModule {
            file_id: self.file_id,
            items,
            span: Span::new(self.file_id, start, end),
        })
    }

    fn parse_item(&mut self) -> Result<SurfaceItem> {
        match &self.peek().kind {
            TokenKind::Import => self.parse_import(),
            TokenKind::Open => self.parse_open(),
            TokenKind::Namespace => self.parse_namespace(),
            TokenKind::End => self.parse_end(),
            TokenKind::Prefix
            | TokenKind::Postfix
            | TokenKind::Infix
            | TokenKind::Infixl
            | TokenKind::Infixr => self.parse_notation(),
            TokenKind::Def => self.parse_def(),
            TokenKind::Theorem => self.parse_theorem(),
            TokenKind::Axiom => self.parse_axiom(),
            TokenKind::Inductive => self.parse_inductive(),
            _ => Err(self.error_here(format!("expected item, found {}", self.peek().kind.label()))),
        }
    }

    fn update_notation_scope_after_item(&mut self, item: &SurfaceItem) -> Result<()> {
        match item {
            SurfaceItem::Namespace { name, span } => {
                self.namespace_stack.push(name.clone());
                self.open_scopes.push(Vec::new());
                self.notation_scopes.push(Vec::new());
                self.activate_current_namespace_notations(*span)?;
            }
            SurfaceItem::End { .. } => {
                if self.notation_scopes.len() > 1 {
                    self.notation_scopes.pop();
                }
                if self.open_scopes.len() > 1 {
                    self.open_scopes.pop();
                }
                self.namespace_stack.pop();
            }
            SurfaceItem::Open { namespace, span } => {
                self.activate_open_namespace_notations(&namespace.parts, *span)?;
            }
            SurfaceItem::Notation(decl) => {
                let entry = NotationSyntaxEntry::from_decl(decl);
                self.add_active_notation(entry.clone(), decl.span)?;
                if !self.namespace_stack.is_empty() {
                    self.namespace_notations
                        .entry(self.namespace_stack.clone())
                        .or_default()
                        .push(entry);
                }
            }
            SurfaceItem::Import { module, span } => {
                self.activate_import_notations(&module.parts, *span)?;
            }
            SurfaceItem::Def(_)
            | SurfaceItem::Theorem(_)
            | SurfaceItem::Axiom(_)
            | SurfaceItem::Inductive { .. } => {}
        }
        Ok(())
    }

    fn activate_import_notations(&mut self, module: &[String], span: Span) -> Result<()> {
        if !self.imported_modules.insert(module.to_vec()) {
            return Ok(());
        }
        let entries = self
            .available_imports
            .get(module)
            .cloned()
            .unwrap_or_default();
        for imported in entries {
            if let Some(namespace) = imported.namespace {
                self.namespace_notations
                    .entry(namespace)
                    .or_default()
                    .push(imported.entry);
            } else {
                self.add_active_notation(imported.entry, span)?;
            }
        }
        Ok(())
    }

    fn activate_current_namespace_notations(&mut self, span: Span) -> Result<()> {
        let namespace = self.namespace_stack.clone();
        if namespace.is_empty() {
            return Ok(());
        }
        for end in 1..=namespace.len() {
            self.activate_namespace_notations(&namespace[..end], span)?;
        }
        Ok(())
    }

    fn activate_namespace_notations(&mut self, namespace: &[String], span: Span) -> Result<()> {
        let entries = self
            .namespace_notations
            .get(namespace)
            .cloned()
            .unwrap_or_default();
        for entry in entries {
            self.add_active_notation(entry, span)?;
        }
        Ok(())
    }

    fn activate_open_namespace_notations(
        &mut self,
        namespace: &[String],
        span: Span,
    ) -> Result<()> {
        let mut candidates = vec![namespace.to_vec()];
        if !self.namespace_stack.is_empty() {
            let mut relative = self.namespace_stack.clone();
            relative.extend_from_slice(namespace);
            candidates.push(relative);
        }
        candidates.extend(self.opened_namespaces().into_iter().map(|mut opened| {
            opened.extend_from_slice(namespace);
            opened
        }));
        candidates.sort();
        candidates.dedup();

        for candidate in &candidates {
            self.activate_namespace_notations(candidate, span)?;
        }
        self.open_scopes
            .last_mut()
            .expect("top-level open scope exists")
            .extend(candidates);
        Ok(())
    }

    fn opened_namespaces(&self) -> Vec<Vec<String>> {
        self.open_scopes
            .iter()
            .flat_map(|scope| scope.iter().cloned())
            .collect()
    }

    fn add_active_notation(&mut self, entry: NotationSyntaxEntry, span: Span) -> Result<()> {
        for active in self.active_notation_entries() {
            if active.symbol == entry.symbol
                && (active.kind != entry.kind
                    || active.precedence != entry.precedence
                    || active.associativity != entry.associativity)
            {
                return Err(Diagnostic::error(
                    DiagnosticKind::NotationConflict,
                    span,
                    format!("conflicting notation declaration for `{}`", entry.symbol),
                ));
            }
        }
        self.notation_scopes
            .last_mut()
            .expect("top-level notation scope exists")
            .push(entry);
        Ok(())
    }

    fn parse_import(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Import, "`import`")?.span;
        let module = self.parse_qual_name()?;
        Ok(SurfaceItem::Import {
            span: start.join(module.span),
            module,
        })
    }

    fn parse_open(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Open, "`open`")?.span;
        let namespace = self.parse_qual_name()?;
        Ok(SurfaceItem::Open {
            span: start.join(namespace.span),
            namespace,
        })
    }

    fn parse_namespace(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Namespace, "`namespace`")?.span;
        let (name, name_span) = self.parse_ident()?;
        Ok(SurfaceItem::Namespace {
            name,
            span: start.join(name_span),
        })
    }

    fn parse_end(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::End, "`end`")?.span;
        let (name, span) = if self.peek_is_ident() {
            let (name, name_span) = self.parse_ident()?;
            (Some(name), start.join(name_span))
        } else {
            (None, start)
        };
        Ok(SurfaceItem::End { name, span })
    }

    fn parse_notation(&mut self) -> Result<SurfaceItem> {
        let (kind, start) = match &self.peek().kind {
            TokenKind::Prefix => (NotationKind::Prefix, self.bump().span),
            TokenKind::Postfix => (NotationKind::Postfix, self.bump().span),
            TokenKind::Infix => (NotationKind::Infix, self.bump().span),
            TokenKind::Infixl => (NotationKind::Infixl, self.bump().span),
            TokenKind::Infixr => (NotationKind::Infixr, self.bump().span),
            _ => return Err(self.error_here("expected notation declaration")),
        };

        self.expect(TokenTag::Colon, "`:`")?;
        let precedence = self.parse_number()?;
        let (raw_symbol, symbol_span) = self.parse_string()?;
        let symbol = normalize_notation_symbol(&raw_symbol, symbol_span)?;
        self.expect(TokenTag::FatArrow, "`=>`")?;
        let target = self.parse_qual_name()?;
        Ok(SurfaceItem::Notation(NotationDecl {
            kind,
            precedence,
            symbol,
            span: start.join(target.span),
            target,
        }))
    }

    fn parse_def(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Def, "`def`")?.span;
        let decl = self.parse_value_decl(start, true)?;
        Ok(SurfaceItem::Def(decl))
    }

    fn parse_theorem(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Theorem, "`theorem`")?.span;
        let decl = self.parse_value_decl(start, true)?;
        Ok(SurfaceItem::Theorem(decl))
    }

    fn parse_axiom(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Axiom, "`axiom`")?.span;
        let decl = self.parse_value_decl(start, false)?;
        Ok(SurfaceItem::Axiom(decl))
    }

    fn parse_value_decl(&mut self, start: Span, has_value: bool) -> Result<SurfaceDecl> {
        let (name, _) = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let binders = self.parse_decl_binders()?;
        self.expect(TokenTag::Colon, "`:`")?;
        let ty = self.parse_term()?;
        let value = if has_value {
            self.expect(TokenTag::ColonEq, "`:=`")?;
            Some(self.parse_term()?)
        } else {
            None
        };
        let end = value.as_ref().map_or(ty.span(), SurfaceExpr::span);
        Ok(SurfaceDecl {
            name,
            universe_params,
            binders,
            ty,
            value,
            span: start.join(end),
        })
    }

    fn parse_inductive(&mut self) -> Result<SurfaceItem> {
        let start = self.expect(TokenTag::Inductive, "`inductive`")?.span;
        let (name, _) = self.parse_ident()?;
        let universe_params = self.parse_universe_params()?;
        let binders = self.parse_decl_binders()?;
        self.expect(TokenTag::Colon, "`:`")?;
        let ty = self.parse_term()?;
        self.expect(TokenTag::Where, "`where`")?;

        let mut constructors = Vec::new();
        while self.at(TokenTag::Pipe) {
            constructors.push(self.parse_ctor()?);
        }
        if constructors.is_empty() {
            return Err(self.error_here("expected at least one constructor"));
        }

        let end = constructors.last().map_or(ty.span(), |ctor| ctor.span);
        Ok(SurfaceItem::Inductive {
            name,
            universe_params,
            binders,
            ty,
            constructors,
            span: start.join(end),
        })
    }

    fn parse_ctor(&mut self) -> Result<SurfaceCtorDecl> {
        let start = self.expect(TokenTag::Pipe, "`|`")?.span;
        let (name, _) = self.parse_ident()?;
        self.expect(TokenTag::Colon, "`:`")?;
        let ty = self.parse_term()?;
        Ok(SurfaceCtorDecl {
            name,
            span: start.join(ty.span()),
            ty,
        })
    }

    fn parse_term(&mut self) -> Result<SurfaceExpr> {
        self.parse_term_with_annotations(true)
    }

    fn parse_term_with_annotations(&mut self, allow_annotations: bool) -> Result<SurfaceExpr> {
        if self.at(TokenTag::Fun) {
            return self.parse_lambda();
        }
        if self.at(TokenTag::Forall) || self.at(TokenTag::Pi) {
            return self.parse_forall();
        }
        if self.at(TokenTag::Let) {
            return self.parse_let();
        }
        self.parse_arrow(allow_annotations)
    }

    fn parse_lambda(&mut self) -> Result<SurfaceExpr> {
        let start = self.expect(TokenTag::Fun, "`fun`")?.span;
        let mut binders = Vec::new();
        while !self.at(TokenTag::FatArrow) {
            if self.at_eof() {
                return Err(self.error_here("expected `=>`"));
            }
            binders.extend(self.parse_lambda_binder()?);
        }
        self.expect(TokenTag::FatArrow, "`=>`")?;
        if binders.is_empty() {
            return Err(Diagnostic::parser(
                start,
                "lambda must bind at least one name",
            ));
        }
        let body = self.parse_term()?;
        Ok(SurfaceExpr::Lam {
            span: start.join(body.span()),
            binders,
            body: Box::new(body),
        })
    }

    fn parse_forall(&mut self) -> Result<SurfaceExpr> {
        let start = self.bump().span;
        let mut binders = Vec::new();
        while !self.at(TokenTag::Comma) {
            if self.at_eof() {
                return Err(self.error_here("expected `,`"));
            }
            binders.extend(self.parse_decl_binder_group()?);
        }
        self.expect(TokenTag::Comma, "`,`")?;
        if binders.is_empty() {
            return Err(Diagnostic::parser(
                start,
                "forall must bind at least one name",
            ));
        }
        let body = self.parse_term()?;
        Ok(SurfaceExpr::Pi {
            span: start.join(body.span()),
            binders,
            body: Box::new(body),
        })
    }

    fn parse_let(&mut self) -> Result<SurfaceExpr> {
        let start = self.expect(TokenTag::Let, "`let`")?.span;
        let (name, name_span) = self.parse_ident()?;
        let name = SurfaceName::single(name, name_span);
        let ty = if self.at(TokenTag::Colon) {
            self.bump();
            Some(Box::new(self.parse_term()?))
        } else {
            None
        };
        self.expect(TokenTag::ColonEq, "`:=`")?;
        let value = self.parse_term()?;
        self.expect(TokenTag::In, "`in`")?;
        let body = self.parse_term()?;
        Ok(SurfaceExpr::Let {
            span: start.join(body.span()),
            name,
            ty,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_arrow(&mut self, allow_annotations: bool) -> Result<SurfaceExpr> {
        let lhs = self.parse_annotation(allow_annotations)?;
        if self.at(TokenTag::Arrow) {
            self.bump();
            let rhs = self.parse_term_with_annotations(allow_annotations)?;
            let span = lhs.span().join(rhs.span());
            let binder = SurfaceBinder {
                kind: SurfaceBinderKind::Anonymous,
                ty: Some(Box::new(lhs)),
                binder_info: BinderInfo::Explicit,
                span,
            };
            Ok(SurfaceExpr::Pi {
                binders: vec![binder],
                body: Box::new(rhs),
                span,
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_annotation(&mut self, allow_annotations: bool) -> Result<SurfaceExpr> {
        let expr = self.parse_application()?;
        if !self.at(TokenTag::Colon) {
            return Ok(expr);
        }
        if !allow_annotations {
            return Err(self.error_here("type annotation is non-associative"));
        }

        self.bump();
        let ty = self.parse_arrow(false)?;
        let span = expr.span().join(ty.span());
        if self.at(TokenTag::Colon) {
            return Err(self.error_here("type annotation is non-associative"));
        }
        Ok(SurfaceExpr::Annot {
            expr: Box::new(expr),
            ty: Box::new(ty),
            span,
        })
    }

    fn parse_application(&mut self) -> Result<SurfaceExpr> {
        self.parse_notation_expr(0)
    }

    fn parse_notation_expr(&mut self, min_bp: u32) -> Result<SurfaceExpr> {
        let mut expr = self.parse_notation_prefix_or_application()?;
        loop {
            if let Some(entry) = self.active_postfix_entry() {
                if entry.precedence < min_bp {
                    break;
                }
                let token = self.bump();
                let span = expr.span().join(token.span);
                expr = SurfaceExpr::Notation {
                    head: crate::NotationHead {
                        kind: NotationKind::Postfix,
                        symbol: entry.symbol,
                        span: token.span,
                    },
                    args: vec![expr],
                    span,
                };
                continue;
            }

            let Some(entry) = self.active_infix_entry() else {
                break;
            };
            if entry.precedence < min_bp {
                break;
            }
            let token = self.bump();
            let right_bp = match entry.associativity {
                Associativity::Left => entry.precedence.saturating_add(1),
                Associativity::Right => entry.precedence,
                Associativity::NonAssoc => entry.precedence.saturating_add(1),
            };
            let rhs = self.parse_notation_expr(right_bp)?;
            if entry.associativity == Associativity::NonAssoc {
                if let Some(next) = self.active_infix_entry() {
                    if next.precedence == entry.precedence {
                        return Err(Diagnostic::parser(
                            self.peek().span,
                            "non-associative infix notation must be parenthesized",
                        ));
                    }
                }
            }
            let span = expr.span().join(rhs.span());
            expr = SurfaceExpr::Notation {
                head: crate::NotationHead {
                    kind: entry.kind,
                    symbol: entry.symbol,
                    span: token.span,
                },
                args: vec![expr, rhs],
                span,
            };
        }
        Ok(expr)
    }

    fn parse_notation_prefix_or_application(&mut self) -> Result<SurfaceExpr> {
        let mut expr = if let Some(entry) = self.active_prefix_entry() {
            let token = self.bump();
            let rhs = self.parse_notation_expr(entry.precedence)?;
            SurfaceExpr::Notation {
                head: crate::NotationHead {
                    kind: NotationKind::Prefix,
                    symbol: entry.symbol,
                    span: token.span,
                },
                span: token.span.join(rhs.span()),
                args: vec![rhs],
            }
        } else {
            self.parse_atom()?
        };

        while self.starts_atom() {
            let arg = self.parse_atom()?;
            let span = expr.span().join(arg.span());
            expr = SurfaceExpr::App {
                func: Box::new(expr),
                arg: Box::new(arg),
                span,
            };
        }
        Ok(expr)
    }

    fn active_prefix_entry(&self) -> Option<NotationSyntaxEntry> {
        self.active_symbol_entry()
            .and_then(|entry| (entry.kind == NotationKind::Prefix).then_some(entry))
    }

    fn active_postfix_entry(&self) -> Option<NotationSyntaxEntry> {
        self.active_symbol_entry()
            .and_then(|entry| (entry.kind == NotationKind::Postfix).then_some(entry))
    }

    fn active_infix_entry(&self) -> Option<NotationSyntaxEntry> {
        self.active_symbol_entry()
            .and_then(|entry| match entry.kind {
                NotationKind::Infix | NotationKind::Infixl | NotationKind::Infixr => Some(entry),
                NotationKind::Prefix | NotationKind::Postfix => None,
            })
    }

    fn active_symbol_entry(&self) -> Option<NotationSyntaxEntry> {
        let TokenKind::Symbol(symbol) = &self.peek().kind else {
            return None;
        };
        self.active_notation_entries()
            .into_iter()
            .find(|entry| &entry.symbol == symbol)
    }

    fn active_notation_entries(&self) -> Vec<NotationSyntaxEntry> {
        self.notation_scopes
            .iter()
            .flat_map(|scope| scope.iter().cloned())
            .collect()
    }

    fn parse_atom(&mut self) -> Result<SurfaceExpr> {
        match &self.peek().kind {
            TokenKind::Ident(_) => {
                let name = self.parse_qual_name()?;
                let universe_args = self.parse_universe_args()?;
                let span = if let Some(args) = &universe_args {
                    args.last()
                        .map_or(name.span, |arg| name.span.join(arg.span()))
                } else {
                    name.span
                };
                Ok(SurfaceExpr::Ident {
                    name,
                    universe_args,
                    implicit_mode: ImplicitMode::Insert,
                    span,
                })
            }
            TokenKind::At => {
                let start = self.bump().span;
                let name = self.parse_qual_name()?;
                let universe_args = self.parse_universe_args()?;
                let span = if let Some(args) = &universe_args {
                    args.last()
                        .map_or(start.join(name.span), |arg| start.join(arg.span()))
                } else {
                    start.join(name.span)
                };
                Ok(SurfaceExpr::Ident {
                    name,
                    universe_args,
                    implicit_mode: ImplicitMode::Explicit,
                    span,
                })
            }
            TokenKind::Prop => {
                let span = self.bump().span;
                Ok(SurfaceExpr::Sort {
                    level: SurfaceLevel::Nat { value: 0, span },
                    span,
                })
            }
            TokenKind::Type => {
                let type_span = self.bump().span;
                let (level, span) = if self.starts_level() {
                    let level = self.parse_level()?;
                    let span = type_span.join(level.span());
                    (
                        SurfaceLevel::Succ {
                            level: Box::new(level),
                            span,
                        },
                        span,
                    )
                } else {
                    (
                        SurfaceLevel::Nat {
                            value: 1,
                            span: type_span,
                        },
                        type_span,
                    )
                };
                Ok(SurfaceExpr::Sort { level, span })
            }
            TokenKind::Sort => {
                let start = self.bump().span;
                let level = self.parse_level()?;
                Ok(SurfaceExpr::Sort {
                    span: start.join(level.span()),
                    level,
                })
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_term()?;
                self.expect(TokenTag::RParen, "`)`")?;
                Ok(expr)
            }
            TokenKind::Underscore => {
                let span = self.bump().span;
                Ok(SurfaceExpr::Hole { name: None, span })
            }
            TokenKind::Question => {
                let start = self.bump().span;
                let (name, name_span) = self.parse_ident()?;
                Ok(SurfaceExpr::Hole {
                    name: Some(SurfaceName::single(name, name_span)),
                    span: start.join(name_span),
                })
            }
            _ => Err(self.error_here(format!("expected term, found {}", self.peek().kind.label()))),
        }
    }

    fn parse_decl_binders(&mut self) -> Result<Vec<SurfaceBinder>> {
        let mut binders = Vec::new();
        while self.at(TokenTag::LParen) || self.at(TokenTag::LBrace) {
            binders.extend(self.parse_decl_binder_group()?);
        }
        Ok(binders)
    }

    fn parse_lambda_binder(&mut self) -> Result<Vec<SurfaceBinder>> {
        if self.at(TokenTag::LParen) || self.at(TokenTag::LBrace) {
            return self.parse_decl_binder_group();
        }
        if self.at(TokenTag::Underscore) {
            let span = self.bump().span;
            return Ok(vec![SurfaceBinder {
                kind: SurfaceBinderKind::Anonymous,
                ty: None,
                binder_info: BinderInfo::Explicit,
                span,
            }]);
        }
        let (name, span) = self.parse_ident()?;
        Ok(vec![SurfaceBinder {
            kind: SurfaceBinderKind::Named(SurfaceName::single(name, span)),
            ty: None,
            binder_info: BinderInfo::Explicit,
            span,
        }])
    }

    fn parse_decl_binder_group(&mut self) -> Result<Vec<SurfaceBinder>> {
        let (binder_info, close) = if self.at(TokenTag::LParen) {
            self.bump();
            (BinderInfo::Explicit, TokenTag::RParen)
        } else if self.at(TokenTag::LBrace) {
            self.bump();
            (BinderInfo::Implicit, TokenTag::RBrace)
        } else {
            return Err(self.error_here("expected binder"));
        };

        let mut names = Vec::new();
        while !self.at(TokenTag::Colon) {
            if self.at_eof() || self.at(close) {
                return Err(self.error_here("expected `:` in binder"));
            }
            names.push(self.parse_ident()?);
        }
        if names.is_empty() {
            return Err(self.error_here("binder must contain at least one identifier"));
        }

        self.expect(TokenTag::Colon, "`:`")?;
        let ty = self.parse_term()?;
        let close_span = self.expect(close, close.label())?.span;
        let group_span = names
            .first()
            .map_or(ty.span(), |(_, span)| span.join(close_span));

        Ok(names
            .into_iter()
            .map(|(name, name_span)| SurfaceBinder {
                kind: SurfaceBinderKind::Named(SurfaceName::single(name, name_span)),
                ty: Some(Box::new(ty.clone())),
                binder_info: binder_info.clone(),
                span: group_span,
            })
            .collect())
    }

    fn parse_universe_params(&mut self) -> Result<Vec<SurfaceUniverseParam>> {
        if !self.at(TokenTag::DotLBrace) {
            return Ok(Vec::new());
        }
        self.bump();
        let mut params = Vec::new();
        loop {
            let (name, span) = self.parse_ident()?;
            params.push(SurfaceUniverseParam { name, span });
            if self.at(TokenTag::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        self.expect(TokenTag::RBrace, "`}`")?;
        Ok(params)
    }

    fn parse_universe_args(&mut self) -> Result<Option<Vec<SurfaceLevel>>> {
        if !self.at(TokenTag::DotLBrace) {
            return Ok(None);
        }
        self.bump();
        let mut args = Vec::new();
        loop {
            args.push(self.parse_level()?);
            if self.at(TokenTag::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        self.expect(TokenTag::RBrace, "`}`")?;
        Ok(Some(args))
    }

    fn parse_level(&mut self) -> Result<SurfaceLevel> {
        match self.peek().kind.clone() {
            TokenKind::Number(value) => {
                let span = self.bump().span;
                Ok(SurfaceLevel::Nat { value, span })
            }
            TokenKind::Ident(name) if name == "succ" => {
                let start = self.bump().span;
                let level = self.parse_level()?;
                Ok(SurfaceLevel::Succ {
                    span: start.join(level.span()),
                    level: Box::new(level),
                })
            }
            TokenKind::Ident(name) if name == "max" => {
                let start = self.bump().span;
                let lhs = self.parse_level()?;
                let rhs = self.parse_level()?;
                Ok(SurfaceLevel::Max {
                    span: start.join(rhs.span()),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            TokenKind::Ident(name) if name == "imax" => {
                let start = self.bump().span;
                let lhs = self.parse_level()?;
                let rhs = self.parse_level()?;
                Ok(SurfaceLevel::IMax {
                    span: start.join(rhs.span()),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                })
            }
            TokenKind::Ident(name) => {
                let span = self.bump().span;
                Ok(SurfaceLevel::Param { name, span })
            }
            _ => Err(self.error_here(format!(
                "expected universe level, found {}",
                self.peek().kind.label()
            ))),
        }
    }

    fn parse_qual_name(&mut self) -> Result<SurfaceName> {
        let (first, first_span) = self.parse_ident()?;
        let mut parts = vec![first];
        let mut span = first_span;
        while self.at(TokenTag::Dot) {
            self.bump();
            let (part, part_span) = self.parse_ident()?;
            span = span.join(part_span);
            parts.push(part);
        }
        Ok(SurfaceName { parts, span })
    }

    fn parse_ident(&mut self) -> Result<(String, Span)> {
        let token = self.bump();
        match token.kind {
            TokenKind::Ident(name) => Ok((name, token.span)),
            kind => Err(Diagnostic::parser(
                token.span,
                format!("expected identifier, found {}", kind.label()),
            )),
        }
    }

    fn parse_number(&mut self) -> Result<u32> {
        let token = self.bump();
        match token.kind {
            TokenKind::Number(value) => u32::try_from(value)
                .map_err(|_| Diagnostic::parser(token.span, "number literal does not fit in u32")),
            kind => Err(Diagnostic::parser(
                token.span,
                format!("expected number, found {}", kind.label()),
            )),
        }
    }

    fn parse_string(&mut self) -> Result<(String, Span)> {
        let token = self.bump();
        match token.kind {
            TokenKind::String(value) => Ok((value, token.span)),
            kind => Err(Diagnostic::parser(
                token.span,
                format!("expected string, found {}", kind.label()),
            )),
        }
    }

    fn starts_atom(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Ident(_)
                | TokenKind::At
                | TokenKind::Prop
                | TokenKind::Type
                | TokenKind::Sort
                | TokenKind::LParen
                | TokenKind::Underscore
                | TokenKind::Question
        )
    }

    fn starts_level(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_) | TokenKind::Number(_))
    }

    fn peek_is_ident(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_))
    }

    fn at(&self, tag: TokenTag) -> bool {
        tag.matches(&self.peek().kind)
    }

    fn at_eof(&self) -> bool {
        self.at(TokenTag::Eof)
    }

    fn expect(&mut self, tag: TokenTag, expected: &str) -> Result<Token> {
        if self.at(tag) {
            Ok(self.bump())
        } else {
            Err(self.error_here(format!(
                "expected {expected}, found {}",
                self.peek().kind.label()
            )))
        }
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if !matches!(token.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        token
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn error_here(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::parser(self.peek().span, message)
    }
}

fn normalize_notation_symbol(raw: &str, span: Span) -> Result<String> {
    let symbol = raw.trim();
    if symbol.is_empty() {
        return Err(invalid_notation(span, "notation symbol must not be empty"));
    }
    if symbol.chars().any(char::is_whitespace) {
        return Err(invalid_notation(
            span,
            "notation symbol must be a single operator token",
        ));
    }
    if is_reserved_notation_symbol(symbol) || contains_reserved_structural_char(symbol) {
        return Err(invalid_notation(
            span,
            format!("reserved token `{symbol}` cannot be used as notation"),
        ));
    }
    if symbol.chars().any(is_identifier_char) {
        return Err(invalid_notation(
            span,
            "notation symbol must not contain identifier characters",
        ));
    }
    Ok(symbol.to_owned())
}

fn associativity_for_kind(kind: &NotationKind) -> Associativity {
    match kind {
        NotationKind::Infixl => Associativity::Left,
        NotationKind::Infixr => Associativity::Right,
        NotationKind::Prefix | NotationKind::Postfix | NotationKind::Infix => {
            Associativity::NonAssoc
        }
    }
}

fn parser_import_map(
    imports: &[ParserImport],
) -> BTreeMap<Vec<String>, Vec<ScopedNotationSyntaxEntry>> {
    let mut map: BTreeMap<Vec<String>, Vec<ScopedNotationSyntaxEntry>> = BTreeMap::new();
    for import in imports {
        map.entry(import.module.clone()).or_default().extend(
            import
                .notations
                .iter()
                .map(ScopedNotationSyntaxEntry::from_imported),
        );
    }
    map
}

fn invalid_notation(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(DiagnosticKind::InvalidNotation, span, message)
}

fn is_reserved_notation_symbol(symbol: &str) -> bool {
    matches!(symbol, "->" | "→" | "=>" | ":=" | ".{")
}

fn contains_reserved_structural_char(symbol: &str) -> bool {
    symbol.chars().any(|ch| {
        matches!(
            ch,
            ':' | ',' | '.' | '(' | ')' | '{' | '}' | '|' | '@' | '_' | '?'
        )
    })
}

fn is_identifier_char(ch: char) -> bool {
    ch == '\'' || ch.is_ascii_alphanumeric()
}

#[derive(Clone, Copy)]
enum TokenTag {
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
    Underscore,
    Eof,
}

impl TokenTag {
    fn matches(self, kind: &TokenKind) -> bool {
        matches!(
            (self, kind),
            (Self::Import, TokenKind::Import)
                | (Self::Open, TokenKind::Open)
                | (Self::Namespace, TokenKind::Namespace)
                | (Self::End, TokenKind::End)
                | (Self::Def, TokenKind::Def)
                | (Self::Theorem, TokenKind::Theorem)
                | (Self::Axiom, TokenKind::Axiom)
                | (Self::Inductive, TokenKind::Inductive)
                | (Self::Where, TokenKind::Where)
                | (Self::Fun, TokenKind::Fun)
                | (Self::Forall, TokenKind::Forall)
                | (Self::Pi, TokenKind::Pi)
                | (Self::Let, TokenKind::Let)
                | (Self::In, TokenKind::In)
                | (Self::LParen, TokenKind::LParen)
                | (Self::RParen, TokenKind::RParen)
                | (Self::LBrace, TokenKind::LBrace)
                | (Self::RBrace, TokenKind::RBrace)
                | (Self::Comma, TokenKind::Comma)
                | (Self::Colon, TokenKind::Colon)
                | (Self::ColonEq, TokenKind::ColonEq)
                | (Self::Arrow, TokenKind::Arrow)
                | (Self::FatArrow, TokenKind::FatArrow)
                | (Self::Pipe, TokenKind::Pipe)
                | (Self::Dot, TokenKind::Dot)
                | (Self::DotLBrace, TokenKind::DotLBrace)
                | (Self::Underscore, TokenKind::Underscore)
                | (Self::Eof, TokenKind::Eof)
        )
    }

    fn label(self) -> &'static str {
        match self {
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
            Self::Underscore => "`_`",
            Self::Eof => "end of file",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiagnosticKind, SurfaceExpr};

    fn parse(source: &str) -> SurfaceModule {
        parse_module(FileId(0), source).expect("source should parse")
    }

    #[test]
    fn parses_explicit_def() {
        let module = parse("def id_explicit (A : Type) (x : A) : A := x");
        let SurfaceItem::Def(decl) = &module.items[0] else {
            panic!("expected def");
        };
        assert_eq!(decl.name, "id_explicit");
        assert_eq!(decl.binders.len(), 2);
        assert!(decl.value.is_some());
        assert!(matches!(
            decl.binders[0].ty.as_deref(),
            Some(SurfaceExpr::Sort {
                level: SurfaceLevel::Nat { value: 1, .. },
                ..
            })
        ));
    }

    #[test]
    fn desugars_arrow_as_right_associative_pi() {
        let module = parse("axiom f : Nat -> Nat -> Nat");
        let SurfaceItem::Axiom(decl) = &module.items[0] else {
            panic!("expected axiom");
        };
        let SurfaceExpr::Pi { body, .. } = &decl.ty else {
            panic!("expected outer pi");
        };
        assert!(matches!(body.as_ref(), SurfaceExpr::Pi { .. }));
    }

    #[test]
    fn expands_grouped_binders() {
        let module = parse("def first (A : Type) (x y : A) : A := x");
        let SurfaceItem::Def(decl) = &module.items[0] else {
            panic!("expected def");
        };
        assert_eq!(decl.binders.len(), 3);
        let names: Vec<_> = decl
            .binders
            .iter()
            .map(|binder| match &binder.kind {
                SurfaceBinderKind::Named(name) => name.parts[0].as_str(),
                SurfaceBinderKind::Anonymous => "_",
            })
            .collect();
        assert_eq!(names, ["A", "x", "y"]);
    }

    #[test]
    fn rejects_import_after_non_import_item() {
        let err = parse_module(FileId(0), "def x : Nat := Nat.zero\nimport Std.Nat.Basic")
            .expect_err("import must be rejected");
        assert_eq!(err.kind, DiagnosticKind::ImportAfterItem);
    }

    #[test]
    fn rejects_chained_type_annotations() {
        let err = parse_module(FileId(0), "def bad : T := x : A : B")
            .expect_err("chained type annotations must be rejected");
        assert_eq!(err.kind, DiagnosticKind::ParserError);
    }

    #[test]
    fn parses_holes_and_explicit_head_mode() {
        let module = parse("theorem h : T := fun _ => @Eq.refl.{u} ?m");
        let SurfaceItem::Theorem(decl) = &module.items[0] else {
            panic!("expected theorem");
        };
        let value = decl.value.as_ref().expect("theorem value");
        let SurfaceExpr::Lam { binders, body, .. } = value else {
            panic!("expected lambda");
        };
        assert!(matches!(binders[0].kind, SurfaceBinderKind::Anonymous));
        let SurfaceExpr::App { func, arg, .. } = body.as_ref() else {
            panic!("expected application");
        };
        assert!(matches!(
            arg.as_ref(),
            SurfaceExpr::Hole { name: Some(_), .. }
        ));
        assert!(matches!(
            func.as_ref(),
            SurfaceExpr::Ident {
                implicit_mode: ImplicitMode::Explicit,
                ..
            }
        ));
    }

    #[test]
    fn parses_module_items_and_simple_inductive() {
        let module = parse(
            r#"
import Std.Nat.Basic
open Nat
namespace Demo
infixl:65 " + " => Nat.add
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
end Demo
"#,
        );
        assert_eq!(module.items.len(), 6);
        assert!(matches!(module.items[0], SurfaceItem::Import { .. }));
        let SurfaceItem::Notation(notation) = &module.items[3] else {
            panic!("expected notation");
        };
        assert_eq!(notation.symbol, "+");
        let SurfaceItem::Inductive { constructors, .. } = &module.items[4] else {
            panic!("expected inductive");
        };
        assert_eq!(constructors.len(), 2);
    }

    #[test]
    fn parses_active_infix_notation_with_precedence() {
        let module = parse(
            r#"
infixl:65 " + " => Nat.add
infix:50 " = " => Eq
axiom t : a + b = c + d
"#,
        );
        let SurfaceItem::Axiom(decl) = &module.items[2] else {
            panic!("expected axiom");
        };
        let SurfaceExpr::Notation { head, args, .. } = &decl.ty else {
            panic!("expected equality notation");
        };
        assert_eq!(head.symbol, "=");
        assert!(matches!(
            &args[0],
            SurfaceExpr::Notation { head, .. } if head.symbol == "+"
        ));
        assert!(matches!(
            &args[1],
            SurfaceExpr::Notation { head, .. } if head.symbol == "+"
        ));
    }

    #[test]
    fn parses_prefix_and_postfix_notation() {
        let module = parse(
            r#"
prefix:70 "!" => negate
postfix:80 "$" => inspect
axiom t : ! a$
"#,
        );
        let SurfaceItem::Axiom(decl) = &module.items[2] else {
            panic!("expected axiom");
        };
        let SurfaceExpr::Notation { head, args, .. } = &decl.ty else {
            panic!("expected prefix notation");
        };
        assert_eq!(head.kind, NotationKind::Prefix);
        assert!(matches!(
            &args[0],
            SurfaceExpr::Notation { head, .. } if head.kind == NotationKind::Postfix
        ));
    }

    #[test]
    fn parses_notation_after_relative_open_inside_namespace() {
        let module = parse(
            r#"
namespace A
namespace N
infixl:65 " + " => add
end N
end A
namespace A
open N
axiom t : a + b
end A
"#,
        );
        let SurfaceItem::Axiom(decl) = &module.items[7] else {
            panic!("expected axiom");
        };
        assert!(matches!(
            decl.ty,
            SurfaceExpr::Notation { ref head, .. } if head.symbol == "+"
        ));
    }

    #[test]
    fn parses_imported_notation_after_open() {
        let module = parse_module_with_imports(
            FileId(0),
            r#"
import Std.Nat
open Nat
axiom t : a + b
"#,
            &[ParserImport {
                module: vec!["Std".to_owned(), "Nat".to_owned()],
                notations: vec![ParserImportedNotation {
                    kind: NotationKind::Infixl,
                    precedence: 65,
                    symbol: "+".to_owned(),
                    namespace: Some(vec!["Nat".to_owned()]),
                }],
            }],
        )
        .expect("imported namespace notation should be active after open");
        let SurfaceItem::Axiom(decl) = &module.items[2] else {
            panic!("expected axiom");
        };
        assert!(matches!(
            decl.ty,
            SurfaceExpr::Notation { ref head, .. } if head.symbol == "+"
        ));
    }

    #[test]
    fn parses_notation_after_open_through_opened_namespace() {
        let module = parse(
            r#"
namespace A
namespace N
infixl:65 " + " => add
end N
end A
open A
open N
axiom t : a + b
"#,
        );
        let SurfaceItem::Axiom(decl) = &module.items[7] else {
            panic!("expected axiom");
        };
        assert!(matches!(
            decl.ty,
            SurfaceExpr::Notation { ref head, .. } if head.symbol == "+"
        ));
    }

    #[test]
    fn rejects_non_associative_infix_chains() {
        let err = parse_module(
            FileId(0),
            r#"
infix:50 " = " => Eq
axiom bad : a = b = c
"#,
        )
        .expect_err("non-associative notation chains must be rejected");
        assert_eq!(err.kind, DiagnosticKind::ParserError);
    }

    #[test]
    fn rejects_active_notation_conflicts() {
        let err = parse_module(
            FileId(0),
            r#"
infixl:65 " + " => Nat.add
infixr:65 " + " => Int.add
"#,
        )
        .expect_err("conflicting active notation must be rejected");
        assert_eq!(err.kind, DiagnosticKind::NotationConflict);
    }

    #[test]
    fn rejects_invalid_notation_symbols() {
        for source in [
            r#"infix:50 "" => Eq"#,
            r#"infix:50 "+ +" => Eq"#,
            r#"infix:50 "->" => Arrow"#,
            r#"infix:50 "foo" => Foo"#,
        ] {
            let err = parse_module(FileId(0), source).expect_err("notation must be rejected");
            assert_eq!(err.kind, DiagnosticKind::InvalidNotation);
        }
    }

    #[test]
    fn normalizes_type_with_level() {
        let module = parse("axiom U.{u} : Type u");
        let SurfaceItem::Axiom(decl) = &module.items[0] else {
            panic!("expected axiom");
        };
        assert!(matches!(
            decl.ty,
            SurfaceExpr::Sort {
                level: SurfaceLevel::Succ { .. },
                ..
            }
        ));
    }
}
