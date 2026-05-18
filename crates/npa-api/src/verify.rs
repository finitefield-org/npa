use std::collections::{BTreeMap, BTreeSet};

use npa_cert::{
    build_module_cert, encode_module_cert, verify_module_cert, AxiomPolicy, AxiomRef, CertError,
    CoreModule, DeclPayload, GlobalRef, Hash, ModuleCert, Name, VerifierSession,
};
use npa_tactic::{
    extract_closed_machine_theorem_decl, MachineTacticDiagnostic, MachineTacticDiagnosticKind,
};

use crate::current::{encode_machine_axiom_ref_wire, MachineAxiomRefWire};
use crate::projection::VerifiedModuleContextEntry;
use crate::snapshot::{MachineSnapshotLookupError, MachineSnapshotMaterializationContext};
use crate::types::{
    HashString, MachineApiEndpoint, MachineApiErrorResponse, MachineApiErrorWire,
    MachineApiOkResponse, MachineApiResponseEnvelope, MachineApiResponseStatus,
    MachineProofSession, SessionId, SnapshotId,
};
use crate::validation::{parse_request_body, MachineApiErrorKind, MachineApiRequestError};
use crate::{
    validate_machine_endpoint_envelope, MachineApiDiagnosticPhase, MachineApiDiagnosticProjection,
    Phase5UpstreamDiagnostic,
};

const CERTIFICATE_ENCODING: &str = "npa.certificate.canonical.v0.1.hex";

