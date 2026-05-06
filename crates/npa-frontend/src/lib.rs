mod diagnostic;
mod elaborator;
mod lexer;
mod machine;
mod parser;
mod resolver;
mod span;

pub use diagnostic::{MachineDiagnostic, MachineDiagnosticKind, MachineDiagnosticSeverity, Result};
pub use elaborator::{
    compile_machine_source_to_certificate, compile_machine_source_to_core,
    elaborate_machine_module, elaborate_machine_term_check,
};
pub use lexer::{lex, Token, TokenKind};
pub use machine::{
    MachineBinder, MachineCompileOptions, MachineItem, MachineLevel, MachineLocalDecl,
    MachineModule, MachineName, MachineSurfaceMode, MachineTerm,
};
pub use parser::parse_machine_module;
pub use resolver::{resolve_machine_module, ResolvedMachineModule, VerifiedImport};
pub use span::{ByteOffset, FileId, Span};
