use crate::Span;

pub type HumanResult<T> = std::result::Result<T, HumanDiagnostic>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanDiagnosticKind {
    NotImplemented,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanDiagnostic {
    pub kind: HumanDiagnosticKind,
    pub severity: HumanDiagnosticSeverity,
    pub primary_span: Span,
    pub message: String,
}

impl HumanDiagnostic {
    pub fn error(
        kind: HumanDiagnosticKind,
        primary_span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity: HumanDiagnosticSeverity::Error,
            primary_span,
            message: message.into(),
        }
    }

    pub fn not_implemented(primary_span: Span, operation: &str) -> Self {
        Self::error(
            HumanDiagnosticKind::NotImplemented,
            primary_span,
            format!("{operation} is reserved for the Phase 3 Human frontend"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileId;

    #[test]
    fn human_diagnostic_is_separate_from_machine_diagnostic() {
        let diagnostic =
            HumanDiagnostic::not_implemented(Span::empty(FileId(2)), "parse_human_module");

        assert_eq!(diagnostic.kind, HumanDiagnosticKind::NotImplemented);
        assert_eq!(diagnostic.severity, HumanDiagnosticSeverity::Error);
        assert_eq!(diagnostic.primary_span, Span::empty(FileId(2)));
        assert!(diagnostic.message.contains("Phase 3 Human"));
    }
}