pub type MachineVerifyResponse =
    MachineApiResponseEnvelope<MachineVerifyOkFields, MachineApiErrorWire, (), ()>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineVerifyOkFields {
    pub root_decl_interface_hash: Hash,
    pub root_decl_certificate_hash: Hash,
    pub root_axioms_used: Vec<MachineAxiomRefWire>,
    pub module_export_hash: Hash,
    pub module_certificate_hash: Hash,
    pub module_axioms_used: Vec<MachineAxiomRefWire>,
    pub certificate: MachineCertificateWirePayload,
    pub dependency_import_closure: Vec<MachineVerifiedModuleCertificatePayload>,
    pub import_payload: MachineVerifiedModuleCertificatePayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineCertificateWirePayload {
    pub encoding: &'static str,
    pub bytes: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineVerifiedModuleCertificatePayload {
    pub module: Name,
    pub expected_export_hash: Hash,
    pub expected_certificate_hash: Hash,
    pub certificate: MachineCertificateWirePayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineVerifyError {
    pub diagnostic: MachineApiDiagnosticProjection,
    pub response: MachineVerifyResponse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineVerifyRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
}

struct GeneratedCertificateContext<'a> {
    cert: &'a ModuleCert,
    cert_decl_to_source_index: BTreeMap<usize, u64>,
}

pub fn parse_machine_verify_request(
    source: &str,
) -> Result<MachineVerifyRequest, MachineApiRequestError> {
    let doc = parse_request_body(source, MachineApiErrorKind::InvalidVerifyRequest)?;
    let envelope = validate_machine_endpoint_envelope(
        doc.root(),
        MachineApiEndpoint::Verify,
        &crate::JsonPath::root(),
    )?;
    let session_id = SessionId::parse(
        envelope
            .field("session_id")
            .expect("endpoint envelope checked required session_id")
            .string_value()
            .expect("endpoint envelope checked session_id string"),
    )
    .expect("endpoint envelope checked session_id grammar");
    let snapshot_id = SnapshotId::parse(
        envelope
            .field("snapshot_id")
            .expect("endpoint envelope checked required snapshot_id")
            .string_value()
            .expect("endpoint envelope checked snapshot_id string"),
    )
    .expect("endpoint envelope checked snapshot_id grammar");
    let state_fingerprint = HashString::parse(
        envelope
            .field("state_fingerprint")
            .expect("endpoint envelope checked required state_fingerprint")
            .string_value()
            .expect("endpoint envelope checked state_fingerprint string"),
    )
    .expect("endpoint envelope checked state_fingerprint grammar")
    .digest();

    Ok(MachineVerifyRequest {
        session_id,
        snapshot_id,
        state_fingerprint,
    })
}

pub fn run_machine_verify_request(
    source: &str,
    session: &MachineProofSession,
) -> Result<MachineVerifyResponse, Box<MachineVerifyError>> {
    run_machine_verify_request_in_sessions(source, std::iter::once(session))
}

pub fn run_machine_verify_request_in_sessions<'session>(
    source: &str,
    sessions: impl IntoIterator<Item = &'session MachineProofSession>,
) -> Result<MachineVerifyResponse, Box<MachineVerifyError>> {
    let request = parse_machine_verify_request(source).map_err(request_error)?;
    let Some(session) = sessions
        .into_iter()
        .find(|session| session.session_id == request.session_id)
    else {
        return Err(plain_error(
            MachineApiErrorKind::UnknownSession,
            MachineApiDiagnosticPhase::SessionLookup,
            format!("unknown session {}", request.session_id.wire()),
        ));
    };
    run_machine_verify_request_parsed(session, request)
}

fn run_machine_verify_request_parsed(
    session: &MachineProofSession,
    request: MachineVerifyRequest,
) -> Result<MachineVerifyResponse, Box<MachineVerifyError>> {
    if session.snapshots.session_id() != &session.session_id {
        return Err(plain_error(
            MachineApiErrorKind::InvalidMachineProofState,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "session snapshot store belongs to a different session",
        ));
    }

    let context = MachineSnapshotMaterializationContext {
        session_id: &session.session_id,
        display_scope: &session.machine_display_render_scope,
        callable_interface_table: &session.machine_surface_callable_interface_table,
    };
    let entry = session
        .snapshots
        .lookup_checked(&context, request.snapshot_id, request.state_fingerprint)
        .map_err(snapshot_lookup_error)?;
    if !entry.materialized_view_payload.open_goals.is_empty() {
        return Err(plain_error(
            MachineApiErrorKind::InvalidVerifyRequest,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "verify requires a closed snapshot with no open goals",
        ));
    }

    let theorem = extract_closed_machine_theorem_decl(&entry.executable_state_payload)
        .map_err(extraction_error)?;
    let mut declarations = session
        .checked_current_decls
        .checked_current_decls()
        .iter()
        .map(|decl| decl.core_decl().clone())
        .collect::<Vec<_>>();
    declarations.push(theorem);

    let imports = session
        .import_certificate_context
        .verified_modules()
        .iter()
        .map(|entry| entry.verified_module.clone())
        .collect::<Vec<_>>();
    let core_module = CoreModule {
        name: session.root.module.clone(),
        declarations,
    };
    let certificate =
        build_module_cert(core_module, &imports).map_err(certificate_generation_error)?;
    let certificate_bytes =
        encode_module_cert(&certificate).map_err(certificate_generation_error)?;
    let mut verifier_session = VerifierSession::new();
    for import in imports {
        verifier_session.register_verified_module(import);
    }
    let verified_module = verify_module_cert(
        &certificate_bytes,
        &mut verifier_session,
        &AxiomPolicy::normal(),
    )
    .map_err(certificate_verify_error)?;

    if certificate.hashes.export_hash != verified_module.export_hash()
        || certificate.hashes.certificate_hash != verified_module.certificate_hash()
    {
        return Err(plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::CertificateGeneration,
            "generated certificate hashes disagree with verifier output",
        ));
    }

    let generated_context = generated_certificate_context(session, &certificate)?;
    let root_decl_index = root_decl_index(session, &certificate)?;
    let root_decl = certificate
        .declarations
        .get(root_decl_index)
        .ok_or_else(|| {
            plain_error(
                MachineApiErrorKind::VerifyFailed,
                MachineApiDiagnosticPhase::CertificateGeneration,
                "root declaration index is outside generated certificate declarations",
            )
        })?;
    let root_axioms_used = axiom_refs_to_wire(
        &generated_context,
        root_axiom_refs(verified_module.axiom_report(), root_decl_index)?,
    )?;
    let module_axioms_used = axiom_refs_to_wire(
        &generated_context,
        &verified_module.axiom_report().module_axioms,
    )?;
    ensure_axioms_allowed(&session.options.allow_axioms, &module_axioms_used)?;

    let import_payload = MachineVerifiedModuleCertificatePayload {
        module: session.root.module.clone(),
        expected_export_hash: certificate.hashes.export_hash,
        expected_certificate_hash: certificate.hashes.certificate_hash,
        certificate: certificate_payload(&certificate_bytes),
    };

    Ok(MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
        status: MachineApiResponseStatus::Verified,
        endpoint_fields: MachineVerifyOkFields {
            root_decl_interface_hash: root_decl.hashes.decl_interface_hash,
            root_decl_certificate_hash: root_decl.hashes.decl_certificate_hash,
            root_axioms_used,
            module_export_hash: certificate.hashes.export_hash,
            module_certificate_hash: certificate.hashes.certificate_hash,
            module_axioms_used,
            certificate: certificate_payload(&certificate_bytes),
            dependency_import_closure: dependency_import_closure_payloads(session),
            import_payload,
        },
    }))
}

