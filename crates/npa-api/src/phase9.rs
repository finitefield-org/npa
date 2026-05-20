use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};

use npa_cert::{Hash, ModuleName, Name};
use npa_kernel::Expr;
use npa_tactic::VerifiedImportRef;
use sha2::{Digest, Sha256};

use crate::types::phase5_name_canonical_bytes;

const CANDIDATE_HASH_TAG: &str = "npa.phase9_ai.candidate.v1";
const OPTIONS_HASH_TAG: &str = "npa.phase9_ai.options.v1";
const ENV_FINGERPRINT_TAG: &str = "npa.phase9_ai.env.v1";
const VALIDATION_RESULT_HASH_TAG: &str = "npa.phase9_ai.validation_result.v1";

const MAX_OPTIONS_BYTES: usize = 16_000_000;
const MAX_PHASE9_GLOBAL_REFS: u64 = 65_536;
const MAX_NAME_COMPONENTS: u64 = 256;
const MAX_STRING_BYTES: u64 = 1_048_576;

pub const PHASE9_INDUCTIVE_CHECK_ENDPOINT: &str = "/machine/phase9/inductive/check";
pub const PHASE9_UNIVERSE_REPAIR_CHECK_ENDPOINT: &str = "/machine/phase9/universe/repair/check";
pub const PHASE9_TYPECLASS_RESOLVE_ENDPOINT: &str = "/machine/phase9/typeclass/resolve";
pub const PHASE9_QUOTIENT_CHECK_ENDPOINT: &str = "/machine/phase9/quotient/check";
pub const PHASE9_SMT_RECONSTRUCT_ENDPOINT: &str = "/machine/phase9/smt/reconstruct";
pub const PHASE9_THEOREM_GRAPH_QUERY_ENDPOINT: &str = "/machine/phase9/theorem-graph/query";
pub const PHASE9_FORMALIZE_CHECK_ENDPOINT: &str = "/machine/phase9/formalize/check";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiProfileVersion {
    MvpV1,
}

