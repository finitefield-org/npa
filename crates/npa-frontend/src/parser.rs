use crate::{lex, FileId, MachineModule, Result, Span};

pub fn parse_machine_module(file_id: FileId, source: &str) -> Result<MachineModule> {
    let _tokens = lex(file_id, source)?;
    Ok(MachineModule {
        file_id,
        items: Vec::new(),
        span: Span::new(file_id, 0, source.len() as u32),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MachineDiagnosticKind;

    #[test]
    fn parses_empty_machine_module() {
        let module = parse_machine_module(FileId(0), " \n\t ").expect("empty module should parse");

        assert_eq!(module.file_id, FileId(0));
        assert!(module.items.is_empty());
        assert_eq!(module.span, Span::new(FileId(0), 0, 4));
    }

    #[test]
    fn rejects_non_empty_input_until_m2() {
        let err = parse_machine_module(FileId(0), "def id").expect_err("M1 parser is skeleton");

        assert_eq!(err.kind, MachineDiagnosticKind::UnsupportedSyntax);
    }
}
