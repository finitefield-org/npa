//! Frontend profiles for NPA source syntax.
//!
//! This crate lowers source into `npa_cert::CoreModule` values and, when asked
//! to produce a certificate, crosses the canonical `build_module_cert` /
//! `verify_module_cert` boundary. Phase 2 producer fast-path candidates remain
//! in `npa-cert` until a separate bridge is designed.

mod callable;
mod diagnostic;
mod elaborator;
mod human;
mod human_diagnostic;
mod human_elaborator;
mod human_parser;
mod human_resolver;
mod lexer;
mod machine;
mod parser;
mod resolver;
mod span;
mod term_source;

pub use callable::{
    builtin_machine_callable_profile, is_machine_surface_renderable_name,
    machine_callable_profile_from_human_binders,
    machine_callable_visibility_from_human_binder_info, MachineCallableBinderVisibility,
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
pub use human::{
    HumanAxiomDecl, HumanBinder, HumanBinderInfo, HumanBinderKind, HumanCompileOptions,
    HumanConstructorDecl, HumanDecl, HumanExpr, HumanFrontendState, HumanGeneratedDeclarationKind,
    HumanGeneratedDeclarationMetadata, HumanImplicitMode, HumanImportedSourceInterface,
    HumanInductiveDecl, HumanItem, HumanLevel, HumanModule, HumanName, HumanNotationAssociativity,
    HumanNotationDecl, HumanNotationHead, HumanNotationKind, HumanOpenScope, HumanOpenScopeFrame,
    HumanSourceBinderMetadata, HumanSourceDeclarationKind, HumanSourceDeclarationMetadata,
    HumanSourceInterface, HumanSourceInterfaceStore, HumanSourceNotationMetadata,
    HumanUniverseParam,
};
pub use human_diagnostic::{
    HumanDiagnostic, HumanDiagnosticKind, HumanDiagnosticPayload, HumanDiagnosticSeverity,
    HumanHoleGoal, HumanHoleGoalLocal, HumanResult,
};
pub use human_elaborator::{
    compile_human_source_to_certificate, compile_human_source_to_core, elaborate_human_module,
};
pub use human_parser::{parse_human_module, parse_human_term};
pub use human_resolver::{
    resolve_human_module, HumanGlobalRef, HumanGlobalScope, HumanGlobalScopeEntry,
    HumanResolvedName, HumanResolvedNameUse, HumanResolvedNotationEntry, HumanResolvedNotationUse,
    ResolvedHumanModule,
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