fn generated_certificate_context<'a>(
    session: &MachineProofSession,
    cert: &'a ModuleCert,
) -> Result<GeneratedCertificateContext<'a>, Box<MachineVerifyError>> {
    let mut source_by_name = BTreeMap::new();
    for entry in session.checked_current_decls.decl_index_table() {
        source_by_name.insert(entry.signature.name.clone(), entry.source_index);
    }
    source_by_name.insert(session.root.theorem_name.clone(), session.root.source_index);

    let mut cert_decl_to_source_index = BTreeMap::new();
    for (decl_index, decl) in cert.declarations.iter().enumerate() {
        let name = decl_payload_name(cert, &decl.decl)?;
        let Some(source_index) = source_by_name.get(&name).copied() else {
            return Err(plain_error(
                MachineApiErrorKind::VerifyFailed,
                MachineApiDiagnosticPhase::CertificateGeneration,
                format!(
                    "generated certificate declaration {} has no Phase 5 source_index",
                    name.as_dotted()
                ),
            ));
        };
        if cert_decl_to_source_index
            .insert(decl_index, source_index)
            .is_some()
        {
            return Err(plain_error(
                MachineApiErrorKind::VerifyFailed,
                MachineApiDiagnosticPhase::CertificateGeneration,
                "duplicate generated certificate declaration index",
            ));
        }
    }

    Ok(GeneratedCertificateContext {
        cert,
        cert_decl_to_source_index,
    })
}

fn root_decl_index(
    session: &MachineProofSession,
    cert: &ModuleCert,
) -> Result<usize, Box<MachineVerifyError>> {
    let mut matches = cert
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(index, decl)| match decl_payload_name(cert, &decl.decl) {
            Ok(name) if name == session.root.theorem_name => Some(Ok(index)),
            Ok(_) => None,
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if matches.len() != 1 {
        return Err(plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::CertificateGeneration,
            "generated certificate does not contain exactly one root theorem declaration",
        ));
    }
    let index = matches.pop().expect("len checked above");
    if !matches!(
        cert.declarations[index].decl,
        DeclPayload::Theorem {
            opacity: npa_cert::Opacity::Opaque,
            ..
        }
    ) {
        return Err(plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::CertificateGeneration,
            "generated root declaration is not an opaque theorem",
        ));
    }
    Ok(index)
}

fn root_axiom_refs(
    report: &npa_cert::AxiomReport,
    root_decl_index: usize,
) -> Result<&[AxiomRef], Box<MachineVerifyError>> {
    let matches = report
        .per_declaration
        .iter()
        .filter(|entry| entry.decl_index == root_decl_index)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::CertificateVerify,
            "verifier output does not contain exactly one root theorem axiom report",
        ));
    }
    Ok(&matches[0].transitive_axioms)
}

fn axiom_refs_to_wire(
    context: &GeneratedCertificateContext<'_>,
    axioms: &[AxiomRef],
) -> Result<Vec<MachineAxiomRefWire>, Box<MachineVerifyError>> {
    let mut out = Vec::with_capacity(axioms.len());
    for axiom in axioms {
        out.push(axiom_ref_to_wire(context, axiom)?);
    }
    sort_dedup_axiom_refs(&mut out);
    Ok(out)
}

fn axiom_ref_to_wire(
    context: &GeneratedCertificateContext<'_>,
    axiom: &AxiomRef,
) -> Result<MachineAxiomRefWire, Box<MachineVerifyError>> {
    let name = cert_name(context.cert, axiom.name)?;
    match &axiom.global_ref {
        GlobalRef::Imported {
            import_index,
            decl_interface_hash,
            ..
        } => {
            let import = context.cert.imports.get(*import_index).ok_or_else(|| {
                plain_error(
                    MachineApiErrorKind::VerifyFailed,
                    MachineApiDiagnosticPhase::CertificateVerify,
                    "verifier output imported axiom ref has out-of-range import_index",
                )
            })?;
            Ok(MachineAxiomRefWire::Imported {
                module: import.module.clone(),
                name,
                export_hash: import.export_hash,
                decl_interface_hash: *decl_interface_hash,
            })
        }
        GlobalRef::Local { decl_index } => {
            let decl = context.cert.declarations.get(*decl_index).ok_or_else(|| {
                plain_error(
                    MachineApiErrorKind::VerifyFailed,
                    MachineApiDiagnosticPhase::CertificateVerify,
                    "verifier output local axiom ref has out-of-range decl_index",
                )
            })?;
            if !matches!(decl.decl, DeclPayload::Axiom { .. }) {
                return Err(plain_error(
                    MachineApiErrorKind::VerifyFailed,
                    MachineApiDiagnosticPhase::CertificateVerify,
                    "verifier output local axiom ref does not point at an axiom declaration",
                ));
            }
            let source_index = context
                .cert_decl_to_source_index
                .get(decl_index)
                .copied()
                .ok_or_else(|| {
                    plain_error(
                        MachineApiErrorKind::VerifyFailed,
                        MachineApiDiagnosticPhase::CertificateVerify,
                        "verifier output local axiom ref has no Phase 5 source_index",
                    )
                })?;
            Ok(MachineAxiomRefWire::CurrentModule {
                module: context.cert.header.module.clone(),
                name,
                source_index,
                decl_interface_hash: axiom.decl_interface_hash,
            })
        }
        GlobalRef::Builtin {
            decl_interface_hash,
            ..
        } => Ok(MachineAxiomRefWire::Builtin {
            name,
            decl_interface_hash: *decl_interface_hash,
        }),
        GlobalRef::LocalGenerated { .. } => Err(plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::CertificateVerify,
            "verifier output axiom ref points at a generated declaration",
        )),
    }
}

