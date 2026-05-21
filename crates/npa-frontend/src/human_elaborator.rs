use crate::{
    parse_human_module, resolve_human_module, HumanCompileOptions, HumanDiagnostic, HumanResult,
    ResolvedHumanModule, Span, VerifiedImport,
};

pub fn elaborate_human_module(
    _module_name: npa_cert::ModuleName,
    module: ResolvedHumanModule,
    _verified_imports: &[VerifiedImport],
    _options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    Err(HumanDiagnostic::not_implemented(
        module.module.span,
        "elaborate_human_module",
    ))
}

pub fn compile_human_source_to_core(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    let module = parse_human_module(file_id, source)?;
    let resolved = resolve_human_module(module_name.clone(), module, verified_imports, options)?;
    elaborate_human_module(module_name, resolved, verified_imports, options)
}

pub fn compile_human_source_to_certificate(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::ModuleCert> {
    let verified_imports: Vec<_> = verified_modules.iter().map(VerifiedImport::from).collect();
    let _core =
        compile_human_source_to_core(file_id, module_name, source, &verified_imports, options)?;
    Err(HumanDiagnostic::not_implemented(
        source_span(file_id, source),
        "compile_human_source_to_certificate",
    ))
}

fn source_span(file_id: crate::FileId, source: &str) -> Span {
    Span::new(file_id, 0, source.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileId, HumanDiagnosticKind};

    #[test]
    fn compile_human_source_checks_verified_imports_before_elaboration() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Current.Module"),
            "import Std.Nat.Basic",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("missing import should fail during Human resolution");

        assert_eq!(err.kind, HumanDiagnosticKind::MissingVerifiedImport);
    }
}