impl Phase9AiProfileVersion {
    fn tag(self) -> u8 {
        match self {
            Self::MvpV1 => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpV1),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiTaskKind {
    AdvancedInductive,
    UniverseRepair,
    TypeclassResolution,
    QuotientConstruction,
    SmtCertificate,
    TheoremGraphQuery,
    NaturalLanguageFormalization,
}

impl Phase9AiTaskKind {
    fn tag(self) -> u8 {
        match self {
            Self::AdvancedInductive => 0,
            Self::UniverseRepair => 1,
            Self::TypeclassResolution => 2,
            Self::QuotientConstruction => 3,
            Self::SmtCertificate => 4,
            Self::TheoremGraphQuery => 5,
            Self::NaturalLanguageFormalization => 6,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::AdvancedInductive),
            1 => Some(Self::UniverseRepair),
            2 => Some(Self::TypeclassResolution),
            3 => Some(Self::QuotientConstruction),
            4 => Some(Self::SmtCertificate),
            5 => Some(Self::TheoremGraphQuery),
            6 => Some(Self::NaturalLanguageFormalization),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiTarget {
    pub env_fingerprint: Hash,
    pub target_decl_hash: Option<Hash>,
    pub goal_fingerprint: Option<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9ImportIdentity {
    pub module: ModuleName,
    pub export_hash: Hash,
    pub certificate_hash: Hash,
}

impl Phase9ImportIdentity {
    pub fn from_verified_import(import: &VerifiedImportRef) -> Self {
        Self {
            module: import.module().clone(),
            export_hash: import.export_hash(),
            certificate_hash: import.certificate_hash(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9AiOptionsRef {
    Inline {
        options_hash: Hash,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: String,
        file_hash: Hash,
        options_hash: Hash,
        size_bytes: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiCandidateEnvelope {
    pub profile_version: Phase9AiProfileVersion,
    pub task_kind: Phase9AiTaskKind,
    pub target: Phase9AiTarget,
    pub imports: Vec<Phase9ImportIdentity>,
    pub options: Phase9AiOptionsRef,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiOptionsVersion {
    MvpV1,
}

impl Phase9AiOptionsVersion {
    fn tag(self) -> u8 {
        match self {
            Self::MvpV1 => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpV1),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9IndependentCheckerProfile {
    Phase8MvpReference,
}

impl Phase9IndependentCheckerProfile {
    fn tag(self) -> u8 {
        match self {
            Self::Phase8MvpReference => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Phase8MvpReference),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9IndependentCheckerOptions {
    pub profile: Phase9IndependentCheckerProfile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AdvancedInductiveOptions {
    pub approved_nested_type_constructors: Vec<Phase9AiGlobalRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9TypeclassOptions {
    pub class_declarations: Vec<Phase9AiGlobalRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9QuotientOptions {
    pub setoid: Phase9AiGlobalRef,
    pub setoid_mk: Phase9AiGlobalRef,
    pub setoid_relation: Phase9AiGlobalRef,
    pub rel_equiv: Phase9AiGlobalRef,
    pub quotient: Phase9AiGlobalRef,
    pub quotient_mk: Phase9AiGlobalRef,
    pub quotient_sound: Phase9AiGlobalRef,
    pub quotient_lift: Phase9AiGlobalRef,
    pub eq: Phase9AiGlobalRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9SmtOptions {
    pub eq: Phase9AiGlobalRef,
    pub prop_false: Option<Phase9AiGlobalRef>,
    pub prop_not: Option<Phase9AiGlobalRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9FormalizationOptions {
    pub tactic_options_canonical_bytes: Vec<u8>,
    pub tactic_budget_canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiOptions {
    pub schema_version: Phase9AiOptionsVersion,
    pub independent_checker: Phase9IndependentCheckerOptions,
    pub advanced_inductive: Phase9AdvancedInductiveOptions,
    pub typeclass: Phase9TypeclassOptions,
    pub quotient: Option<Phase9QuotientOptions>,
    pub smt: Option<Phase9SmtOptions>,
    pub formalization: Option<Phase9FormalizationOptions>,
}

impl Default for Phase9AiOptions {
    fn default() -> Self {
        Self {
            schema_version: Phase9AiOptionsVersion::MvpV1,
            independent_checker: Phase9IndependentCheckerOptions {
                profile: Phase9IndependentCheckerProfile::Phase8MvpReference,
            },
            advanced_inductive: Phase9AdvancedInductiveOptions {
                approved_nested_type_constructors: Vec::new(),
            },
            typeclass: Phase9TypeclassOptions {
                class_declarations: Vec::new(),
            },
            quotient: None,
            smt: None,
            formalization: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiGlobalRef {
    pub module: ModuleName,
    pub export_hash: Hash,
    pub certificate_hash: Hash,
    pub name: Name,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiValidationError {
    EnvelopeMalformed,
    TargetFingerprintMismatch,
    ImportClosureMismatch,
    PayloadHashMismatch,
    KernelRejected,
    IndependentCheckerRejected,
    NonDeterministicResult,
    BudgetExceeded,
    AmbiguousResolution,
    NoSolution,
    FeatureRejected,
    UnsupportedFeature,
}

impl Phase9AiValidationError {
    fn tag(self) -> u8 {
        match self {
            Self::EnvelopeMalformed => 0,
            Self::TargetFingerprintMismatch => 1,
            Self::ImportClosureMismatch => 2,
            Self::PayloadHashMismatch => 3,
            Self::KernelRejected => 4,
            Self::IndependentCheckerRejected => 5,
            Self::NonDeterministicResult => 6,
            Self::BudgetExceeded => 7,
            Self::AmbiguousResolution => 8,
            Self::NoSolution => 9,
            Self::FeatureRejected => 10,
            Self::UnsupportedFeature => 11,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiEndpointError {
    NonCanonicalRequestBytes,
    ArtifactUnavailable,
    InternalValidatorFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiFeatureError {
    AdvancedInductive(Phase9AdvancedInductiveError),
    UniverseRepair(Phase9UniverseRepairError),
    TypeclassResolution(Phase9TypeclassResolutionError),
    QuotientConstruction(Phase9QuotientConstructionError),
    SmtCertificate(Phase9SmtCertificateError),
    TheoremGraphQuery(Phase9TheoremGraphError),
    Formalization(Phase9FormalizationError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AdvancedInductiveError {
    TargetRefMismatch,
    PositivityProfileUnsupported,
    ArtifactGeneratorUnavailable,
    GeneratedArtifactMismatch,
    NameCollision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9UniverseRepairError {
    UnknownUniverseParam,
    IllFormedLevelExpr,
    UnsatisfiedConstraint,
    NonCanonicalSolution,
    TargetFingerprintMismatch,
    InvalidOccurrencePath,
    AmbiguousOccurrence,
    TargetRefMismatch,
    ConstraintHintMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TypeclassResolutionError {
    ClassDeclarationMismatch,
    CandidateInterfaceInvalid,
    ClassHeadUnsupported,
    NoSolution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9QuotientConstructionError {
    TargetRefMismatch,
    PrimitiveInterfaceMismatch,
    UniverseLevelMismatch,
    CompatibilityProofMismatch,
    QuotientTypeMismatch,
    RelationTypeMismatch,
    EquivalenceProofMismatch,
    RawFunctionTypeMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9SmtCertificateError {
    EncodingMismatch,
    RuleFingerprintMismatch,
    RuleRegistryMismatch,
    NonCanonicalPayload,
    ReconstructionProofMismatch,
    ConclusionEncodingMismatch,
    PayloadBindingMismatch,
    ReconstructionConclusionMismatch,
    ReconstructionPremiseMismatch,
    PublicInterfaceMismatch,
    TheoryRefMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphError {
    SnapshotMalformed,
    QueryFeaturesMalformed,
    NodeResolutionMismatch,
    LimitOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9FormalizationError {
    IntentRecordMismatch,
    CandidateStatementElaborationFailed,
    FormalizationProofStatementMismatch,
    RejectedIntentHasProofCandidate,
    ProofBridgeFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9AiSuccessPayload {
    AdvancedInductive {
        decl_interface_hash: Hash,
        decl_certificate_hash: Hash,
    },
    UniverseRepair {
        repaired_expr: Expr,
        constraint_set_hash: Hash,
    },
    TypeclassResolution {
        proof: Expr,
    },
    QuotientConstruction {
        decl_certificate_hash: Hash,
    },
    SmtCertificate {
        final_proof: Expr,
    },
    TheoremGraphQuery {
        result: Phase9MachineTheoremGraphResult,
    },
    NaturalLanguageFormalization {
        kind: Phase9FormalizationSuccessKind,
        accepted_statement_hash: Option<Hash>,
        formalization_proof_root_hash: Option<Hash>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphResult {
    pub result_hash: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9FormalizationSuccessKind {
    CandidateStatementChecked,
    IntentRecordOnly,
    ProofBridgeChecked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9AiEndpointResponse {
    Success {
        candidate_hash: Hash,
        validation_result_hash: Hash,
        payload: Box<Phase9AiSuccessPayload>,
    },
    Rejected {
        candidate_hash: Hash,
        validation_result_hash: Hash,
        error: Phase9AiValidationError,
        feature_error: Option<Phase9AiFeatureError>,
    },
    Error {
        error: Phase9AiEndpointError,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9ValidatedCommonEnvelope {
    pub candidate_hash: Hash,
    pub options_hash: Hash,
    pub env_fingerprint: Hash,
    pub envelope: Phase9AiCandidateEnvelope,
    pub options: Phase9AiOptions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiCanonicalError {
    InvalidName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecodeError {
    Malformed,
}

pub fn phase9_ai_candidate_hash(envelope_canonical_bytes: &[u8]) -> Hash {
    hash_with_domain(CANDIDATE_HASH_TAG, envelope_canonical_bytes)
}

pub fn phase9_ai_options_hash(options_canonical_bytes: &[u8]) -> Hash {
    hash_with_domain(OPTIONS_HASH_TAG, options_canonical_bytes)
}

pub fn phase9_file_hash(bytes: &[u8]) -> Hash {
    sha256(bytes)
}

pub fn phase9_ai_validation_result_hash_for_rejection(
    candidate_hash: Hash,
    error: Phase9AiValidationError,
    feature_error: Option<Phase9AiFeatureError>,
) -> Hash {
    let mut payload = Vec::new();
    payload.push(1);
    encode_validation_error_to(&mut payload, error);
    encode_feature_error_option_to(&mut payload, feature_error);
    validation_result_hash(candidate_hash, &payload)
}

pub fn phase9_ai_env_fingerprint(
    profile_version: Phase9AiProfileVersion,
    task_kind: Phase9AiTaskKind,
    imports: &[Phase9ImportIdentity],
    options_hash: Hash,
) -> std::result::Result<Hash, Phase9AiCanonicalError> {
    let mut payload = Vec::new();
    payload.push(profile_version.tag());
    payload.push(task_kind.tag());
    encode_import_identities_to(&mut payload, imports)?;
    encode_hash_to(&mut payload, &options_hash);
    Ok(hash_with_domain(ENV_FINGERPRINT_TAG, &payload))
}

pub fn phase9_ai_options_canonical_bytes(
    options: &Phase9AiOptions,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_options_to(&mut out, options)?;
    Ok(out)
}

pub fn phase9_ai_candidate_envelope_canonical_bytes(
    envelope: &Phase9AiCandidateEnvelope,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_candidate_envelope_to(&mut out, envelope)?;
    Ok(out)
}

pub fn validate_phase9_ai_common_envelope(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
    expected_task_kind: Phase9AiTaskKind,
) -> std::result::Result<Phase9ValidatedCommonEnvelope, Phase9AiEndpointResponse> {
    let envelope = match decode_candidate_envelope(request_canonical_bytes) {
        Ok(envelope) => envelope,
        Err(_) => {
            return Err(Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::NonCanonicalRequestBytes,
            });
        }
    };
    let candidate_hash = phase9_ai_candidate_hash(request_canonical_bytes);

    if envelope.task_kind != expected_task_kind {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        ));
    }

    validate_imports(candidate_hash, &envelope.imports, verified_imports)?;

    let (options, options_hash) =
        validate_options_ref(candidate_hash, &envelope.options, workspace_root)?;

    if !options
        .advanced_inductive
        .approved_nested_type_constructors
        .is_empty()
    {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        ));
    }

    let env_fingerprint = phase9_ai_env_fingerprint(
        envelope.profile_version,
        envelope.task_kind,
        &envelope.imports,
        options_hash,
    )
    .map_err(|_| {
        rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        )
    })?;

    if envelope.target.env_fingerprint != env_fingerprint {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        ));
    }

    validate_target_shape(candidate_hash, envelope.task_kind, &envelope.target)?;
    validate_required_options(candidate_hash, envelope.task_kind, &options)?;

    Ok(Phase9ValidatedCommonEnvelope {
        candidate_hash,
        options_hash,
        env_fingerprint,
        envelope,
        options,
    })
}

pub fn run_phase9_inductive_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::AdvancedInductive,
    )
}

pub fn run_phase9_universe_repair_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::UniverseRepair,
    )
}

pub fn run_phase9_typeclass_resolve_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::TypeclassResolution,
    )
}

pub fn run_phase9_quotient_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::QuotientConstruction,
    )
}

pub fn run_phase9_smt_reconstruct_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::SmtCertificate,
    )
}

pub fn run_phase9_theorem_graph_query_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::TheoremGraphQuery,
    )
}

pub fn run_phase9_formalize_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::NaturalLanguageFormalization,
    )
}

fn run_phase9_skeleton_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
    expected_task_kind: Phase9AiTaskKind,
) -> Phase9AiEndpointResponse {
    match validate_phase9_ai_common_envelope(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        expected_task_kind,
    ) {
        Ok(validated) => rejected_response(
            validated.candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            None,
        ),
        Err(response) => response,
    }
}

fn validate_imports(
    candidate_hash: Hash,
    imports: &[Phase9ImportIdentity],
    verified_imports: &[VerifiedImportRef],
) -> std::result::Result<(), Phase9AiEndpointResponse> {
    let mut previous: Option<&Phase9ImportIdentity> = None;
    for import in imports {
        if !import.module.is_canonical() {
            return Err(rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            ));
        }
        if let Some(previous) = previous {
            match compare_import_identities(previous, import) {
                Ok(Ordering::Greater) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::EnvelopeMalformed,
                        None,
                    ));
                }
                Ok(Ordering::Equal) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::ImportClosureMismatch,
                        None,
                    ));
                }
                Ok(Ordering::Less) => {}
                Err(_) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::EnvelopeMalformed,
                        None,
                    ));
                }
            }
        }
        previous = Some(import);
    }

    if imports.len() != verified_imports.len() {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        ));
    }

    for (expected, actual) in imports.iter().zip(verified_imports) {
        let actual = Phase9ImportIdentity::from_verified_import(actual);
        if expected != &actual {
            return Err(rejected_response(
                candidate_hash,
                Phase9AiValidationError::ImportClosureMismatch,
                None,
            ));
        }
    }

    Ok(())
}

fn validate_options_ref(
    candidate_hash: Hash,
    options_ref: &Phase9AiOptionsRef,
    workspace_root: &Path,
) -> std::result::Result<(Phase9AiOptions, Hash), Phase9AiEndpointResponse> {
    let (declared_options_hash, canonical_bytes) = match options_ref {
        Phase9AiOptionsRef::Inline {
            options_hash,
            canonical_bytes,
        } => {
            if canonical_bytes.len() > MAX_OPTIONS_BYTES {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::EnvelopeMalformed,
                    None,
                ));
            }
            (*options_hash, canonical_bytes.clone())
        }
        Phase9AiOptionsRef::Artifact {
            path,
            file_hash,
            options_hash,
            size_bytes,
        } => {
            if usize::try_from(*size_bytes)
                .map(|size| size > MAX_OPTIONS_BYTES)
                .unwrap_or(true)
            {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::EnvelopeMalformed,
                    None,
                ));
            }
            let path = match validate_artifact_path(workspace_root, path) {
                Ok(path) => path,
                Err(ArtifactPathError::EnvelopeMalformed) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::EnvelopeMalformed,
                        None,
                    ));
                }
                Err(ArtifactPathError::ArtifactUnavailable) => {
                    return Err(Phase9AiEndpointResponse::Error {
                        error: Phase9AiEndpointError::ArtifactUnavailable,
                    });
                }
            };
            let bytes = std::fs::read(path).map_err(|_| Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::ArtifactUnavailable,
            })?;
            if bytes.len() as u64 != *size_bytes || phase9_file_hash(&bytes) != *file_hash {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::PayloadHashMismatch,
                    None,
                ));
            }
            (*options_hash, bytes)
        }
    };

    let options = decode_options(&canonical_bytes).map_err(|_| {
        rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        )
    })?;
    let actual_options_hash = phase9_ai_options_hash(&canonical_bytes);
    if actual_options_hash != declared_options_hash {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        ));
    }

    Ok((options, actual_options_hash))
}