fn ensure_axioms_allowed(
    allow_axioms: &[MachineAxiomRefWire],
    module_axioms_used: &[MachineAxiomRefWire],
) -> Result<(), Box<MachineVerifyError>> {
    let allowed = allow_axioms
        .iter()
        .map(encode_machine_axiom_ref_wire)
        .collect::<BTreeSet<_>>();
    for axiom in module_axioms_used {
        if !allowed.contains(&encode_machine_axiom_ref_wire(axiom)) {
            return Err(disallowed_axiom_error(axiom.clone()));
        }
    }
    Ok(())
}

fn dependency_import_closure_payloads(
    session: &MachineProofSession,
) -> Vec<MachineVerifiedModuleCertificatePayload> {
    let mut entries = session
        .import_certificate_context
        .verified_modules()
        .iter()
        .collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| lhs.key.cmp(&rhs.key));
    entries
        .into_iter()
        .map(import_entry_payload)
        .collect::<Vec<_>>()
}

fn import_entry_payload(
    entry: &VerifiedModuleContextEntry,
) -> MachineVerifiedModuleCertificatePayload {
    MachineVerifiedModuleCertificatePayload {
        module: entry.key.module.clone(),
        expected_export_hash: entry.key.export_hash,
        expected_certificate_hash: entry.key.certificate_hash,
        certificate: certificate_payload(&entry.certificate_bytes),
    }
}

fn certificate_payload(bytes: &[u8]) -> MachineCertificateWirePayload {
    MachineCertificateWirePayload {
        encoding: CERTIFICATE_ENCODING,
        bytes: hex_bytes(bytes),
    }
}

fn decl_payload_name(
    cert: &ModuleCert,
    payload: &DeclPayload,
) -> Result<Name, Box<MachineVerifyError>> {
    let name = match payload {
        DeclPayload::Axiom { name, .. }
        | DeclPayload::Def { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::Inductive { name, .. } => *name,
    };
    cert_name(cert, name)
}

fn cert_name(cert: &ModuleCert, name: usize) -> Result<Name, Box<MachineVerifyError>> {
    cert.name_table.get(name).cloned().ok_or_else(|| {
        plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::CertificateGeneration,
            "generated certificate references an out-of-range name table entry",
        )
    })
}

fn sort_dedup_axiom_refs(entries: &mut Vec<MachineAxiomRefWire>) {
    entries.sort_by_key(encode_machine_axiom_ref_wire);
    entries.dedup_by(|lhs, rhs| {
        encode_machine_axiom_ref_wire(lhs) == encode_machine_axiom_ref_wire(rhs)
    });
}

fn request_error(error: MachineApiRequestError) -> Box<MachineVerifyError> {
    plain_error(
        error.kind,
        MachineApiDiagnosticPhase::RequestValidation,
        format!(
            "request validation failed at {}: {:?}",
            json_path_display(&error.path),
            error.reason
        ),
    )
}

fn snapshot_lookup_error(error: MachineSnapshotLookupError) -> Box<MachineVerifyError> {
    match error {
        MachineSnapshotLookupError::UnknownSnapshot { .. } => plain_error(
            MachineApiErrorKind::UnknownSnapshot,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "unknown snapshot",
        ),
        MachineSnapshotLookupError::StateFingerprintMismatch { .. } => plain_error(
            MachineApiErrorKind::StateFingerprintMismatch,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "snapshot state_fingerprint does not match request",
        ),
        MachineSnapshotLookupError::InvalidMachineProofState { .. }
        | MachineSnapshotLookupError::ExecutableStateFingerprintMismatch { .. }
        | MachineSnapshotLookupError::SnapshotIdentityMismatch { .. }
        | MachineSnapshotLookupError::StoredSnapshotViewMismatch { .. } => plain_error(
            MachineApiErrorKind::InvalidMachineProofState,
            MachineApiDiagnosticPhase::SnapshotLookup,
            format!("stored snapshot self-check failed: {error:?}"),
        ),
    }
}

