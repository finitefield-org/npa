mod diagnostic;
mod elaborator;
mod lexer;
mod machine;
mod parser;
mod resolver;
mod span;
mod term_source;

pub use diagnostic::{MachineDiagnostic, MachineDiagnosticKind, MachineDiagnosticSeverity, Result};
pub use elaborator::{
    compile_machine_source_to_certificate, compile_machine_source_to_core,
    elaborate_machine_module, elaborate_machine_term_check, elaborate_machine_term_infer_from_ast,
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
    resolve_machine_module, ResolvedMachineModule, VerifiedDependency, VerifiedExport,
    VerifiedImport,
};
pub use span::{ByteOffset, FileId, Span};
pub use term_source::{
    canonicalize_machine_term_source, decode_machine_term_source_canonical,
    lex_machine_surface_tokens, MachineSurfaceToken,
};