fn validate_target_shape(
    candidate_hash: Hash,
    task_kind: Phase9AiTaskKind,
    target: &Phase9AiTarget,
) -> std::result::Result<(), Phase9AiEndpointResponse> {
    let valid = match task_kind {
        Phase9AiTaskKind::AdvancedInductive
        | Phase9AiTaskKind::QuotientConstruction
        | Phase9AiTaskKind::NaturalLanguageFormalization => {
            target.target_decl_hash.is_none() && target.goal_fingerprint.is_none()
        }
        Phase9AiTaskKind::UniverseRepair => {
            if target.target_decl_hash.is_some() && target.goal_fingerprint.is_none() {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::UnsupportedFeature,
                    None,
                ));
            }
            target.target_decl_hash.is_none() && target.goal_fingerprint.is_some()
        }
        Phase9AiTaskKind::TypeclassResolution
        | Phase9AiTaskKind::SmtCertificate
        | Phase9AiTaskKind::TheoremGraphQuery => {
            target.target_decl_hash.is_none() && target.goal_fingerprint.is_some()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        ))
    }
}

fn validate_required_options(
    candidate_hash: Hash,
    task_kind: Phase9AiTaskKind,
    options: &Phase9AiOptions,
) -> std::result::Result<(), Phase9AiEndpointResponse> {
    let valid = match task_kind {
        Phase9AiTaskKind::QuotientConstruction => options.quotient.is_some(),
        Phase9AiTaskKind::SmtCertificate => options.smt.is_some(),
        Phase9AiTaskKind::NaturalLanguageFormalization => options.formalization.is_some(),
        Phase9AiTaskKind::AdvancedInductive
        | Phase9AiTaskKind::UniverseRepair
        | Phase9AiTaskKind::TypeclassResolution
        | Phase9AiTaskKind::TheoremGraphQuery => true,
    };
    if valid {
        Ok(())
    } else {
        Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        ))
    }
}

