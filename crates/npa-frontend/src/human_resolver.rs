use crate::{HumanCompileOptions, HumanDiagnostic, HumanModule, HumanResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedHumanModule {
    pub module: HumanModule,
}

pub fn resolve_human_module(
    module: HumanModule,
    _options: &HumanCompileOptions,
) -> HumanResult<ResolvedHumanModule> {
    Err(HumanDiagnostic::not_implemented(
        module.span,
        "resolve_human_module",
    ))
}
