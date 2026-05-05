mod diagnostic;
mod elaborator;
mod lexer;
mod parser;
mod resolver;
mod span;
mod surface;

pub use diagnostic::{Diagnostic, DiagnosticKind, DiagnosticSeverity, Result};
pub use elaborator::{
    elaborate_module, elaborate_resolved_module, elaborate_source, ElaboratedModule,
};
pub use lexer::{lex, Token, TokenKind};
pub use parser::parse_module;
pub use resolver::{
    resolve_module, resolve_source, ElabGlobalRef, FrontendState, GlobalDeclaration, GlobalOrigin,
    GlobalScope, ImportedDeclaration, ImportedNotation, ImportedTypeMetadata, LocalBinding,
    LocalId, LocalRef, LocalScopeStack, Name, OpenScope, ResolvedBinder, ResolvedCtorDecl,
    ResolvedDecl, ResolvedExpr, ResolvedImport, ResolvedItem, ResolvedModule, ResolvedName,
    ResolvedNotationDecl, VerifiedImport,
};
pub use span::{ByteOffset, FileId, Span};
pub use surface::{
    BinderInfo, ImplicitMode, NotationDecl, NotationHead, NotationKind, SurfaceBinder,
    SurfaceBinderKind, SurfaceCtorDecl, SurfaceDecl, SurfaceExpr, SurfaceItem, SurfaceLevel,
    SurfaceModule, SurfaceName, SurfaceUniverseParam,
};