fn rejected_response(
    candidate_hash: Hash,
    error: Phase9AiValidationError,
    feature_error: Option<Phase9AiFeatureError>,
) -> Phase9AiEndpointResponse {
    Phase9AiEndpointResponse::Rejected {
        candidate_hash,
        validation_result_hash: phase9_ai_validation_result_hash_for_rejection(
            candidate_hash,
            error,
            feature_error,
        ),
        error,
        feature_error,
    }
}

fn validation_result_hash(candidate_hash: Hash, payload: &[u8]) -> Hash {
    let mut bytes = Vec::new();
    encode_hash_to(&mut bytes, &candidate_hash);
    bytes.extend_from_slice(payload);
    hash_with_domain(VALIDATION_RESULT_HASH_TAG, &bytes)
}

fn encode_candidate_envelope_to(
    out: &mut Vec<u8>,
    envelope: &Phase9AiCandidateEnvelope,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    out.push(envelope.profile_version.tag());
    out.push(envelope.task_kind.tag());
    encode_target_to(out, &envelope.target);
    encode_import_identities_to(out, &envelope.imports)?;
    encode_options_ref_to(out, &envelope.options);
    encode_bytes_to(out, &envelope.payload);
    Ok(())
}

fn encode_target_to(out: &mut Vec<u8>, target: &Phase9AiTarget) {
    encode_hash_to(out, &target.env_fingerprint);
    encode_option_hash_to(out, target.target_decl_hash.as_ref());
    encode_option_hash_to(out, target.goal_fingerprint.as_ref());
}

fn encode_import_identities_to(
    out: &mut Vec<u8>,
    imports: &[Phase9ImportIdentity],
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_len_to(out, imports.len());
    for import in imports {
        encode_name_to(out, &import.module)?;
        encode_hash_to(out, &import.export_hash);
        encode_hash_to(out, &import.certificate_hash);
    }
    Ok(())
}

fn encode_options_ref_to(out: &mut Vec<u8>, options_ref: &Phase9AiOptionsRef) {
    match options_ref {
        Phase9AiOptionsRef::Inline {
            options_hash,
            canonical_bytes,
        } => {
            out.push(0);
            encode_hash_to(out, options_hash);
            encode_bytes_to(out, canonical_bytes);
        }
        Phase9AiOptionsRef::Artifact {
            path,
            file_hash,
            options_hash,
            size_bytes,
        } => {
            out.push(1);
            encode_string_to(out, path);
            encode_hash_to(out, file_hash);
            encode_hash_to(out, options_hash);
            encode_u64_to(out, *size_bytes);
        }
    }
}

fn encode_options_to(
    out: &mut Vec<u8>,
    options: &Phase9AiOptions,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    out.push(options.schema_version.tag());
    out.push(options.independent_checker.profile.tag());
    encode_global_ref_list_to(
        out,
        &options.advanced_inductive.approved_nested_type_constructors,
    )?;
    encode_global_ref_list_to(out, &options.typeclass.class_declarations)?;
    encode_option_quotient_to(out, options.quotient.as_ref())?;
    encode_option_smt_to(out, options.smt.as_ref())?;
    encode_option_formalization_to(out, options.formalization.as_ref());
    Ok(())
}

fn encode_global_ref_list_to(
    out: &mut Vec<u8>,
    refs: &[Phase9AiGlobalRef],
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_len_to(out, refs.len());
    for global_ref in refs {
        encode_global_ref_to(out, global_ref)?;
    }
    Ok(())
}

fn encode_global_ref_to(
    out: &mut Vec<u8>,
    global_ref: &Phase9AiGlobalRef,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_name_to(out, &global_ref.module)?;
    encode_hash_to(out, &global_ref.export_hash);
    encode_hash_to(out, &global_ref.certificate_hash);
    encode_name_to(out, &global_ref.name)?;
    encode_hash_to(out, &global_ref.decl_interface_hash);
    Ok(())
}

