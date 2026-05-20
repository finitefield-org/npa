//! Human-facing frontend for NPA machine surface syntax.
//!
//! This crate lowers source into `npa_cert::CoreModule` values and, when asked
//! to produce a certificate, crosses the canonical `build_module_cert` /
//! `verify_module_cert` boundary. Phase 2 producer fast-path candidates remain
//! in `npa-cert` until a separate bridge is designed.

mod callable;
mod diagnostic;
mod elaborator;
mod lexer;
mod machine;
mod parser;
mod resolver;
mod span;
mod term_source;

pub use callable::{
    is_machine_surface_renderable_name, MachineCallableBinderVisibility,
    MachineSurfaceCallableInterfaceEntry, MachineSurfaceCallableInterfaceError,
    MachineSurfaceCallableInterfaceTable, MachineSurfaceCallableRef,
};
pub use diagnostic::{
    MachineDiagnostic, MachineDiagnosticKind, MachineDiagnosticPayload, MachineDiagnosticSeverity,
    MachineRepairCandidate, MachineRepairSuggestion, MachineRepairSuggestionKind, Result,
};
pub use elaborator::{
    compile_machine_source_to_certificate, compile_machine_source_to_core,
    elaborate_machine_module, elaborate_machine_term_check, elaborate_machine_term_infer_from_ast,
    MachineTermElabContextInModuleRequest,
};
pub use lexer::{lex, Token, TokenKind};
pub use machine::{
    MachineBinder, MachineCheckedCurrentDecl, MachineCheckedCurrentGeneratedDecl,
    MachineCompileOptions, MachineDecl, MachineGlobalScope, MachineGlobalScopeEntry, MachineItem,
    MachineKernelEnvView, MachineLevel, MachineLocalDecl, MachineModule, MachineName,
    MachineResolvedConstant, MachineSurfaceMode, MachineTerm, MachineTermAst,
    MachineTermCheckResult, MachineTermElabContext, MachineTermSourceCanonical,
    MachineUniverseParam,
};
pub use parser::{parse_machine_module, parse_machine_term};
pub use resolver::{
    resolve_machine_module, resolve_machine_module_with_options, ResolvedMachineModule,
    VerifiedDependency, VerifiedExport, VerifiedImport,
};
pub use span::{ByteOffset, FileId, Span};
pub use term_source::{
    canonicalize_machine_term_source, decode_machine_term_source_canonical,
    lex_machine_surface_tokens, MachineSurfaceToken, MachineSurfaceTokenKind,
};
