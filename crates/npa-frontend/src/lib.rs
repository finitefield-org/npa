mod diagnostic;
mod lexer;
mod parser;
mod resolver;
mod span;
mod surface;

pub use diagnostic::{Diagnostic, DiagnosticKind, DiagnosticSeverity, Result};
pub use lexer::{lex, Token, TokenKind};
pub use parser::parse_module;
pub use resolver::{
    resolve_module, resolve_source, ElabGlobalRef, FrontendState, GlobalDeclaration, GlobalOrigin,
    GlobalScope, ImportedDeclaration, LocalBinding, LocalId, LocalRef, LocalScopeStack, Name,
    OpenScope, ResolvedBinder, ResolvedCtorDecl, ResolvedDecl, ResolvedExpr, ResolvedImport,
    ResolvedItem, ResolvedModule, ResolvedName, VerifiedImport,
};
pub use span::{ByteOffset, FileId, Span};
pub use surface::{
    BinderInfo, ImplicitMode, NotationDecl, NotationHead, NotationKind, SurfaceBinder,
    SurfaceBinderKind, SurfaceCtorDecl, SurfaceDecl, SurfaceExpr, SurfaceItem, SurfaceLevel,
    SurfaceModule, SurfaceName, SurfaceUniverseParam,
};
