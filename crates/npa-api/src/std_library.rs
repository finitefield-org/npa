use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

use npa_cert::{
    decode_module_cert, verify_module_cert, AxiomPolicy, CertError, Hash, ImportEntry, ModuleCert,
    Name, TrustMode, VerifiedModule, VerifierSession,
};
use sha2::{Digest, Sha256};

use crate::types::{phase5_name_canonical_bytes, MachineWireGrammarError};

const STD_LOGIC_PATH: &str = "Std/Logic.npcert";
const STD_NAT_PATH: &str = "Std/Nat.npcert";
const STD_LIST_PATH: &str = "Std/List.npcert";
const STD_ALGEBRA_BASIC_PATH: &str = "Std/Algebra/Basic.npcert";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdModuleLocator {
    pub module: Name,
    pub relative_path: String,
}

impl MachineStdModuleLocator {
    pub fn new(module: Name, relative_path: impl Into<String>) -> Self {
        Self {
            module,
            relative_path: relative_path.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MachineStdLoadedRelease {
    modules: Vec<MachineStdLoadedModule>,
    module_index: BTreeMap<Name, usize>,
    verification_order: Vec<Name>,
}

impl MachineStdLoadedRelease {
    pub fn modules(&self) -> &[MachineStdLoadedModule] {
        &self.modules
    }

    pub fn module(&self, module: &Name) -> Option<&MachineStdLoadedModule> {
        self.module_index
            .get(module)
            .map(|index| &self.modules[*index])
    }

    pub fn verification_order(&self) -> &[Name] {
        &self.verification_order
    }
}

#[derive(Clone, Debug)]
pub struct MachineStdLoadedModule {
    pub module: Name,
    pub locator_path: String,
    pub resolved_path: PathBuf,
    pub certificate_bytes: Vec<u8>,
    pub certificate_bytes_hash: Hash,
    pub expected_export_hash: Hash,
    pub expected_certificate_hash: Hash,
    pub axiom_report_hash: Hash,
    pub imports: Vec<ImportEntry>,
    pub verified_module: VerifiedModule,
}

#[derive(Debug)]
pub enum MachineStdReleaseLoaderError {
    InvalidModuleMembership {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    DuplicateModule {
        module: Name,
    },
    NonCanonicalModuleOrder {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    FixedPathMismatch {
        module: Name,
        expected: String,
        actual: String,
    },
    InvalidLocatorPath {
        path: String,
        reason: MachineStdLocatorPathError,
    },
    InvalidPackageRoot {
        path: PathBuf,
        source: io::Error,
    },
    MissingCertificateFile {
        module: Name,
        path: PathBuf,
        source: io::Error,
    },
    ReadCertificateFile {
        module: Name,
        path: PathBuf,
        source: io::Error,
    },
    SymlinkEscape {
        module: Name,
        path: PathBuf,
        resolved: PathBuf,
        package_root: PathBuf,
    },
    DecodeFailed {
        module: Name,
        source: Box<CertError>,
    },
    ModuleNameMismatch {
        expected: Name,
        actual: Name,
    },
    MissingImportCertificateHash {
        owner: Name,
        imported_module: Name,
    },
    UnresolvedImport {
        owner: Name,
        imported_module: Name,
    },
    ImportHashMismatch {
        owner: Name,
        imported_module: Name,
    },
    ImportCycle {
        module: Name,
    },
    InvalidCanonicalModuleName {
        module: Name,
        source: Box<MachineWireGrammarError>,
    },
    VerifyFailed {
        module: Name,
        source: Box<CertError>,
    },
    VerifiedIdentityMismatch {
        module: Name,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStdLocatorPathError {
    Empty,
    Absolute,
    Backslash,
    TrailingSlash,
    DuplicateSlash,
    DotComponent,
    ParentComponent,
}

#[derive(Clone, Debug)]
struct DecodedStdModule {
    locator: MachineStdModuleLocator,
    resolved_path: PathBuf,
    certificate_bytes: Vec<u8>,
    certificate_bytes_hash: Hash,
    cert: ModuleCert,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CertificateKey {
    module: Name,
    export_hash: Hash,
    certificate_hash: Hash,
}

pub fn machine_std_mvp_module_locators() -> Vec<MachineStdModuleLocator> {
    vec![
        MachineStdModuleLocator::new(Name::from_dotted("Std.Nat"), STD_NAT_PATH),
        MachineStdModuleLocator::new(Name::from_dotted("Std.List"), STD_LIST_PATH),
        MachineStdModuleLocator::new(Name::from_dotted("Std.Logic"), STD_LOGIC_PATH),
        MachineStdModuleLocator::new(
            Name::from_dotted("Std.Algebra.Basic"),
            STD_ALGEBRA_BASIC_PATH,
        ),
    ]
}

pub fn load_machine_std_mvp_certificates(
    package_root: impl AsRef<Path>,
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    load_machine_std_certificates_from_locators(package_root, &machine_std_mvp_module_locators())
}

pub fn load_machine_std_certificates_from_locators(
    package_root: impl AsRef<Path>,
    locators: &[MachineStdModuleLocator],
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    validate_machine_std_mvp_locators(locators)?;
    let package_root = canonical_package_root(package_root.as_ref())?;
    let decoded = read_and_decode_std_modules(&package_root, locators)?;
    validate_import_graph(&decoded)?;
    let verification_order = topological_verification_order(&decoded)?;
    verify_decoded_modules(decoded, verification_order)
}

pub fn validate_machine_std_mvp_locators(
    locators: &[MachineStdModuleLocator],
) -> Result<(), MachineStdReleaseLoaderError> {
    let expected = machine_std_mvp_module_locators();
    let expected_modules = expected
        .iter()
        .map(|locator| locator.module.clone())
        .collect::<Vec<_>>();
    let actual_modules = locators
        .iter()
        .map(|locator| locator.module.clone())
        .collect::<Vec<_>>();

    let mut seen = BTreeSet::new();
    for module in &actual_modules {
        if !seen.insert(module.clone()) {
            return Err(MachineStdReleaseLoaderError::DuplicateModule {
                module: module.clone(),
            });
        }
    }

    let expected_set = expected_modules.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual_modules.iter().cloned().collect::<BTreeSet<_>>();
    if actual_set != expected_set {
        return Err(MachineStdReleaseLoaderError::InvalidModuleMembership {
            expected: expected_modules,
            actual: actual_modules,
        });
    }
    if actual_modules != expected_modules {
        return Err(MachineStdReleaseLoaderError::NonCanonicalModuleOrder {
            expected: expected_modules,
            actual: actual_modules,
        });
    }

    for (locator, expected_locator) in locators.iter().zip(expected.iter()) {
        validate_machine_std_locator_path(&locator.relative_path).map_err(|reason| {
            MachineStdReleaseLoaderError::InvalidLocatorPath {
                path: locator.relative_path.clone(),
                reason,
            }
        })?;
        if locator.relative_path != expected_locator.relative_path {
            return Err(MachineStdReleaseLoaderError::FixedPathMismatch {
                module: locator.module.clone(),
                expected: expected_locator.relative_path.clone(),
                actual: locator.relative_path.clone(),
            });
        }
    }

    Ok(())
}

pub fn validate_machine_std_locator_path(path: &str) -> Result<(), MachineStdLocatorPathError> {
    if path.is_empty() {
        return Err(MachineStdLocatorPathError::Empty);
    }
    if path.starts_with('/') {
        return Err(MachineStdLocatorPathError::Absolute);
    }
    if path.contains('\\') {
        return Err(MachineStdLocatorPathError::Backslash);
    }
    if path.ends_with('/') {
        return Err(MachineStdLocatorPathError::TrailingSlash);
    }
    if path.contains("//") {
        return Err(MachineStdLocatorPathError::DuplicateSlash);
    }
    for component in path.split('/') {
        match component {
            "" => return Err(MachineStdLocatorPathError::Empty),
            "." => return Err(MachineStdLocatorPathError::DotComponent),
            ".." => return Err(MachineStdLocatorPathError::ParentComponent),
            _ => {}
        }
    }
    Ok(())
}

fn canonical_package_root(package_root: &Path) -> Result<PathBuf, MachineStdReleaseLoaderError> {
    fs::canonicalize(package_root).map_err(|source| {
        MachineStdReleaseLoaderError::InvalidPackageRoot {
            path: package_root.to_path_buf(),
            source,
        }
    })
}

fn read_and_decode_std_modules(
    package_root: &Path,
    locators: &[MachineStdModuleLocator],
) -> Result<BTreeMap<Name, DecodedStdModule>, MachineStdReleaseLoaderError> {
    let mut decoded = BTreeMap::new();
    for locator in locators {
        let (resolved_path, certificate_bytes) = read_locator_certificate(package_root, locator)?;
        let certificate_bytes_hash = sha256(&certificate_bytes);
        let cert = decode_module_cert(&certificate_bytes).map_err(|source| {
            MachineStdReleaseLoaderError::DecodeFailed {
                module: locator.module.clone(),
                source: Box::new(source),
            }
        })?;
        if cert.header.module != locator.module {
            return Err(MachineStdReleaseLoaderError::ModuleNameMismatch {
                expected: locator.module.clone(),
                actual: cert.header.module.clone(),
            });
        }
        decoded.insert(
            locator.module.clone(),
            DecodedStdModule {
                locator: locator.clone(),
                resolved_path,
                certificate_bytes,
                certificate_bytes_hash,
                cert,
            },
        );
    }
    Ok(decoded)
}

fn read_locator_certificate(
    package_root: &Path,
    locator: &MachineStdModuleLocator,
) -> Result<(PathBuf, Vec<u8>), MachineStdReleaseLoaderError> {
    let path = join_posix_relative_path(package_root, &locator.relative_path);
    let resolved = fs::canonicalize(&path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            MachineStdReleaseLoaderError::MissingCertificateFile {
                module: locator.module.clone(),
                path: path.clone(),
                source,
            }
        } else {
            MachineStdReleaseLoaderError::ReadCertificateFile {
                module: locator.module.clone(),
                path: path.clone(),
                source,
            }
        }
    })?;
    if !resolved.starts_with(package_root) {
        return Err(MachineStdReleaseLoaderError::SymlinkEscape {
            module: locator.module.clone(),
            path,
            resolved,
            package_root: package_root.to_path_buf(),
        });
    }
    let bytes = fs::read(&resolved).map_err(|source| {
        MachineStdReleaseLoaderError::ReadCertificateFile {
            module: locator.module.clone(),
            path: resolved.clone(),
            source,
        }
    })?;
    Ok((resolved, bytes))
}

fn join_posix_relative_path(root: &Path, relative_path: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for component in relative_path.split('/') {
        path.push(component);
    }
    path
}

fn validate_import_graph(
    modules: &BTreeMap<Name, DecodedStdModule>,
) -> Result<(), MachineStdReleaseLoaderError> {
    let mut keys = BTreeSet::new();
    for module in modules.values() {
        keys.insert(CertificateKey {
            module: module.cert.header.module.clone(),
            export_hash: module.cert.hashes.export_hash,
            certificate_hash: module.cert.hashes.certificate_hash,
        });
    }

    for module in modules.values() {
        for import in &module.cert.imports {
            let certificate_hash = import.certificate_hash.ok_or_else(|| {
                MachineStdReleaseLoaderError::MissingImportCertificateHash {
                    owner: module.cert.header.module.clone(),
                    imported_module: import.module.clone(),
                }
            })?;
            if !modules.contains_key(&import.module) {
                return Err(MachineStdReleaseLoaderError::UnresolvedImport {
                    owner: module.cert.header.module.clone(),
                    imported_module: import.module.clone(),
                });
            }
            let key = CertificateKey {
                module: import.module.clone(),
                export_hash: import.export_hash,
                certificate_hash,
            };
            if !keys.contains(&key) {
                return Err(MachineStdReleaseLoaderError::ImportHashMismatch {
                    owner: module.cert.header.module.clone(),
                    imported_module: import.module.clone(),
                });
            }
        }
    }
    Ok(())
}

fn topological_verification_order(
    modules: &BTreeMap<Name, DecodedStdModule>,
) -> Result<Vec<Name>, MachineStdReleaseLoaderError> {
    let mut remaining = modules.keys().cloned().collect::<BTreeSet<_>>();
    let mut order = Vec::new();

    while !remaining.is_empty() {
        let mut ready = Vec::new();
        for module in &remaining {
            let record = modules
                .get(module)
                .expect("remaining module came from decoded module table");
            if record
                .cert
                .imports
                .iter()
                .all(|import| !remaining.contains(&import.module))
            {
                ready.push(module.clone());
            }
        }

        let next = ready
            .into_iter()
            .min_by(compare_module_names)
            .ok_or_else(|| {
                let module = remaining
                    .iter()
                    .min_by(|lhs, rhs| compare_module_names(lhs, rhs))
                    .expect("remaining is non-empty")
                    .clone();
                MachineStdReleaseLoaderError::ImportCycle { module }
            })?;
        remaining.remove(&next);
        order.push(next);
    }

    Ok(order)
}

fn compare_module_names(lhs: &Name, rhs: &Name) -> std::cmp::Ordering {
    let lhs_bytes = module_name_canonical_bytes(lhs).unwrap_or_default();
    let rhs_bytes = module_name_canonical_bytes(rhs).unwrap_or_default();
    lhs_bytes.cmp(&rhs_bytes)
}

fn module_name_canonical_bytes(module: &Name) -> Result<Vec<u8>, MachineStdReleaseLoaderError> {
    phase5_name_canonical_bytes(module).map_err(|source| {
        MachineStdReleaseLoaderError::InvalidCanonicalModuleName {
            module: module.clone(),
            source: Box::new(source),
        }
    })
}

fn verify_decoded_modules(
    decoded: BTreeMap<Name, DecodedStdModule>,
    verification_order: Vec<Name>,
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    let policy = AxiomPolicy {
        mode: TrustMode::HighTrust,
        allowlisted_axioms: BTreeSet::new(),
        deny_sorry: true,
    };
    let mut session = VerifierSession::new();
    let mut verified_by_module = BTreeMap::new();

    for module in &verification_order {
        let record = decoded
            .get(module)
            .expect("verification order came from decoded module table");
        let verified = verify_module_cert(&record.certificate_bytes, &mut session, &policy)
            .map_err(|source| MachineStdReleaseLoaderError::VerifyFailed {
                module: module.clone(),
                source: Box::new(source),
            })?;
        if verified.module() != module
            || verified.export_hash() != record.cert.hashes.export_hash
            || verified.certificate_hash() != record.cert.hashes.certificate_hash
        {
            return Err(MachineStdReleaseLoaderError::VerifiedIdentityMismatch {
                module: module.clone(),
            });
        }
        verified_by_module.insert(module.clone(), verified);
    }

    let mut modules = Vec::new();
    for locator in machine_std_mvp_module_locators() {
        let record = decoded
            .get(&locator.module)
            .expect("validated locators contain every MVP module");
        let verified_module = verified_by_module
            .remove(&locator.module)
            .expect("every decoded module was verified");
        modules.push(MachineStdLoadedModule {
            module: locator.module.clone(),
            locator_path: record.locator.relative_path.clone(),
            resolved_path: record.resolved_path.clone(),
            certificate_bytes: record.certificate_bytes.clone(),
            certificate_bytes_hash: record.certificate_bytes_hash,
            expected_export_hash: record.cert.hashes.export_hash,
            expected_certificate_hash: record.cert.hashes.certificate_hash,
            axiom_report_hash: record.cert.hashes.axiom_report_hash,
            imports: verified_module.imports().to_vec(),
            verified_module,
        });
    }

    let module_index = modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module.clone(), index))
        .collect();

    Ok(MachineStdLoadedRelease {
        modules,
        module_index,
        verification_order,
    })
}

fn sha256(bytes: &[u8]) -> Hash {
    let digest = Sha256::digest(bytes);
    let mut out = [0; 32];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use npa_cert::{build_module_cert, encode_module_cert, CoreModule};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn loads_valid_mvp_certificate_package() {
        let package = TestPackage::new("valid_mvp_certificate_package");
        write_valid_mvp_package(package.path());

        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        assert_eq!(loaded.modules().len(), 4);
        assert_eq!(
            loaded
                .modules()
                .iter()
                .map(|module| module.module.as_dotted())
                .collect::<Vec<_>>(),
            vec!["Std.Nat", "Std.List", "Std.Logic", "Std.Algebra.Basic"]
        );
        assert_eq!(
            loaded
                .verification_order()
                .iter()
                .map(Name::as_dotted)
                .collect::<Vec<_>>(),
            vec!["Std.Logic", "Std.Nat", "Std.List", "Std.Algebra.Basic"]
        );
        for module in loaded.modules() {
            assert_eq!(
                module.certificate_bytes_hash,
                sha256(&module.certificate_bytes)
            );
            assert_eq!(
                module.expected_certificate_hash,
                module.verified_module.certificate_hash()
            );
            assert_eq!(
                module.expected_export_hash,
                module.verified_module.export_hash()
            );
        }
    }

    #[test]
    fn rejects_bad_mvp_locator_membership_order_and_path() {
        let mut missing = machine_std_mvp_module_locators();
        missing.pop();
        assert!(matches!(
            validate_machine_std_mvp_locators(&missing),
            Err(MachineStdReleaseLoaderError::InvalidModuleMembership { .. })
        ));

        let mut extra = machine_std_mvp_module_locators();
        extra.push(MachineStdModuleLocator::new(
            Name::from_dotted("Std.Extra"),
            "Std/Extra.npcert",
        ));
        assert!(matches!(
            validate_machine_std_mvp_locators(&extra),
            Err(MachineStdReleaseLoaderError::InvalidModuleMembership { .. })
        ));

        let mut reordered = machine_std_mvp_module_locators();
        reordered.swap(0, 1);
        assert!(matches!(
            validate_machine_std_mvp_locators(&reordered),
            Err(MachineStdReleaseLoaderError::NonCanonicalModuleOrder { .. })
        ));

        let mut wrong_path = machine_std_mvp_module_locators();
        wrong_path[0].relative_path = "Std/NatWrong.npcert".to_owned();
        assert!(matches!(
            validate_machine_std_mvp_locators(&wrong_path),
            Err(MachineStdReleaseLoaderError::FixedPathMismatch { .. })
        ));
    }

    #[test]
    fn rejects_invalid_locator_paths() {
        let cases = [
            ("", MachineStdLocatorPathError::Empty),
            ("/Std/Nat.npcert", MachineStdLocatorPathError::Absolute),
            ("Std\\Nat.npcert", MachineStdLocatorPathError::Backslash),
            ("Std/Nat.npcert/", MachineStdLocatorPathError::TrailingSlash),
            (
                "Std//Nat.npcert",
                MachineStdLocatorPathError::DuplicateSlash,
            ),
            ("Std/./Nat.npcert", MachineStdLocatorPathError::DotComponent),
            (
                "Std/../Nat.npcert",
                MachineStdLocatorPathError::ParentComponent,
            ),
        ];
        for (path, expected) in cases {
            assert_eq!(validate_machine_std_locator_path(path), Err(expected));
        }
    }

    #[test]
    fn rejects_missing_certificate_file() {
        let package = TestPackage::new("missing_certificate_file");
        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::MissingCertificateFile { .. }
        ));
    }

    #[test]
    fn rejects_core_or_prelude_import_entry_as_unresolved_release_import() {
        let package = TestPackage::new("core_import_entry");
        let mut certs = mvp_certificate_bytes();
        let mut logic = decode_module_cert(certs.logic.as_slice()).unwrap();
        logic.imports.push(ImportEntry {
            module: Name::from_dotted("Core"),
            export_hash: [0; 32],
            certificate_hash: Some([0; 32]),
        });
        certs.logic = encode_module_cert(&logic).unwrap();
        write_mvp_package(package.path(), &certs);

        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::UnresolvedImport {
                imported_module,
                ..
            } if imported_module == Name::from_dotted("Core")
        ));
    }

