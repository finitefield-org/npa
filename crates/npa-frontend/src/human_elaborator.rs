use crate::{
    HumanCompileOptions, HumanDiagnostic, HumanResult, ResolvedHumanModule, Span, VerifiedImport,
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
    _module_name: npa_cert::ModuleName,
    source: &str,
    _verified_imports: &[VerifiedImport],
    _options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    Err(HumanDiagnostic::not_implemented(
        source_span(file_id, source),
        "compile_human_source_to_core",
    ))
}

pub fn compile_human_source_to_certificate(
    file_id: crate::FileId,
    _module_name: npa_cert::ModuleName,
    source: &str,
    _verified_modules: &[npa_cert::VerifiedModule],
    _options: &HumanCompileOptions,
) -> HumanResult<npa_cert::ModuleCert> {
    Err(HumanDiagnostic::not_implemented(
        source_span(file_id, source),
        "compile_human_source_to_certificate",
    ))
}

fn source_span(file_id: crate::FileId, source: &str) -> Span {
    Span::new(file_id, 0, source.len() as u32)
}
