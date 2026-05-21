use crate::{FileId, HumanDiagnostic, HumanExpr, HumanModule, HumanResult, Span};

pub fn parse_human_module(file_id: FileId, source: &str) -> HumanResult<HumanModule> {
    Err(HumanDiagnostic::not_implemented(
        source_span(file_id, source),
        "parse_human_module",
    ))
}

pub fn parse_human_term(file_id: FileId, source: &str) -> HumanResult<HumanExpr> {
    Err(HumanDiagnostic::not_implemented(
        source_span(file_id, source),
        "parse_human_term",
    ))
}

fn source_span(file_id: FileId, source: &str) -> Span {
    Span::new(file_id, 0, source.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HumanDiagnosticKind;

    #[test]
    fn human_parser_skeleton_does_not_parse_via_machine_surface() {
        let source = "def id (A : Type) (x : A) : A := x";
        let diagnostic = parse_human_module(FileId(5), source)
            .expect_err("P3H-00 reserves the Human parser boundary only");

        assert_eq!(diagnostic.kind, HumanDiagnosticKind::NotImplemented);
        assert_eq!(
            diagnostic.primary_span,
            Span::new(FileId(5), 0, source.len() as u32)
        );
    }
}
