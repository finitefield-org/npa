use crate::Span;

pub type Result<T> = std::result::Result<T, MachineDiagnostic>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineDiagnosticKind {
    ParseError,
    UnsupportedItem,
    UnsupportedSyntax,
    ImportAfterItem,
    ImportResolutionError,
    MissingVerifiedImport,
    UnknownGlobalName,
    ShortGlobalName,
    AmbiguousGlobalName,
    GlobalShadowedByLocal,
    UnknownLocalName,
    DuplicateDeclaration,
    DuplicateUniverseParam,
    UnknownUniverseParam,
    ImplicitArgumentRequired,
    MissingExplicitUniverse,
    UnannotatedBinder,
    UnannotatedLet,
    HoleNotAllowed,
    ExpectedFunctionType,
    ExpectedSort,
    TypeMismatch,
    TooManyArguments,
    TooFewArguments,
    UnsolvedUniverseMeta,
    KernelRejected,
    CertificateRejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineDiagnostic {
    pub kind: MachineDiagnosticKind,
    pub severity: MachineDiagnosticSeverity,
    pub primary_span: Span,
    pub message: String,
}

impl MachineDiagnostic {
    pub fn error(
        kind: MachineDiagnosticKind,
        primary_span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity: MachineDiagnosticSeverity::Error,
            primary_span,
            message: message.into(),
        }
    }

    pub fn warning(
        kind: MachineDiagnosticKind,
        primary_span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity: MachineDiagnosticSeverity::Warning,
            primary_span,
            message: message.into(),
        }
    }

    pub fn parse(primary_span: Span, message: impl Into<String>) -> Self {
        Self::error(MachineDiagnosticKind::ParseError, primary_span, message)
    }

    pub fn unsupported_syntax(primary_span: Span, syntax: impl Into<String>) -> Self {
        Self::error(
            MachineDiagnosticKind::UnsupportedSyntax,
            primary_span,
            format!("unsupported Machine Surface syntax: {}", syntax.into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileId;

    #[test]
    fn builds_simple_error_diagnostic() {
        let span = Span::new(FileId(1), 2, 3);
        let diagnostic = MachineDiagnostic::unsupported_syntax(span, "open");

        assert_eq!(diagnostic.kind, MachineDiagnosticKind::UnsupportedSyntax);
        assert_eq!(diagnostic.severity, MachineDiagnosticSeverity::Error);
        assert_eq!(diagnostic.primary_span, span);
    }
}