fn extraction_error(diagnostic: MachineTacticDiagnostic) -> Box<MachineVerifyError> {
    match diagnostic.kind {
        MachineTacticDiagnosticKind::UnresolvedGoal => plain_error(
            MachineApiErrorKind::InvalidVerifyRequest,
            MachineApiDiagnosticPhase::SnapshotLookup,
            diagnostic.message.to_string(),
        ),
        MachineTacticDiagnosticKind::InvalidMachineProofState => plain_error(
            MachineApiErrorKind::InvalidMachineProofState,
            MachineApiDiagnosticPhase::SnapshotLookup,
            diagnostic.message.to_string(),
        ),
        _ => plain_error(
            MachineApiErrorKind::VerifyFailed,
            MachineApiDiagnosticPhase::KernelCheck,
            diagnostic.message.to_string(),
        ),
    }
}

fn certificate_generation_error(error: CertError) -> Box<MachineVerifyError> {
    let phase = match error {
        CertError::Kernel(_)
        | CertError::UnresolvedMetavariable
        | CertError::InvalidBVar { .. } => MachineApiDiagnosticPhase::KernelCheck,
        _ => MachineApiDiagnosticPhase::CertificateGeneration,
    };
    plain_error(
        MachineApiErrorKind::VerifyFailed,
        phase,
        format!("certificate generation failed: {error:?}"),
    )
}

fn certificate_verify_error(error: CertError) -> Box<MachineVerifyError> {
    plain_error(
        MachineApiErrorKind::VerifyFailed,
        MachineApiDiagnosticPhase::CertificateVerify,
        format!("certificate verifier rejected generated certificate: {error:?}"),
    )
}

fn disallowed_axiom_error(axiom: MachineAxiomRefWire) -> Box<MachineVerifyError> {
    let name = axiom_ref_name(&axiom).clone();
    let message = format!("axiom {} is not allowed in this session", name.as_dotted());
    let diagnostic = MachineApiDiagnosticProjection {
        kind: MachineApiErrorKind::DisallowedAxiom,
        phase: MachineApiDiagnosticPhase::CertificateVerify,
        retryable: false,
        goal_id: None,
        tactic_kind: None,
        primary_name: Some(name),
        primary_axiom_ref: Some(axiom),
        expected_hash: None,
        actual_hash: None,
        source_message: message.clone(),
        upstream: Phase5UpstreamDiagnostic::Phase4(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofState,
            message,
        )),
    };
    boxed_error(diagnostic)
}

fn plain_error(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
) -> Box<MachineVerifyError> {
    let message = message.into();
    let diagnostic = MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id: None,
        tactic_kind: None,
        primary_name: None,
        primary_axiom_ref: None,
        expected_hash: None,
        actual_hash: None,
        source_message: message.clone(),
        upstream: Phase5UpstreamDiagnostic::Phase4(MachineTacticDiagnostic::new(
            phase4_kind_for_api_kind(kind),
            message,
        )),
    };
    boxed_error(diagnostic)
}

fn boxed_error(diagnostic: MachineApiDiagnosticProjection) -> Box<MachineVerifyError> {
    let wire = MachineApiErrorWire::from_projection(&diagnostic)
        .expect("verify diagnostics must satisfy Phase 5 wire invariants");
    let response = MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
        status: MachineApiResponseStatus::Error,
        error: wire,
        endpoint_fields: (),
    }));
    Box::new(MachineVerifyError {
        diagnostic,
        response,
    })
}

fn phase4_kind_for_api_kind(kind: MachineApiErrorKind) -> MachineTacticDiagnosticKind {
    match kind {
        MachineApiErrorKind::VerifyFailed => MachineTacticDiagnosticKind::KernelRejected,
        MachineApiErrorKind::InvalidVerifyRequest
        | MachineApiErrorKind::UnknownSession
        | MachineApiErrorKind::UnknownSnapshot
        | MachineApiErrorKind::StateFingerprintMismatch
        | MachineApiErrorKind::DisallowedAxiom
        | MachineApiErrorKind::InvalidMachineProofState => {
            MachineTacticDiagnosticKind::InvalidMachineProofState
        }
        _ => MachineTacticDiagnosticKind::InvalidMachineProofState,
    }
}

