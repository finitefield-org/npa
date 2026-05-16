use std::collections::{BTreeMap, BTreeSet};

use crate::{FileId, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineModule {
    pub file_id: FileId,
    pub items: Vec<MachineItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineItem {
    Import { module: MachineName, span: Span },
    Def(MachineDecl),
    Theorem(MachineDecl),
}

impl MachineItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Import { span, .. } => *span,
            Self::Def(decl) | Self::Theorem(decl) => decl.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineDecl {
    pub name: MachineName,
    pub universe_params: Vec<MachineUniverseParam>,
    pub binders: Vec<MachineBinder>,
    pub ty: MachineTerm,
    pub value: MachineTerm,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineUniverseParam {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineName {
    pub parts: Vec<String>,
    pub span: Span,
}

impl MachineName {
    pub fn new(parts: Vec<String>, span: Span) -> Self {
        Self { parts, span }
    }

    pub fn as_dotted(&self) -> String {
        self.parts.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineBinder {
    pub name: String,
    pub ty: MachineTerm,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineLevel {
    Nat {
        value: u64,
        span: Span,
    },
    Param {
        name: String,
        span: Span,
    },
    Succ {
        level: Box<MachineLevel>,
        span: Span,
    },
    Max {
        lhs: Box<MachineLevel>,
        rhs: Box<MachineLevel>,
        span: Span,
    },
    IMax {
        lhs: Box<MachineLevel>,
        rhs: Box<MachineLevel>,
        span: Span,
    },
}

impl MachineLevel {
    pub fn span(&self) -> Span {
        match self {
            Self::Nat { span, .. }
            | Self::Param { span, .. }
            | Self::Succ { span, .. }
            | Self::Max { span, .. }
            | Self::IMax { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineTerm {
    Ident {
        name: MachineName,
        universe_args: Option<Vec<MachineLevel>>,
        explicit_mode: bool,
        span: Span,
    },
    Local {
        name: String,
        span: Span,
    },
    Sort {
        level: MachineLevel,
        span: Span,
    },
    App {
        func: Box<MachineTerm>,
        arg: Box<MachineTerm>,
        span: Span,
    },
    Lam {
        binders: Vec<MachineBinder>,
        body: Box<MachineTerm>,
        span: Span,
    },
    Pi {
        binders: Vec<MachineBinder>,
        body: Box<MachineTerm>,
        span: Span,
    },
    Let {
        name: String,
        ty: Box<MachineTerm>,
        value: Box<MachineTerm>,
        body: Box<MachineTerm>,
        span: Span,
    },
    Annot {
        expr: Box<MachineTerm>,
        ty: Box<MachineTerm>,
        span: Span,
    },
}

impl MachineTerm {
    pub fn span(&self) -> Span {
        match self {
            Self::Ident { span, .. }
            | Self::Local { span, .. }
            | Self::Sort { span, .. }
            | Self::App { span, .. }
            | Self::Lam { span, .. }
            | Self::Pi { span, .. }
            | Self::Let { span, .. }
            | Self::Annot { span, .. } => *span,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineSurfaceMode {
    Complete,
    Repair,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineCompileOptions {
    pub mode: MachineSurfaceMode,
    pub allow_universe_meta: bool,
}

impl Default for MachineCompileOptions {
    fn default() -> Self {
        Self {
            mode: MachineSurfaceMode::Complete,
            allow_universe_meta: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineLocalDecl {
    pub name: String,
    pub ty: npa_kernel::Expr,
    pub value: Option<npa_kernel::Expr>,
}

#[derive(Clone, Debug)]
pub struct MachineTermElabContext {
    pub(crate) global_scope: MachineGlobalScope,
    pub(crate) local_context: Vec<MachineLocalDecl>,
    pub(crate) universe_params: Vec<String>,
    pub(crate) kernel_env: MachineKernelEnvView,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineCheckedCurrentDecl {
    pub name: npa_cert::Name,
    pub source_index: u64,
    pub decl_interface_hash: npa_cert::Hash,
    pub decl: npa_kernel::Decl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineCheckedCurrentGeneratedDecl {
    pub name: npa_cert::Name,
    pub parent_source_index: u64,
    pub decl_interface_hash: npa_cert::Hash,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineGlobalScope {
    pub(crate) entries: Vec<MachineGlobalScopeEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineGlobalScopeEntry {
    Imported {
        name: npa_cert::Name,
        import_index: u32,
        decl_interface_hash: npa_cert::Hash,
    },
    CurrentModule {
        name: npa_cert::Name,
        source_index: u64,
        decl_interface_hash: npa_cert::Hash,
    },
    CurrentGenerated {
        name: npa_cert::Name,
        parent_source_index: u64,
        decl_interface_hash: npa_cert::Hash,
    },
}

impl MachineGlobalScopeEntry {
    pub fn name(&self) -> &npa_cert::Name {
        match self {
            Self::Imported { name, .. }
            | Self::CurrentModule { name, .. }
            | Self::CurrentGenerated { name, .. } => name,
        }
    }

    pub fn decl_interface_hash(&self) -> &npa_cert::Hash {
        match self {
            Self::Imported {
                decl_interface_hash,
                ..
            }
            | Self::CurrentModule {
                decl_interface_hash,
                ..
            }
            | Self::CurrentGenerated {
                decl_interface_hash,
                ..
            } => decl_interface_hash,
        }
    }
}

impl MachineTermElabContext {
    pub fn local_context(&self) -> &[MachineLocalDecl] {
        &self.local_context
    }

    pub fn universe_params(&self) -> &[String] {
        &self.universe_params
    }

    pub fn kernel_env(&self) -> &MachineKernelEnvView {
        &self.kernel_env
    }

    pub fn global_scope_entries(&self) -> &[MachineGlobalScopeEntry] {
        &self.global_scope.entries
    }
}

#[derive(Clone, Debug)]
pub struct MachineKernelEnvView {
    pub(crate) env: npa_kernel::Env,
    decl_interface_hashes: BTreeMap<String, BTreeSet<npa_cert::Hash>>,
}

impl MachineKernelEnvView {
    pub(crate) fn new(env: npa_kernel::Env) -> Self {
        Self {
            env,
            decl_interface_hashes: BTreeMap::new(),
        }
    }

    pub(crate) fn with_decl_interface_hashes(
        env: npa_kernel::Env,
        hashes: impl IntoIterator<Item = (npa_cert::Name, npa_cert::Hash)>,
    ) -> Self {
        let mut view = Self::new(env);
        for (name, hash) in hashes {
            view.add_decl_interface_hash(name, hash);
        }
        view
    }

    pub fn empty() -> Self {
        Self {
            env: npa_kernel::Env::new(),
            decl_interface_hashes: BTreeMap::new(),
        }
    }

    pub fn env(&self) -> &npa_kernel::Env {
        &self.env
    }

    pub(crate) fn add_decl_interface_hash(&mut self, name: npa_cert::Name, hash: npa_cert::Hash) {
        self.decl_interface_hashes
            .entry(name.as_dotted())
            .or_default()
            .insert(hash);
    }

    pub(crate) fn has_decl_interface_hash(&self, name: &str, hash: &npa_cert::Hash) -> bool {
        self.decl_interface_hashes
            .get(name)
            .is_some_and(|hashes| hashes.contains(hash))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTermSourceCanonical {
    pub source: String,
    pub canonical_bytes: Vec<u8>,
    pub canonical_hash: npa_cert::Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTermAst {
    pub(crate) term: MachineTerm,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MachineResolvedConstant {
    pub name: npa_cert::Name,
    pub decl_interface_hash: npa_cert::Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTermCheckResult {
    pub expr: npa_kernel::Expr,
    pub inferred_type: npa_kernel::Expr,
    pub core_hash: npa_cert::Hash,
    pub contextual_core_hash: npa_cert::Hash,
    pub constants: Vec<MachineResolvedConstant>,
}
