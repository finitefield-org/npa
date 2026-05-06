use crate::{
    parse_machine_module, resolve_machine_module, MachineCompileOptions, MachineDiagnostic,
    MachineDiagnosticKind, MachineLocalDecl, ResolvedMachineModule, Result, VerifiedImport,
};

pub fn elaborate_machine_module(
    module_name: npa_cert::ModuleName,
    module: ResolvedMachineModule,
    _verified_imports: &[VerifiedImport],
    _options: &MachineCompileOptions,
) -> Result<npa_cert::CoreModule> {
    if !module.module.items.is_empty() {
        return Err(MachineDiagnostic::error(
            MachineDiagnosticKind::UnsupportedSyntax,
            module.module.items[0].span(),
            "non-empty Machine Surface modules are implemented in M2-M5",
        ));
    }

    Ok(npa_cert::CoreModule {
        name: module_name,
        declarations: Vec::new(),
    })
}

pub fn compile_machine_source_to_core(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<npa_cert::CoreModule> {
    let module = parse_machine_module(file_id, source)?;
    let resolved = resolve_machine_module(module, verified_imports)?;
    elaborate_machine_module(module_name, resolved, verified_imports, options)
}

pub fn compile_machine_source_to_certificate(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    options: &MachineCompileOptions,
) -> Result<npa_cert::ModuleCert> {
    let verified_imports: Vec<_> = verified_modules.iter().map(VerifiedImport::from).collect();
    let module =
        compile_machine_source_to_core(file_id, module_name, source, &verified_imports, options)?;
    npa_cert::build_module_cert(module, verified_modules).map_err(|err| {
        MachineDiagnostic::error(
            MachineDiagnosticKind::CertificateRejected,
            crate::Span::empty(file_id),
            format!("certificate construction failed: {err:?}"),
        )
    })
}

pub fn elaborate_machine_term_check(
    source: &str,
    _local_context: &[MachineLocalDecl],
    _expected: &npa_kernel::Expr,
    _verified_imports: &[VerifiedImport],
    _options: &MachineCompileOptions,
) -> Result<npa_kernel::Expr> {
    Err(MachineDiagnostic::unsupported_syntax(
        crate::Span::new(crate::FileId(0), 0, source.len() as u32),
        "term-level Machine Surface elaboration is implemented in M7",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileId;

    #[test]
    fn compiles_empty_machine_module_to_empty_core_module() {
        let module = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test.Empty"),
            "",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("empty module should compile in M1");

        assert_eq!(module.name, npa_cert::Name::from_dotted("Test.Empty"));
        assert!(module.declarations.is_empty());
    }
}