    #[test]
    fn rejects_release_import_cycles_before_verification() {
        let package = TestPackage::new("import_cycle");
        let mut certs = mvp_certificate_bytes();
        let mut logic = decode_module_cert(certs.logic.as_slice()).unwrap();
        let mut nat = decode_module_cert(certs.nat.as_slice()).unwrap();
        logic.imports.push(ImportEntry {
            module: Name::from_dotted("Std.Nat"),
            export_hash: nat.hashes.export_hash,
            certificate_hash: Some(nat.hashes.certificate_hash),
        });
        nat.imports.push(ImportEntry {
            module: Name::from_dotted("Std.Logic"),
            export_hash: logic.hashes.export_hash,
            certificate_hash: Some(logic.hashes.certificate_hash),
        });
        certs.logic = encode_module_cert(&logic).unwrap();
        certs.nat = encode_module_cert(&nat).unwrap();
        write_mvp_package(package.path(), &certs);

        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::ImportCycle { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape_from_package_root() {
        use std::os::unix::fs::symlink;

        let package = TestPackage::new("symlink_escape");
        let outside = TestPackage::new("symlink_escape_outside");
        let certs = mvp_certificate_bytes();
        write_mvp_package(package.path(), &certs);
        fs::write(outside.path().join("Logic.npcert"), &certs.logic).unwrap();
        fs::remove_file(package.path().join(STD_LOGIC_PATH)).unwrap();
        symlink(
            outside.path().join("Logic.npcert"),
            package.path().join(STD_LOGIC_PATH),
        )
        .unwrap();

        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::SymlinkEscape { .. }
        ));
    }

