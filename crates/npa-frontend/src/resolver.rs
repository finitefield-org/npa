use crate::{MachineModule, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedImport {
    pub module: npa_cert::ModuleName,
    pub export_hash: npa_cert::Hash,
    pub certificate_hash: Option<npa_cert::Hash>,
}

impl From<&npa_cert::VerifiedModule> for VerifiedImport {
    fn from(module: &npa_cert::VerifiedModule) -> Self {
        Self {
            module: module.module.clone(),
            export_hash: module.export_hash,
            certificate_hash: Some(module.certificate_hash),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMachineModule {
    pub module: MachineModule,
}

pub fn resolve_machine_module(
    module: MachineModule,
    _verified_imports: &[VerifiedImport],
) -> Result<ResolvedMachineModule> {
    Ok(ResolvedMachineModule { module })
}
