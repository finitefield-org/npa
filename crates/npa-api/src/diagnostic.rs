use npa_cert::{Hash, Name};
use npa_tactic::goal_id_canonical_bytes;
use sha2::{Digest, Sha256};

use crate::current::{encode_machine_axiom_ref_wire, MachineAxiomRefWire};
use crate::{MachineApiDiagnosticProjection, MachineApiErrorKind};

const API_DIAGNOSTIC_TAG: &str = "npa.machine-api.api-diagnostic.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineApiDiagnosticCanonicalizationError {
    RetryableDiagnosticUnsupported,
    LengthExceeded { field: &'static str, len: usize },
    NonCanonicalName { field: &'static str },
    MissingPrimaryAxiomRef,
    UnexpectedPrimaryAxiomRef,
    DisallowedAxiomPrimaryNameMismatch,
    IncompleteTypeMismatchHashes,
    UnexpectedExpectedActualHash { kind: MachineApiErrorKind },
    MissingGoalId { kind: MachineApiErrorKind },
    UnexpectedGoalId { kind: MachineApiErrorKind },
    MissingTacticKind { kind: MachineApiErrorKind },
    UnexpectedTacticKind { kind: MachineApiErrorKind },
    UnexpectedPrimaryName { kind: MachineApiErrorKind },
}

impl MachineApiDiagnosticProjection {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, MachineApiDiagnosticCanonicalizationError> {
        machine_api_diagnostic_canonical_bytes(self)
    }

    pub fn diagnostic_hash(&self) -> Result<Hash, MachineApiDiagnosticCanonicalizationError> {
        machine_api_diagnostic_hash(self)
    }
}

pub fn machine_api_diagnostic_canonical_bytes(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<Vec<u8>, MachineApiDiagnosticCanonicalizationError> {
    validate_diagnostic(diagnostic)?;

    let mut out = Vec::new();
    encode_string(&mut out, "tag", API_DIAGNOSTIC_TAG)?;
    encode_string(&mut out, "kind", diagnostic.kind.as_str())?;
    encode_some_string(&mut out, "phase", diagnostic.phase.as_str())?;
    encode_option_goal_id(&mut out, diagnostic.goal_id);
    encode_option_tactic_kind(&mut out, diagnostic.tactic_kind)?;
    encode_option_name(&mut out, "primary_name", diagnostic.primary_name.as_ref())?;
    encode_option_axiom_ref(&mut out, diagnostic.primary_axiom_ref.as_ref());
    encode_option_hash(&mut out, diagnostic.expected_hash.as_ref());
    encode_option_hash(&mut out, diagnostic.actual_hash.as_ref());
    out.push(0x00);
    Ok(out)
}

pub fn machine_api_diagnostic_hash(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<Hash, MachineApiDiagnosticCanonicalizationError> {
    let canonical = machine_api_diagnostic_canonical_bytes(diagnostic)?;
    let digest = Sha256::digest(&canonical);
    let mut hash = [0; 32];
    hash.copy_from_slice(&digest);
    Ok(hash)
}

fn validate_diagnostic(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.retryable {
        return Err(MachineApiDiagnosticCanonicalizationError::RetryableDiagnosticUnsupported);
    }

    if let Some(name) = &diagnostic.primary_name {
        validate_name("primary_name", name)?;
    }

    match diagnostic.kind {
        MachineApiErrorKind::DisallowedAxiom => {
            let axiom_ref = diagnostic
                .primary_axiom_ref
                .as_ref()
                .ok_or(MachineApiDiagnosticCanonicalizationError::MissingPrimaryAxiomRef)?;
            validate_axiom_ref(axiom_ref)?;
            if diagnostic.primary_name.as_ref() != Some(axiom_ref_name(axiom_ref)) {
                return Err(
                    MachineApiDiagnosticCanonicalizationError::DisallowedAxiomPrimaryNameMismatch,
                );
            }
        }
        _ if diagnostic.primary_axiom_ref.is_some() => {
            return Err(MachineApiDiagnosticCanonicalizationError::UnexpectedPrimaryAxiomRef);
        }
        _ => {}
    }

    match (
        diagnostic.kind,
        diagnostic.expected_hash.is_some(),
        diagnostic.actual_hash.is_some(),
    ) {
        (MachineApiErrorKind::TypeMismatch, true, true) => Ok(()),
        (MachineApiErrorKind::TypeMismatch, _, _) => {
            Err(MachineApiDiagnosticCanonicalizationError::IncompleteTypeMismatchHashes)
        }
        (_, false, false) => Ok(()),
        (kind, _, _) => {
            Err(MachineApiDiagnosticCanonicalizationError::UnexpectedExpectedActualHash { kind })
        }
    }?;

    validate_primary_name_population(diagnostic)?;
    validate_goal_tactic_population(diagnostic)
}

fn validate_axiom_ref(
    axiom_ref: &MachineAxiomRefWire,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    match axiom_ref {
        MachineAxiomRefWire::Imported { module, name, .. } => {
            validate_name("primary_axiom_ref.module", module)?;
            validate_name("primary_axiom_ref.name", name)
        }
        MachineAxiomRefWire::CurrentModule { module, name, .. } => {
            validate_name("primary_axiom_ref.module", module)?;
            validate_name("primary_axiom_ref.name", name)
        }
        MachineAxiomRefWire::Builtin { name, .. } => validate_name("primary_axiom_ref.name", name),
    }
}

fn axiom_ref_name(axiom_ref: &MachineAxiomRefWire) -> &Name {
    match axiom_ref {
        MachineAxiomRefWire::Imported { name, .. }
        | MachineAxiomRefWire::CurrentModule { name, .. }
        | MachineAxiomRefWire::Builtin { name, .. } => name,
    }
}

fn validate_primary_name_population(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.primary_name.is_none() || diagnostic.kind == MachineApiErrorKind::DisallowedAxiom
    {
        return Ok(());
    }

    if primary_name_forbidden(diagnostic.kind) {
        return Err(
            MachineApiDiagnosticCanonicalizationError::UnexpectedPrimaryName {
                kind: diagnostic.kind,
            },
        );
    }

    Ok(())
}

fn primary_name_forbidden(kind: MachineApiErrorKind) -> bool {
    matches!(
        kind,
        MachineApiErrorKind::UnknownSession
            | MachineApiErrorKind::UnknownSnapshot
            | MachineApiErrorKind::StateFingerprintMismatch
            | MachineApiErrorKind::SessionRootHashMismatch
            | MachineApiErrorKind::InvalidSnapshotRequest
            | MachineApiErrorKind::InvalidTacticRunRequest
            | MachineApiErrorKind::InvalidTheoremQuery
            | MachineApiErrorKind::InvalidPromptPayloadRequest
            | MachineApiErrorKind::InvalidBatchPolicy
            | MachineApiErrorKind::InvalidSchedulerLimits
            | MachineApiErrorKind::InvalidReplayPlan
            | MachineApiErrorKind::InvalidVerifyRequest
            | MachineApiErrorKind::InvalidBudget
            | MachineApiErrorKind::GoalNotOpen
            | MachineApiErrorKind::ReplayHashMismatch
            | MachineApiErrorKind::MachineTermParseError
            | MachineApiErrorKind::TypeMismatch
            | MachineApiErrorKind::ExpectedPiType
            | MachineApiErrorKind::UnsupportedTactic
            | MachineApiErrorKind::RewriteRuleInvalid
            | MachineApiErrorKind::SimpNoProgress
            | MachineApiErrorKind::InductionTargetNotNat
            | MachineApiErrorKind::BudgetExceeded
            | MachineApiErrorKind::TooManyGoals
            | MachineApiErrorKind::TooLargeTerm
            | MachineApiErrorKind::VerifyFailed
    )
}

fn validate_goal_tactic_population(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.tactic_kind.is_some() && diagnostic.goal_id.is_none() {
        return Err(MachineApiDiagnosticCanonicalizationError::MissingGoalId {
            kind: diagnostic.kind,
        });
    }

    match diagnostic.kind {
        MachineApiErrorKind::DisallowedAxiom => ensure_no_goal_tactic(diagnostic),
        MachineApiErrorKind::GoalNotOpen => {
            ensure_goal_id(diagnostic)?;
            ensure_no_tactic_kind(diagnostic)
        }
        MachineApiErrorKind::ReplayHashMismatch => ensure_goal_id(diagnostic),
        MachineApiErrorKind::UnsupportedTactic
        | MachineApiErrorKind::RewriteRuleInvalid
        | MachineApiErrorKind::SimpNoProgress
        | MachineApiErrorKind::InductionTargetNotNat
        | MachineApiErrorKind::BudgetExceeded
        | MachineApiErrorKind::TooManyGoals
        | MachineApiErrorKind::TooLargeTerm => {
            ensure_goal_id(diagnostic)?;
            ensure_tactic_kind(diagnostic)
        }
        MachineApiErrorKind::MachineTermParseError
        | MachineApiErrorKind::MachineTermElaborationError
        | MachineApiErrorKind::UnknownName
        | MachineApiErrorKind::ImplicitArgumentRequired
        | MachineApiErrorKind::TypeMismatch
        | MachineApiErrorKind::ExpectedPiType => {
            if diagnostic.goal_id.is_some() {
                ensure_tactic_kind(diagnostic)
            } else {
                Ok(())
            }
        }
        MachineApiErrorKind::InvalidMachineProofState | MachineApiErrorKind::InvalidCandidate => {
            Ok(())
        }
        MachineApiErrorKind::UnknownSession
        | MachineApiErrorKind::UnknownSnapshot
        | MachineApiErrorKind::StateFingerprintMismatch
        | MachineApiErrorKind::SessionRootHashMismatch
        | MachineApiErrorKind::InvalidVerifiedImport
        | MachineApiErrorKind::InvalidCheckedCurrentDecl
        | MachineApiErrorKind::InvalidMachineApiOptions
        | MachineApiErrorKind::InvalidSessionRequest
        | MachineApiErrorKind::InvalidSnapshotRequest
        | MachineApiErrorKind::InvalidTacticRunRequest
        | MachineApiErrorKind::InvalidTheoremIndex
        | MachineApiErrorKind::InvalidTheoremQuery
        | MachineApiErrorKind::InvalidPromptPayloadRequest
        | MachineApiErrorKind::InvalidBatchPolicy
        | MachineApiErrorKind::InvalidSchedulerLimits
        | MachineApiErrorKind::InvalidReplayPlan
        | MachineApiErrorKind::InvalidVerifyRequest
        | MachineApiErrorKind::InvalidBudget
        | MachineApiErrorKind::VerifyFailed => ensure_no_goal_tactic(diagnostic),
    }
}

fn ensure_no_goal_tactic(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    ensure_no_goal_id(diagnostic)?;
    ensure_no_tactic_kind(diagnostic)
}

fn ensure_goal_id(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.goal_id.is_some() {
        Ok(())
    } else {
        Err(MachineApiDiagnosticCanonicalizationError::MissingGoalId {
            kind: diagnostic.kind,
        })
    }
}

fn ensure_no_goal_id(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.goal_id.is_none() {
        Ok(())
    } else {
        Err(
            MachineApiDiagnosticCanonicalizationError::UnexpectedGoalId {
                kind: diagnostic.kind,
            },
        )
    }
}

fn ensure_tactic_kind(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.tactic_kind.is_some() {
        Ok(())
    } else {
        Err(
            MachineApiDiagnosticCanonicalizationError::MissingTacticKind {
                kind: diagnostic.kind,
            },
        )
    }
}

fn ensure_no_tactic_kind(
    diagnostic: &MachineApiDiagnosticProjection,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if diagnostic.tactic_kind.is_none() {
        Ok(())
    } else {
        Err(
            MachineApiDiagnosticCanonicalizationError::UnexpectedTacticKind {
                kind: diagnostic.kind,
            },
        )
    }
}

fn validate_name(
    field: &'static str,
    name: &Name,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    if !name.is_canonical() {
        return Err(MachineApiDiagnosticCanonicalizationError::NonCanonicalName { field });
    }
    checked_u32_len(field, name.0.len())?;
    for component in &name.0 {
        checked_u32_len(field, component.len())?;
    }
    Ok(())
}

fn encode_some_string(
    out: &mut Vec<u8>,
    field: &'static str,
    value: &str,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    out.push(0x01);
    encode_string(out, field, value)
}

fn encode_option_goal_id(out: &mut Vec<u8>, value: Option<npa_tactic::GoalId>) {
    match value {
        Some(goal_id) => {
            out.push(0x01);
            out.extend(goal_id_canonical_bytes(goal_id));
        }
        None => out.push(0x00),
    }
}

fn encode_option_tactic_kind(
    out: &mut Vec<u8>,
    value: Option<crate::MachineApiTacticKind>,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    match value {
        Some(kind) => encode_some_string(out, "tactic_kind", kind.as_str()),
        None => {
            out.push(0x00);
            Ok(())
        }
    }
}

fn encode_option_name(
    out: &mut Vec<u8>,
    field: &'static str,
    value: Option<&Name>,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    match value {
        Some(name) => {
            out.push(0x01);
            encode_name(out, field, name)
        }
        None => {
            out.push(0x00);
            Ok(())
        }
    }
}

fn encode_option_axiom_ref(out: &mut Vec<u8>, value: Option<&MachineAxiomRefWire>) {
    match value {
        Some(axiom_ref) => {
            out.push(0x01);
            out.extend(encode_machine_axiom_ref_wire(axiom_ref));
        }
        None => out.push(0x00),
    }
}

fn encode_option_hash(out: &mut Vec<u8>, value: Option<&Hash>) {
    match value {
        Some(hash) => {
            out.push(0x01);
            out.extend(hash);
        }
        None => out.push(0x00),
    }
}

fn encode_name(
    out: &mut Vec<u8>,
    field: &'static str,
    name: &Name,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    validate_name(field, name)?;
    encode_u32(out, checked_u32_len(field, name.0.len())?);
    for component in &name.0 {
        encode_string(out, field, component)?;
    }
    Ok(())
}

fn encode_string(
    out: &mut Vec<u8>,
    field: &'static str,
    value: &str,
) -> Result<(), MachineApiDiagnosticCanonicalizationError> {
    encode_u32(out, checked_u32_len(field, value.len())?);
    out.extend(value.as_bytes());
    Ok(())
}

fn checked_u32_len(
    field: &'static str,
    len: usize,
) -> Result<u32, MachineApiDiagnosticCanonicalizationError> {
    u32::try_from(len)
        .map_err(|_| MachineApiDiagnosticCanonicalizationError::LengthExceeded { field, len })
}

fn encode_u32(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MachineApiDiagnosticPhase, MachineApiTacticKind, MachineApiUpstreamDiagnostic};
    use npa_tactic::{GoalId, MachineTacticDiagnostic, MachineTacticDiagnosticKind};

    fn projection(kind: MachineApiErrorKind) -> MachineApiDiagnosticProjection {
        MachineApiDiagnosticProjection {
            kind,
            phase: MachineApiDiagnosticPhase::MachineTermCheck,
            retryable: false,
            goal_id: Some(GoalId(7)),
            tactic_kind: Some(MachineApiTacticKind::Exact),
            primary_name: None,
            primary_axiom_ref: None,
            expected_hash: None,
            actual_hash: None,
            source_message: "display only".to_owned(),
            upstream: MachineApiUpstreamDiagnostic::MachineTactic(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::MachineTermElaborationError,
                "display only",
            )),
        }
    }

    fn manual_string(value: &str) -> Vec<u8> {
        let mut out = Vec::new();
        manual_u32(&mut out, value.len() as u32);
        out.extend(value.as_bytes());
        out
    }

    fn manual_name(components: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        manual_u32(&mut out, components.len() as u32);
        for component in components {
            out.extend(manual_string(component));
        }
        out
    }

    fn manual_u32(out: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    #[test]
    fn canonical_bytes_use_fixed_field_order_and_wire_names() {
        let mut diagnostic = projection(MachineApiErrorKind::UnknownName);
        diagnostic.tactic_kind = Some(MachineApiTacticKind::Rw);
        diagnostic.primary_name = Some(Name::from_dotted("Nat.zero"));

        let mut expected = Vec::new();
        expected.extend(manual_string(API_DIAGNOSTIC_TAG));
        expected.extend(manual_string("unknown_name"));
        expected.push(0x01);
        expected.extend(manual_string("machine_term_check"));
        expected.push(0x01);
        expected.extend(goal_id_canonical_bytes(GoalId(7)));
        expected.push(0x01);
        expected.extend(manual_string("rw"));
        expected.push(0x01);
        expected.extend(manual_name(&["Nat", "zero"]));
        expected.push(0x00);
        expected.push(0x00);
        expected.push(0x00);
        expected.push(0x00);

        assert_eq!(diagnostic.canonical_bytes().unwrap(), expected);
    }

    #[test]
    fn diagnostic_hash_uses_canonical_bytes_only() {
        let mut diagnostic = projection(MachineApiErrorKind::TypeMismatch);
        diagnostic.expected_hash = Some([1; 32]);
        diagnostic.actual_hash = Some([2; 32]);

        let hash = diagnostic.diagnostic_hash().unwrap();
        let canonical = diagnostic.canonical_bytes().unwrap();
        let manual = Sha256::digest(&canonical);
        assert_eq!(hash.as_slice(), manual.as_slice());

        let mut display_changed = diagnostic.clone();
        display_changed.source_message = "different display text".to_owned();
        display_changed.upstream =
            MachineApiUpstreamDiagnostic::MachineTactic(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::TypeMismatch,
                "different source diagnostic message",
            ));
        assert_eq!(display_changed.diagnostic_hash().unwrap(), hash);

        let mut structured_changed = diagnostic;
        structured_changed.actual_hash = Some([3; 32]);
        assert_ne!(structured_changed.diagnostic_hash().unwrap(), hash);
    }

    #[test]
    fn disallowed_axiom_requires_matching_primary_axiom_ref() {
        let axiom_ref = MachineAxiomRefWire::Builtin {
            name: Name::from_dotted("Classical.choice"),
            decl_interface_hash: [4; 32],
        };
        let mut diagnostic = projection(MachineApiErrorKind::DisallowedAxiom);
        diagnostic.phase = MachineApiDiagnosticPhase::CertificateVerify;
        diagnostic.goal_id = None;
        diagnostic.tactic_kind = None;
        diagnostic.primary_name = Some(Name::from_dotted("Classical.choice"));
        diagnostic.primary_axiom_ref = Some(axiom_ref.clone());

        let hash = diagnostic.diagnostic_hash().unwrap();
        let mut changed = diagnostic;
        changed.primary_axiom_ref = Some(MachineAxiomRefWire::Builtin {
            name: Name::from_dotted("Classical.choice"),
            decl_interface_hash: [5; 32],
        });
        assert_ne!(changed.diagnostic_hash().unwrap(), hash);

        let mut missing = changed;
        missing.primary_axiom_ref = None;
        assert_eq!(
            missing.diagnostic_hash().unwrap_err(),
            MachineApiDiagnosticCanonicalizationError::MissingPrimaryAxiomRef
        );
    }

    #[test]
    fn scheduler_retryable_stop_is_not_a_deterministic_diagnostic() {
        let mut diagnostic = projection(MachineApiErrorKind::BudgetExceeded);
        diagnostic.retryable = true;

        assert_eq!(
            diagnostic.diagnostic_hash().unwrap_err(),
            MachineApiDiagnosticCanonicalizationError::RetryableDiagnosticUnsupported
        );
    }

    #[test]
    fn type_mismatch_requires_both_hashes() {
        let mut diagnostic = projection(MachineApiErrorKind::TypeMismatch);
        diagnostic.expected_hash = Some([1; 32]);

        assert_eq!(
            diagnostic.diagnostic_hash().unwrap_err(),
            MachineApiDiagnosticCanonicalizationError::IncompleteTypeMismatchHashes
        );
    }

    #[test]
    fn rejects_primary_name_for_kinds_with_none_override() {
        for kind in [
            MachineApiErrorKind::GoalNotOpen,
            MachineApiErrorKind::ReplayHashMismatch,
            MachineApiErrorKind::BudgetExceeded,
        ] {
            let mut diagnostic = projection(kind);
            diagnostic.primary_name = Some(Name::from_dotted("Nat.zero"));

            assert_eq!(
                diagnostic.diagnostic_hash().unwrap_err(),
                MachineApiDiagnosticCanonicalizationError::UnexpectedPrimaryName { kind }
            );
        }
    }

    #[test]
    fn rejects_goal_tactic_population_mismatch() {
        let mut budget_exceeded = projection(MachineApiErrorKind::BudgetExceeded);
        budget_exceeded.goal_id = None;
        budget_exceeded.tactic_kind = None;
        assert_eq!(
            budget_exceeded.diagnostic_hash().unwrap_err(),
            MachineApiDiagnosticCanonicalizationError::MissingGoalId {
                kind: MachineApiErrorKind::BudgetExceeded
            }
        );

        let mut goal_not_open = projection(MachineApiErrorKind::GoalNotOpen);
        assert_eq!(
            goal_not_open.diagnostic_hash().unwrap_err(),
            MachineApiDiagnosticCanonicalizationError::UnexpectedTacticKind {
                kind: MachineApiErrorKind::GoalNotOpen
            }
        );

        goal_not_open.tactic_kind = None;
        assert!(goal_not_open.diagnostic_hash().is_ok());

        let mut request_error = projection(MachineApiErrorKind::InvalidBatchPolicy);
        request_error.tactic_kind = None;
        assert_eq!(
            request_error.diagnostic_hash().unwrap_err(),
            MachineApiDiagnosticCanonicalizationError::UnexpectedGoalId {
                kind: MachineApiErrorKind::InvalidBatchPolicy
            }
        );
    }
}
