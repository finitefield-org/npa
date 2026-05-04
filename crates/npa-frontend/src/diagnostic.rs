use crate::Span;

pub type Result<T> = std::result::Result<T, Diagnostic>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    ParserError,
    ImportResolutionError,
    ImportAfterItem,
    NamespaceMismatch,
    UnknownNamespace,
    DuplicateDeclaration,
    DuplicateUniverseParam,
    InvalidNotation,
    NotationConflict,
    UnknownIdentifier,
    UnknownUniverseParam,
    AmbiguousName,
    AmbiguousNotation,
    TypeMismatch,
    ExpectedFunctionType,
    ExpectedSort,
    BinderInfoMismatch,
    TooManyArguments,
    UnsolvedImplicit,
    UnsolvedUniverseMeta,
    UnsolvedHole,
    NamedHoleContextMismatch,
    OccursCheckFailed,
    IncompleteDependency,
    ForwardReference,
    KernelRejected,
    ShadowingWarning,
    DuplicateImportWarning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub severity: DiagnosticSeverity,
    pub primary_span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn error(kind: DiagnosticKind, primary_span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            severity: DiagnosticSeverity::Error,
            primary_span,
            message: message.into(),
        }
    }

    pub fn parser(primary_span: Span, message: impl Into<String>) -> Self {
        Self::error(DiagnosticKind::ParserError, primary_span, message)
    }

    pub fn import_after_item(primary_span: Span) -> Self {
        Self::error(
            DiagnosticKind::ImportAfterItem,
            primary_span,
            "import declarations must appear before all non-import items",
        )
    }
}