fn axiom_ref_name(axiom: &MachineAxiomRefWire) -> &Name {
    match axiom {
        MachineAxiomRefWire::Imported { name, .. }
        | MachineAxiomRefWire::CurrentModule { name, .. }
        | MachineAxiomRefWire::Builtin { name, .. } => name,
    }
}

fn json_path_display(path: &crate::JsonPath) -> String {
    if path.elements.is_empty() {
        return "$".to_owned();
    }
    let mut out = "$".to_owned();
    for element in &path.elements {
        match element {
            crate::JsonPathElement::Field(field) => {
                out.push('.');
                out.push_str(field);
            }
            crate::JsonPathElement::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    out
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("hex nybble is in range"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npa_cert::{
        build_module_cert, encode_module_cert, verify_module_cert, AxiomPolicy, CoreModule,
        VerifierSession,
    };
    use npa_kernel::{Decl, Expr, Level};

    use crate::{
        create_machine_session, format_goal_id_wire, format_hash_string,
        run_machine_tactic_request, MachineTacticRunSuccessFields,
    };

    fn default_options_json(allow_axioms: &str) -> String {
        format!(
            r#"{{
              "kernel_check_profile":"npa.kernel.v0.1.builtin-nat-eq-rec",
              "allow_axioms": {allow_axioms},
              "tactic_options": {{
                "simp_rules": [],
                "eq_family": null,
                "nat_family": null,
                "max_simp_rewrite_steps": 100,
                "max_open_goals": 32,
                "max_metas": 64
              }}
            }}"#
        )
    }

    fn minimal_session_json(theorem_type: &str) -> String {
        format!(
            r#"{{
              "protocol_version":"npa.machine-api.v1",
              "root":{{
                "module":"Scratch",
                "theorem_name":"Scratch.t",
                "source_index":0,
                "universe_params":[],
                "theorem_type":{{"format":"machine_surface_v1","source":"{theorem_type}"}}
              }},
              "import_closure":[],
              "imports":[],
              "checked_current_decls":[],
              "options":{}
            }}"#,
            default_options_json("[]")
        )
    }

    fn budget_json() -> &'static str {
        r#"{
          "max_tactic_steps":64,
          "max_whnf_steps":10000,
          "max_conversion_steps":10000,
          "max_rewrite_steps":100,
          "max_meta_allocations":8,
          "max_expr_nodes":20000
        }"#
    }

    fn run_json(
        session: &MachineProofSession,
        snapshot_id: SnapshotId,
        state_fingerprint: Hash,
        goal_id: npa_tactic::GoalId,
        candidate: &str,
    ) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"{}",
              "candidate":{},
              "deterministic_budget":{}
            }}"#,
            session.session_id.wire(),
            snapshot_id.wire(),
            format_hash_string(&state_fingerprint),
            format_goal_id_wire(goal_id),
            candidate,
            budget_json()
        )
    }

    fn verify_json(
        session: &MachineProofSession,
        snapshot_id: SnapshotId,
        state_fingerprint: Hash,
    ) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "mode":"certificate"
            }}"#,
            session.session_id.wire(),
            snapshot_id.wire(),
            format_hash_string(&state_fingerprint)
        )
    }

    fn unwrap_run_ok(response: crate::MachineTacticRunResponse) -> MachineTacticRunSuccessFields {
        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected tactic run success");
        };
        ok.endpoint_fields
    }

    #[test]
    fn verify_closed_snapshot_returns_certificate_import_payload() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let response = run_machine_tactic_request(
            &run_json(
                &session,
                session.initial_snapshot.snapshot_id,
                session.initial_snapshot.state_fingerprint,
                npa_tactic::GoalId(0),
                r#"{"kind":"exact","term":{"source":"Prop"}}"#,
            ),
            &mut session,
        )
        .unwrap();
        let run = unwrap_run_ok(response).result;

        let response = run_machine_verify_request(
            &verify_json(&session, run.next_snapshot_id, run.next_state_fingerprint),
            &session,
        )
        .unwrap();

        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected verify success");
        };
        assert_eq!(ok.status, MachineApiResponseStatus::Verified);
        assert_eq!(ok.endpoint_fields.root_axioms_used, Vec::new());
        assert_eq!(ok.endpoint_fields.module_axioms_used, Vec::new());
        assert_eq!(
            ok.endpoint_fields.import_payload.module,
            Name::from_dotted("Scratch")
        );
        assert_eq!(
            ok.endpoint_fields.import_payload.expected_export_hash,
            ok.endpoint_fields.module_export_hash
        );
        assert_eq!(
            ok.endpoint_fields.import_payload.expected_certificate_hash,
            ok.endpoint_fields.module_certificate_hash
        );
        assert_eq!(
            ok.endpoint_fields.certificate,
            ok.endpoint_fields.import_payload.certificate
        );
        assert_eq!(
            ok.endpoint_fields.certificate.encoding,
            "npa.certificate.canonical.v0.1.hex"
        );
        assert!(!ok.endpoint_fields.certificate.bytes.is_empty());
        assert!(ok.endpoint_fields.dependency_import_closure.is_empty());
    }

    #[test]
    fn verify_rejects_open_snapshot_as_invalid_verify_request() {
        let session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;

        let err = run_machine_verify_request(
            &verify_json(
                &session,
                session.initial_snapshot.snapshot_id,
                session.initial_snapshot.state_fingerprint,
            ),
            &session,
        )
        .unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::InvalidVerifyRequest
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
    }

    #[test]
    fn verify_checks_axiom_subset_against_final_certificate() {
        let (session_json, axiom_ref_json) = imported_axiom_session_json();
        let mut session = create_machine_session(&session_json(&axiom_ref_json))
            .unwrap()
            .session;
        let response = run_machine_tactic_request(
            &run_json(
                &session,
                session.initial_snapshot.snapshot_id,
                session.initial_snapshot.state_fingerprint,
                npa_tactic::GoalId(0),
                r#"{"kind":"exact","term":{"source":"A.T"}}"#,
            ),
            &mut session,
        )
        .unwrap();
        let run = unwrap_run_ok(response).result;

        let ok = run_machine_verify_request(
            &verify_json(&session, run.next_snapshot_id, run.next_state_fingerprint),
            &session,
        )
        .unwrap();
        let MachineApiResponseEnvelope::Ok(ok) = ok else {
            panic!("expected verify success with allowlist");
        };
        assert_eq!(ok.endpoint_fields.module_axioms_used.len(), 1);

        session.options.allow_axioms.clear();
        let err = run_machine_verify_request(
            &verify_json(&session, run.next_snapshot_id, run.next_state_fingerprint),
            &session,
        )
        .unwrap_err();
        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::DisallowedAxiom);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::CertificateVerify
        );
        assert_eq!(err.diagnostic.primary_name, Some(Name::from_dotted("A.T")));
    }

    #[test]
    fn verify_includes_transitive_axiom_origin_imports_in_generated_certificate() {
        let (session_json, axiom_ref_json) = transitive_axiom_session_json();
        let mut session = create_machine_session(&session_json(&axiom_ref_json))
            .unwrap()
            .session;
        let response = run_machine_tactic_request(
            &run_json(
                &session,
                session.initial_snapshot.snapshot_id,
                session.initial_snapshot.state_fingerprint,
                npa_tactic::GoalId(0),
                r#"{"kind":"exact","term":{"source":"A.t"}}"#,
            ),
            &mut session,
        )
        .unwrap();
        let run = unwrap_run_ok(response).result;

        let response = run_machine_verify_request(
            &verify_json(&session, run.next_snapshot_id, run.next_state_fingerprint),
            &session,
        )
        .unwrap();

        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected verify success with transitive axiom provenance");
        };
        let p_entry = session
            .import_certificate_context
            .verified_modules()
            .iter()
            .find(|entry| entry.key.module == Name::from_dotted("P"))
            .expect("P is in the verified import closure");
        assert_eq!(ok.endpoint_fields.module_axioms_used.len(), 1);
        assert_eq!(
            ok.endpoint_fields.module_axioms_used[0],
            MachineAxiomRefWire::Imported {
                module: Name::from_dotted("P"),
                name: Name::from_dotted("P.p"),
                export_hash: p_entry.key.export_hash,
                decl_interface_hash: p_entry.decl_index_table[0].hashes.decl_interface_hash,
            }
        );
        assert_eq!(ok.endpoint_fields.dependency_import_closure.len(), 2);
    }

    fn imported_axiom_session_json() -> (impl Fn(&str) -> String, String) {
        let module = CoreModule {
            name: Name::from_dotted("A"),
            declarations: vec![Decl::Axiom {
                name: "A.T".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::zero()),
            }],
        };
        let cert = build_module_cert(module, &[]).unwrap();
        let bytes = encode_module_cert(&cert).unwrap();
        let mut verifier_session = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy.allowlisted_axioms.insert(Name::from_dotted("A.T"));
        let verified = verify_module_cert(&bytes, &mut verifier_session, &policy).unwrap();
        let export_hash = format_hash_string(&verified.export_hash());
        let certificate_hash = format_hash_string(&verified.certificate_hash());
        let cert_hex = hex_bytes(&bytes);
        let decl_interface_hash =
            format_hash_string(&verified.declarations()[0].hashes.decl_interface_hash);
        let allow = format!(
            r#"[{{
              "kind":"imported",
              "module":"A",
              "name":"A.T",
              "export_hash":"{export_hash}",
              "decl_interface_hash":"{decl_interface_hash}"
            }}]"#
        );

        let session_json = move |allow_axioms: &str| {
            format!(
                r#"{{
                  "protocol_version":"npa.machine-api.v1",
                  "root":{{
                    "module":"Scratch",
                    "theorem_name":"Scratch.t",
                    "source_index":0,
                    "universe_params":[],
                    "theorem_type":{{"format":"machine_surface_v1","source":"Prop"}}
                  }},
                  "import_closure":[{{
                    "module":"A",
                    "expected_export_hash":"{export_hash}",
                    "expected_certificate_hash":"{certificate_hash}",
                    "certificate":{{
                      "encoding":"npa.certificate.canonical.v0.1.hex",
                      "bytes":"{cert_hex}"
                    }}
                  }}],
                  "imports":[{{
                    "module":"A",
                    "expected_export_hash":"{export_hash}",
                    "expected_certificate_hash":"{certificate_hash}"
                  }}],
                  "checked_current_decls":[],
                  "options":{}
                }}"#,
                default_options_json(allow_axioms)
            )
        };
        (session_json, allow)
    }

    fn transitive_axiom_session_json() -> (impl Fn(&str) -> String, String) {
        let p_module = CoreModule {
            name: Name::from_dotted("P"),
            declarations: vec![Decl::Axiom {
                name: "P.p".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::zero()),
            }],
        };
        let p_cert = build_module_cert(p_module, &[]).unwrap();
        let p_bytes = encode_module_cert(&p_cert).unwrap();
        let mut verifier_session = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy.allowlisted_axioms.insert(Name::from_dotted("P.p"));
        let verified_p = verify_module_cert(&p_bytes, &mut verifier_session, &policy).unwrap();

        let a_module = CoreModule {
            name: Name::from_dotted("A"),
            declarations: vec![Decl::Theorem {
                name: "A.t".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::zero()),
                proof: Expr::konst("P.p", Vec::new()),
            }],
        };
        let a_cert = build_module_cert(a_module, std::slice::from_ref(&verified_p)).unwrap();
        let a_bytes = encode_module_cert(&a_cert).unwrap();
        let verified_a = verify_module_cert(&a_bytes, &mut verifier_session, &policy).unwrap();

        let p_export_hash = format_hash_string(&verified_p.export_hash());
        let p_certificate_hash = format_hash_string(&verified_p.certificate_hash());
        let p_cert_hex = hex_bytes(&p_bytes);
        let p_decl_interface_hash =
            format_hash_string(&verified_p.declarations()[0].hashes.decl_interface_hash);
        let a_export_hash = format_hash_string(&verified_a.export_hash());
        let a_certificate_hash = format_hash_string(&verified_a.certificate_hash());
        let a_cert_hex = hex_bytes(&a_bytes);
        let allow = format!(
            r#"[{{
              "kind":"imported",
              "module":"P",
              "name":"P.p",
              "export_hash":"{p_export_hash}",
              "decl_interface_hash":"{p_decl_interface_hash}"
            }}]"#
        );

        let session_json = move |allow_axioms: &str| {
            format!(
                r#"{{
                  "protocol_version":"npa.machine-api.v1",
                  "root":{{
                    "module":"Scratch",
                    "theorem_name":"Scratch.t",
                    "source_index":0,
                    "universe_params":[],
                    "theorem_type":{{"format":"machine_surface_v1","source":"Prop"}}
                  }},
                  "import_closure":[{{
                    "module":"P",
                    "expected_export_hash":"{p_export_hash}",
                    "expected_certificate_hash":"{p_certificate_hash}",
                    "certificate":{{
                      "encoding":"npa.certificate.canonical.v0.1.hex",
                      "bytes":"{p_cert_hex}"
                    }}
                  }},{{
                    "module":"A",
                    "expected_export_hash":"{a_export_hash}",
                    "expected_certificate_hash":"{a_certificate_hash}",
                    "certificate":{{
                      "encoding":"npa.certificate.canonical.v0.1.hex",
                      "bytes":"{a_cert_hex}"
                    }}
                  }}],
                  "imports":[{{
                    "module":"A",
                    "expected_export_hash":"{a_export_hash}",
                    "expected_certificate_hash":"{a_certificate_hash}"
                  }}],
                  "checked_current_decls":[],
                  "options":{}
                }}"#,
                default_options_json(allow_axioms)
            )
        };
        (session_json, allow)
    }
}