    struct MvpCertificateBytes {
        logic: Vec<u8>,
        nat: Vec<u8>,
        list: Vec<u8>,
        algebra_basic: Vec<u8>,
    }

    fn mvp_certificate_bytes() -> MvpCertificateBytes {
        let mut session = VerifierSession::new();
        let policy = AxiomPolicy::high_trust();

        let logic_cert = build_module_cert(empty_module("Std.Logic"), &[]).unwrap();
        let logic = encode_module_cert(&logic_cert).unwrap();
        let logic_verified = verify_module_cert(&logic, &mut session, &policy).unwrap();

        let nat_cert = build_module_cert(
            empty_module("Std.Nat"),
            std::slice::from_ref(&logic_verified),
        )
        .unwrap();
        let nat = encode_module_cert(&nat_cert).unwrap();
        let nat_verified = verify_module_cert(&nat, &mut session, &policy).unwrap();

        let list_cert = build_module_cert(
            empty_module("Std.List"),
            &[logic_verified.clone(), nat_verified.clone()],
        )
        .unwrap();
        let list = encode_module_cert(&list_cert).unwrap();
        verify_module_cert(&list, &mut session, &policy).unwrap();

        let algebra_cert =
            build_module_cert(empty_module("Std.Algebra.Basic"), &[logic_verified]).unwrap();
        let algebra_basic = encode_module_cert(&algebra_cert).unwrap();

        MvpCertificateBytes {
            logic,
            nat,
            list,
            algebra_basic,
        }
    }

    fn write_valid_mvp_package(root: &Path) {
        let certs = mvp_certificate_bytes();
        write_mvp_package(root, &certs);
    }

    fn write_mvp_package(root: &Path, certs: &MvpCertificateBytes) {
        write_cert(root, STD_LOGIC_PATH, &certs.logic);
        write_cert(root, STD_NAT_PATH, &certs.nat);
        write_cert(root, STD_LIST_PATH, &certs.list);
        write_cert(root, STD_ALGEBRA_BASIC_PATH, &certs.algebra_basic);
    }

    fn write_cert(root: &Path, relative_path: &str, bytes: &[u8]) {
        let path = join_posix_relative_path(root, relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn empty_module(name: &str) -> CoreModule {
        CoreModule {
            name: Name::from_dotted(name),
            declarations: Vec::new(),
        }
    }

    struct TestPackage {
        path: PathBuf,
    }

    impl TestPackage {
        fn new(label: &str) -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            path.push(format!(
                "npa-stdlib-loader-{label}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestPackage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