fn encode_option_quotient_to(
    out: &mut Vec<u8>,
    options: Option<&Phase9QuotientOptions>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match options {
        Some(options) => {
            out.push(1);
            encode_global_ref_to(out, &options.setoid)?;
            encode_global_ref_to(out, &options.setoid_mk)?;
            encode_global_ref_to(out, &options.setoid_relation)?;
            encode_global_ref_to(out, &options.rel_equiv)?;
            encode_global_ref_to(out, &options.quotient)?;
            encode_global_ref_to(out, &options.quotient_mk)?;
            encode_global_ref_to(out, &options.quotient_sound)?;
            encode_global_ref_to(out, &options.quotient_lift)?;
            encode_global_ref_to(out, &options.eq)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_option_smt_to(
    out: &mut Vec<u8>,
    options: Option<&Phase9SmtOptions>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match options {
        Some(options) => {
            out.push(1);
            encode_global_ref_to(out, &options.eq)?;
            encode_option_global_ref_to(out, options.prop_false.as_ref())?;
            encode_option_global_ref_to(out, options.prop_not.as_ref())?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_option_global_ref_to(
    out: &mut Vec<u8>,
    global_ref: Option<&Phase9AiGlobalRef>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match global_ref {
        Some(global_ref) => {
            out.push(1);
            encode_global_ref_to(out, global_ref)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_option_formalization_to(out: &mut Vec<u8>, options: Option<&Phase9FormalizationOptions>) {
    match options {
        Some(options) => {
            out.push(1);
            encode_bytes_to(out, &options.tactic_options_canonical_bytes);
            encode_bytes_to(out, &options.tactic_budget_canonical_bytes);
        }
        None => out.push(0),
    }
}

fn decode_candidate_envelope(
    input: &[u8],
) -> std::result::Result<Phase9AiCandidateEnvelope, DecodeError> {
    let mut decoder = Decoder::new(input);
    let profile_version =
        Phase9AiProfileVersion::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let task_kind = Phase9AiTaskKind::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let target = decoder.target()?;
    let imports = decoder.import_identities()?;
    let options = decoder.options_ref()?;
    let payload = decoder.bytes()?;
    decoder.done()?;

    let envelope = Phase9AiCandidateEnvelope {
        profile_version,
        task_kind,
        target,
        imports,
        options,
        payload,
    };
    let encoded = phase9_ai_candidate_envelope_canonical_bytes(&envelope)
        .map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(envelope)
}

fn decode_options(input: &[u8]) -> std::result::Result<Phase9AiOptions, DecodeError> {
    let mut decoder = Decoder::new(input);
    let schema_version =
        Phase9AiOptionsVersion::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let independent_checker = Phase9IndependentCheckerOptions {
        profile: Phase9IndependentCheckerProfile::from_tag(decoder.u8()?)
            .ok_or(DecodeError::Malformed)?,
    };
    let approved_nested_type_constructors = decoder.global_ref_list()?;
    ensure_sorted_global_refs(&approved_nested_type_constructors)?;
    let class_declarations = decoder.global_ref_list()?;
    ensure_sorted_global_refs(&class_declarations)?;
    let quotient = decoder.option_quotient()?;
    let smt = decoder.option_smt()?;
    let formalization = decoder.option_formalization()?;
    decoder.done()?;

    let options = Phase9AiOptions {
        schema_version,
        independent_checker,
        advanced_inductive: Phase9AdvancedInductiveOptions {
            approved_nested_type_constructors,
        },
        typeclass: Phase9TypeclassOptions { class_declarations },
        quotient,
        smt,
        formalization,
    };
    let encoded =
        phase9_ai_options_canonical_bytes(&options).map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(options)
}

struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn done(&self) -> std::result::Result<(), DecodeError> {
        if self.pos == self.input.len() {
            Ok(())
        } else {
            Err(DecodeError::Malformed)
        }
    }

    fn u8(&mut self) -> std::result::Result<u8, DecodeError> {
        let value = *self.input.get(self.pos).ok_or(DecodeError::Malformed)?;
        self.pos += 1;
        Ok(value)
    }

    fn u64(&mut self) -> std::result::Result<u64, DecodeError> {
        let end = self.pos.checked_add(8).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn hash(&mut self) -> std::result::Result<Hash, DecodeError> {
        let end = self.pos.checked_add(32).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(bytes.try_into().unwrap())
    }

    fn bytes(&mut self) -> std::result::Result<Vec<u8>, DecodeError> {
        let len = usize::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        let end = self.pos.checked_add(len).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(bytes.to_vec())
    }

    fn string(&mut self) -> std::result::Result<String, DecodeError> {
        let bytes = self.bytes()?;
        if bytes.len() as u64 > MAX_STRING_BYTES {
            return Err(DecodeError::Malformed);
        }
        String::from_utf8(bytes).map_err(|_| DecodeError::Malformed)
    }

    fn name(&mut self) -> std::result::Result<Name, DecodeError> {
        let len = self.u64()?;
        if len == 0 || len > MAX_NAME_COMPONENTS {
            return Err(DecodeError::Malformed);
        }
        let mut components = Vec::new();
        for _ in 0..len {
            let component = self.string()?;
            components.push(component);
        }
        let name = Name(components);
        if name.is_canonical() {
            Ok(name)
        } else {
            Err(DecodeError::Malformed)
        }
    }

    fn target(&mut self) -> std::result::Result<Phase9AiTarget, DecodeError> {
        Ok(Phase9AiTarget {
            env_fingerprint: self.hash()?,
            target_decl_hash: self.option_hash()?,
            goal_fingerprint: self.option_hash()?,
        })
    }

    fn option_hash(&mut self) -> std::result::Result<Option<Hash>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.hash()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn import_identities(&mut self) -> std::result::Result<Vec<Phase9ImportIdentity>, DecodeError> {
        let len = usize::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        let mut imports = Vec::new();
        for _ in 0..len {
            imports.push(Phase9ImportIdentity {
                module: self.name()?,
                export_hash: self.hash()?,
                certificate_hash: self.hash()?,
            });
        }
        Ok(imports)
    }

    fn options_ref(&mut self) -> std::result::Result<Phase9AiOptionsRef, DecodeError> {
        match self.u8()? {
            0 => Ok(Phase9AiOptionsRef::Inline {
                options_hash: self.hash()?,
                canonical_bytes: self.bytes()?,
            }),
            1 => Ok(Phase9AiOptionsRef::Artifact {
                path: self.string()?,
                file_hash: self.hash()?,
                options_hash: self.hash()?,
                size_bytes: self.u64()?,
            }),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn global_ref_list(&mut self) -> std::result::Result<Vec<Phase9AiGlobalRef>, DecodeError> {
        let len = self.u64()?;
        if len > MAX_PHASE9_GLOBAL_REFS {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut refs = Vec::with_capacity(len);
        for _ in 0..len {
            refs.push(self.global_ref()?);
        }
        Ok(refs)
    }

    fn global_ref(&mut self) -> std::result::Result<Phase9AiGlobalRef, DecodeError> {
        Ok(Phase9AiGlobalRef {
            module: self.name()?,
            export_hash: self.hash()?,
            certificate_hash: self.hash()?,
            name: self.name()?,
            decl_interface_hash: self.hash()?,
        })
    }

    fn option_quotient(
        &mut self,
    ) -> std::result::Result<Option<Phase9QuotientOptions>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Phase9QuotientOptions {
                setoid: self.global_ref()?,
                setoid_mk: self.global_ref()?,
                setoid_relation: self.global_ref()?,
                rel_equiv: self.global_ref()?,
                quotient: self.global_ref()?,
                quotient_mk: self.global_ref()?,
                quotient_sound: self.global_ref()?,
                quotient_lift: self.global_ref()?,
                eq: self.global_ref()?,
            })),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_smt(&mut self) -> std::result::Result<Option<Phase9SmtOptions>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Phase9SmtOptions {
                eq: self.global_ref()?,
                prop_false: self.option_global_ref()?,
                prop_not: self.option_global_ref()?,
            })),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_global_ref(&mut self) -> std::result::Result<Option<Phase9AiGlobalRef>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.global_ref()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_formalization(
        &mut self,
    ) -> std::result::Result<Option<Phase9FormalizationOptions>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Phase9FormalizationOptions {
                tactic_options_canonical_bytes: self.bytes()?,
                tactic_budget_canonical_bytes: self.bytes()?,
            })),
            _ => Err(DecodeError::Malformed),
        }
    }
}

fn ensure_sorted_global_refs(refs: &[Phase9AiGlobalRef]) -> std::result::Result<(), DecodeError> {
    let mut previous: Option<Vec<u8>> = None;
    for global_ref in refs {
        let key = global_ref_sort_key(global_ref).map_err(|_| DecodeError::Malformed)?;
        if let Some(previous) = previous.as_ref() {
            if previous >= &key {
                return Err(DecodeError::Malformed);
            }
        }
        previous = Some(key);
    }
    Ok(())
}

fn compare_import_identities(
    left: &Phase9ImportIdentity,
    right: &Phase9ImportIdentity,
) -> std::result::Result<Ordering, Phase9AiCanonicalError> {
    Ok(import_identity_sort_key(left)?.cmp(&import_identity_sort_key(right)?))
}

fn import_identity_sort_key(
    import: &Phase9ImportIdentity,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut key = phase5_name_canonical_bytes(&import.module)
        .map_err(|_| Phase9AiCanonicalError::InvalidName)?;
    key.extend_from_slice(&import.export_hash);
    key.extend_from_slice(&import.certificate_hash);
    Ok(key)
}

fn global_ref_sort_key(
    global_ref: &Phase9AiGlobalRef,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut key = phase5_name_canonical_bytes(&global_ref.module)
        .map_err(|_| Phase9AiCanonicalError::InvalidName)?;
    key.extend_from_slice(&global_ref.export_hash);
    key.extend_from_slice(&global_ref.certificate_hash);
    key.extend_from_slice(
        &phase5_name_canonical_bytes(&global_ref.name)
            .map_err(|_| Phase9AiCanonicalError::InvalidName)?,
    );
    key.extend_from_slice(&global_ref.decl_interface_hash);
    Ok(key)
}

fn encode_validation_error_to(out: &mut Vec<u8>, error: Phase9AiValidationError) {
    out.push(error.tag());
}

fn encode_feature_error_option_to(out: &mut Vec<u8>, feature: Option<Phase9AiFeatureError>) {
    match feature {
        Some(feature) => {
            out.push(1);
            encode_feature_error_to(out, feature);
        }
        None => out.push(0),
    }
}

fn encode_feature_error_to(out: &mut Vec<u8>, feature: Phase9AiFeatureError) {
    match feature {
        Phase9AiFeatureError::AdvancedInductive(error) => {
            out.push(0);
            out.push(match error {
                Phase9AdvancedInductiveError::TargetRefMismatch => 0,
                Phase9AdvancedInductiveError::PositivityProfileUnsupported => 1,
                Phase9AdvancedInductiveError::ArtifactGeneratorUnavailable => 2,
                Phase9AdvancedInductiveError::GeneratedArtifactMismatch => 3,
                Phase9AdvancedInductiveError::NameCollision => 4,
            });
        }
        Phase9AiFeatureError::UniverseRepair(error) => {
            out.push(1);
            out.push(match error {
                Phase9UniverseRepairError::UnknownUniverseParam => 0,
                Phase9UniverseRepairError::IllFormedLevelExpr => 1,
                Phase9UniverseRepairError::UnsatisfiedConstraint => 2,
                Phase9UniverseRepairError::NonCanonicalSolution => 3,
                Phase9UniverseRepairError::TargetFingerprintMismatch => 4,
                Phase9UniverseRepairError::InvalidOccurrencePath => 5,
                Phase9UniverseRepairError::AmbiguousOccurrence => 6,
                Phase9UniverseRepairError::TargetRefMismatch => 7,
                Phase9UniverseRepairError::ConstraintHintMismatch => 8,
            });
        }
        Phase9AiFeatureError::TypeclassResolution(error) => {
            out.push(2);
            out.push(match error {
                Phase9TypeclassResolutionError::ClassDeclarationMismatch => 0,
                Phase9TypeclassResolutionError::CandidateInterfaceInvalid => 1,
                Phase9TypeclassResolutionError::ClassHeadUnsupported => 2,
                Phase9TypeclassResolutionError::NoSolution => 3,
            });
        }
        Phase9AiFeatureError::QuotientConstruction(error) => {
            out.push(3);
            out.push(match error {
                Phase9QuotientConstructionError::TargetRefMismatch => 0,
                Phase9QuotientConstructionError::PrimitiveInterfaceMismatch => 1,
                Phase9QuotientConstructionError::UniverseLevelMismatch => 2,
                Phase9QuotientConstructionError::CompatibilityProofMismatch => 3,
                Phase9QuotientConstructionError::QuotientTypeMismatch => 4,
                Phase9QuotientConstructionError::RelationTypeMismatch => 5,
                Phase9QuotientConstructionError::EquivalenceProofMismatch => 6,
                Phase9QuotientConstructionError::RawFunctionTypeMismatch => 7,
            });
        }
        Phase9AiFeatureError::SmtCertificate(error) => {
            out.push(4);
            out.push(match error {
                Phase9SmtCertificateError::EncodingMismatch => 0,
                Phase9SmtCertificateError::RuleFingerprintMismatch => 1,
                Phase9SmtCertificateError::RuleRegistryMismatch => 2,
                Phase9SmtCertificateError::NonCanonicalPayload => 3,
                Phase9SmtCertificateError::ReconstructionProofMismatch => 4,
                Phase9SmtCertificateError::ConclusionEncodingMismatch => 5,
                Phase9SmtCertificateError::PayloadBindingMismatch => 6,
                Phase9SmtCertificateError::ReconstructionConclusionMismatch => 7,
                Phase9SmtCertificateError::ReconstructionPremiseMismatch => 8,
                Phase9SmtCertificateError::PublicInterfaceMismatch => 9,
                Phase9SmtCertificateError::TheoryRefMismatch => 10,
            });
        }
        Phase9AiFeatureError::TheoremGraphQuery(error) => {
            out.push(5);
            out.push(match error {
                Phase9TheoremGraphError::SnapshotMalformed => 0,
                Phase9TheoremGraphError::QueryFeaturesMalformed => 1,
                Phase9TheoremGraphError::NodeResolutionMismatch => 2,
                Phase9TheoremGraphError::LimitOutOfRange => 3,
            });
        }
        Phase9AiFeatureError::Formalization(error) => {
            out.push(6);
            out.push(match error {
                Phase9FormalizationError::IntentRecordMismatch => 0,
                Phase9FormalizationError::CandidateStatementElaborationFailed => 1,
                Phase9FormalizationError::FormalizationProofStatementMismatch => 2,
                Phase9FormalizationError::RejectedIntentHasProofCandidate => 3,
                Phase9FormalizationError::ProofBridgeFailed => 4,
            });
        }
    }
}

fn encode_name_to(
    out: &mut Vec<u8>,
    name: &Name,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    if !name.is_canonical() {
        return Err(Phase9AiCanonicalError::InvalidName);
    }
    encode_len_to(out, name.0.len());
    for component in &name.0 {
        encode_string_to(out, component);
    }
    Ok(())
}

fn encode_option_hash_to(out: &mut Vec<u8>, hash: Option<&Hash>) {
    match hash {
        Some(hash) => {
            out.push(1);
            encode_hash_to(out, hash);
        }
        None => out.push(0),
    }
}

fn encode_hash_to(out: &mut Vec<u8>, hash: &Hash) {
    out.extend_from_slice(hash);
}

fn encode_bytes_to(out: &mut Vec<u8>, bytes: &[u8]) {
    encode_len_to(out, bytes.len());
    out.extend_from_slice(bytes);
}

fn encode_string_to(out: &mut Vec<u8>, value: &str) {
    encode_bytes_to(out, value.as_bytes());
}

fn encode_len_to(out: &mut Vec<u8>, len: usize) {
    encode_u64_to(out, len as u64);
}

fn encode_u64_to(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn hash_with_domain(domain: &str, payload: &[u8]) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain.as_bytes());
    bytes.extend_from_slice(payload);
    sha256(&bytes)
}

fn sha256(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArtifactPathError {
    EnvelopeMalformed,
    ArtifactUnavailable,
}

fn validate_artifact_path(
    workspace_root: &Path,
    path: &str,
) -> std::result::Result<PathBuf, ArtifactPathError> {
    if path.is_empty() || path.as_bytes().contains(&0) {
        return Err(ArtifactPathError::EnvelopeMalformed);
    }
    if path
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(ArtifactPathError::EnvelopeMalformed);
    }
    let relative = Path::new(path);
    if relative.is_absolute() {
        return Err(ArtifactPathError::EnvelopeMalformed);
    }
    for component in relative.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(ArtifactPathError::EnvelopeMalformed);
            }
        }
    }

    let root = workspace_root
        .canonicalize()
        .map_err(|_| ArtifactPathError::ArtifactUnavailable)?;
    let mut current = root.clone();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(ArtifactPathError::EnvelopeMalformed);
        };
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let resolved = current
                    .canonicalize()
                    .map_err(|_| ArtifactPathError::ArtifactUnavailable)?;
                if !resolved.starts_with(&root) {
                    return Err(ArtifactPathError::EnvelopeMalformed);
                }
                current = resolved;
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    Ok(workspace_root.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn hash(byte: u8) -> Hash {
        [byte; 32]
    }

    fn empty_options_bytes() -> Vec<u8> {
        phase9_ai_options_canonical_bytes(&Phase9AiOptions::default()).unwrap()
    }

    fn global_ref(seed: u8) -> Phase9AiGlobalRef {
        Phase9AiGlobalRef {
            module: Name::from_dotted("Std.Prim"),
            export_hash: hash(seed),
            certificate_hash: hash(seed.wrapping_add(1)),
            name: Name::from_dotted(format!("ref{seed}")),
            decl_interface_hash: hash(seed.wrapping_add(2)),
        }
    }

    fn target_for(
        task_kind: Phase9AiTaskKind,
        imports: &[Phase9ImportIdentity],
        options_hash: Hash,
        goal_fingerprint: Option<Hash>,
    ) -> Phase9AiTarget {
        Phase9AiTarget {
            env_fingerprint: phase9_ai_env_fingerprint(
                Phase9AiProfileVersion::MvpV1,
                task_kind,
                imports,
                options_hash,
            )
            .unwrap(),
            target_decl_hash: None,
            goal_fingerprint,
        }
    }

    fn inline_request(
        task_kind: Phase9AiTaskKind,
        options_bytes: Vec<u8>,
        imports: Vec<Phase9ImportIdentity>,
        goal_fingerprint: Option<Hash>,
    ) -> Vec<u8> {
        let options_hash = phase9_ai_options_hash(&options_bytes);
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind,
            target: target_for(task_kind, &imports, options_hash, goal_fingerprint),
            imports,
            options: Phase9AiOptionsRef::Inline {
                options_hash,
                canonical_bytes: options_bytes,
            },
            payload: b"opaque-payload".to_vec(),
        };
        phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap()
    }

    fn workspace_root() -> PathBuf {
        std::env::current_dir().unwrap()
    }

    fn assert_rejected(
        response: Phase9AiEndpointResponse,
        expected_error: Phase9AiValidationError,
        expected_feature_error: Option<Phase9AiFeatureError>,
    ) -> Hash {
        match response {
            Phase9AiEndpointResponse::Rejected {
                candidate_hash,
                validation_result_hash,
                error,
                feature_error,
            } => {
                assert_eq!(error, expected_error);
                assert_eq!(feature_error, expected_feature_error);
                assert_eq!(
                    validation_result_hash,
                    phase9_ai_validation_result_hash_for_rejection(
                        candidate_hash,
                        error,
                        feature_error
                    )
                );
                candidate_hash
            }
            other => panic!("expected rejected response, got {other:?}"),
        }
    }

    #[test]
    fn common_candidate_hash_is_available_when_options_decode_fails() {
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            b"not-options".to_vec(),
            Vec::new(),
            None,
        );
        let expected_candidate_hash = phase9_ai_candidate_hash(&request);

        let candidate_hash = assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );

        assert_eq!(candidate_hash, expected_candidate_hash);
    }

    #[test]
    fn top_level_decode_failure_is_endpoint_error_without_candidate_hash() {
        assert_eq!(
            run_phase9_inductive_check_request(b"not-an-envelope", &[], &workspace_root()),
            Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::NonCanonicalRequestBytes
            }
        );
    }

    #[test]
    fn options_hash_mismatch_is_payload_hash_mismatch() {
        let options_bytes = empty_options_bytes();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: target_for(Phase9AiTaskKind::AdvancedInductive, &[], hash(9), None),
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Inline {
                options_hash: hash(9),
                canonical_bytes: options_bytes,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
    }

    #[test]
    fn quotient_options_round_trip_named_primitive_refs() {
        let options = Phase9AiOptions {
            quotient: Some(Phase9QuotientOptions {
                setoid: global_ref(1),
                setoid_mk: global_ref(4),
                setoid_relation: global_ref(7),
                rel_equiv: global_ref(10),
                quotient: global_ref(13),
                quotient_mk: global_ref(16),
                quotient_sound: global_ref(19),
                quotient_lift: global_ref(22),
                eq: global_ref(25),
            }),
            ..Default::default()
        };
        let bytes = phase9_ai_options_canonical_bytes(&options).unwrap();

        assert_eq!(decode_options(&bytes).unwrap(), options);

        let mut changed = options.clone();
        changed.quotient.as_mut().unwrap().eq.decl_interface_hash = hash(99);
        assert_ne!(phase9_ai_options_canonical_bytes(&changed).unwrap(), bytes);
    }

    #[test]
    fn formalization_options_preserve_nested_phase4_bytes() {
        let options = Phase9AiOptions {
            formalization: Some(Phase9FormalizationOptions {
                tactic_options_canonical_bytes: b"phase4-options".to_vec(),
                tactic_budget_canonical_bytes: b"phase4-budget".to_vec(),
            }),
            ..Default::default()
        };
        let bytes = phase9_ai_options_canonical_bytes(&options).unwrap();

        assert_eq!(decode_options(&bytes).unwrap(), options);

        let mut changed = options.clone();
        changed
            .formalization
            .as_mut()
            .unwrap()
            .tactic_budget_canonical_bytes
            .push(0);
        assert_ne!(phase9_ai_options_canonical_bytes(&changed).unwrap(), bytes);
    }

    #[test]
    fn phase9_domain_hashes_use_documented_tag_concatenation() {
        let payload = b"payload";
        let mut expected = Vec::new();
        expected.extend_from_slice(CANDIDATE_HASH_TAG.as_bytes());
        expected.extend_from_slice(payload);

        assert_eq!(phase9_ai_candidate_hash(payload), sha256(&expected));
    }

    #[test]
    fn artifact_hash_and_size_mismatch_is_candidate_rejection() {
        let root = std::env::temp_dir().join(format!("npa-phase9-m1-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("options.bin"), empty_options_bytes()).unwrap();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: hash(1),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Artifact {
                path: "options.bin".to_owned(),
                file_hash: hash(2),
                options_hash: phase9_ai_options_hash(&empty_options_bytes()),
                size_bytes: empty_options_bytes().len() as u64,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &root),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn artifact_path_shape_failure_is_candidate_rejection() {
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: hash(1),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Artifact {
                path: "../options.bin".to_owned(),
                file_hash: hash(2),
                options_hash: phase9_ai_options_hash(&empty_options_bytes()),
                size_bytes: empty_options_bytes().len() as u64,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }

    #[cfg(unix)]
    #[test]
    fn artifact_symlink_escape_is_candidate_rejection() {
        let root =
            std::env::temp_dir().join(format!("npa-phase9-symlink-root-{}", std::process::id()));
        let outside =
            std::env::temp_dir().join(format!("npa-phase9-symlink-outside-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, empty_options_bytes()).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("escaped-options.bin")).unwrap();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: hash(1),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Artifact {
                path: "escaped-options.bin".to_owned(),
                file_hash: phase9_file_hash(&empty_options_bytes()),
                options_hash: phase9_ai_options_hash(&empty_options_bytes()),
                size_bytes: empty_options_bytes().len() as u64,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &root),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn duplicate_import_identity_is_import_closure_mismatch() {
        let import = Phase9ImportIdentity {
            module: Name::from_dotted("A"),
            export_hash: hash(1),
            certificate_hash: hash(2),
        };
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            vec![import.clone(), import],
            None,
        );

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        );
    }

    #[test]
    fn import_sort_order_uses_phase5_name_canonical_bytes() {
        let import_b = Phase9ImportIdentity {
            module: Name::from_dotted("B"),
            export_hash: hash(1),
            certificate_hash: hash(2),
        };
        let import_aa = Phase9ImportIdentity {
            module: Name::from_dotted("AA"),
            export_hash: hash(3),
            certificate_hash: hash(4),
        };
        assert_eq!(
            compare_import_identities(&import_b, &import_aa).unwrap(),
            Ordering::Less
        );
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            vec![import_aa, import_b],
            None,
        );

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }

    #[test]
    fn env_fingerprint_mismatch_is_target_fingerprint_mismatch() {
        let mut request = decode_candidate_envelope(&inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            Vec::new(),
            None,
        ))
        .unwrap();
        request.target.env_fingerprint = hash(7);
        let request = phase9_ai_candidate_envelope_canonical_bytes(&request).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        );
    }

    #[test]
    fn approved_nested_type_constructor_is_common_unsupported_feature() {
        let mut options = Phase9AiOptions::default();
        options
            .advanced_inductive
            .approved_nested_type_constructors
            .push(Phase9AiGlobalRef {
                module: Name::from_dotted("Std.List"),
                export_hash: hash(1),
                certificate_hash: hash(2),
                name: Name::from_dotted("List"),
                decl_interface_hash: hash(3),
            });
        let options_bytes = phase9_ai_options_canonical_bytes(&options).unwrap();
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            options_bytes,
            Vec::new(),
            None,
        );

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }

    #[test]
    fn route_skeletons_bind_each_endpoint_to_its_task_kind() {
        type Phase9Route = fn(&[u8], &[VerifiedImportRef], &Path) -> Phase9AiEndpointResponse;

        let routes: [(&str, Phase9Route); 7] = [
            (
                PHASE9_INDUCTIVE_CHECK_ENDPOINT,
                run_phase9_inductive_check_request,
            ),
            (
                PHASE9_UNIVERSE_REPAIR_CHECK_ENDPOINT,
                run_phase9_universe_repair_check_request,
            ),
            (
                PHASE9_TYPECLASS_RESOLVE_ENDPOINT,
                run_phase9_typeclass_resolve_request,
            ),
            (
                PHASE9_QUOTIENT_CHECK_ENDPOINT,
                run_phase9_quotient_check_request,
            ),
            (
                PHASE9_SMT_RECONSTRUCT_ENDPOINT,
                run_phase9_smt_reconstruct_request,
            ),
            (
                PHASE9_THEOREM_GRAPH_QUERY_ENDPOINT,
                run_phase9_theorem_graph_query_request,
            ),
            (
                PHASE9_FORMALIZE_CHECK_ENDPOINT,
                run_phase9_formalize_check_request,
            ),
        ];
        assert_eq!(routes.len(), 7);

        let options_bytes = empty_options_bytes();
        let goal = Some(hash(4));
        let universe = inline_request(
            Phase9AiTaskKind::UniverseRepair,
            options_bytes,
            Vec::new(),
            goal,
        );
        assert_rejected(
            run_phase9_inductive_check_request(&universe, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
        assert_rejected(
            run_phase9_universe_repair_check_request(&universe, &[], &workspace_root()),
            Phase9AiValidationError::UnsupportedFeature,
            None,
        );
    }

    #[test]
    fn common_validation_success_is_deterministic_for_same_replay_input() {
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            Vec::new(),
            None,
        );

        let first = run_phase9_inductive_check_request(&request, &[], &workspace_root());
        let second = run_phase9_inductive_check_request(&request, &[], &workspace_root());

        assert_eq!(first, second);
        assert_rejected(first, Phase9AiValidationError::UnsupportedFeature, None);
    }
}
