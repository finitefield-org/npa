use npa_kernel::Decl;

use crate::{CertError, Hash, VerifiedModule};

/// Sidecar-only producer classification for audit and diagnostics.
///
/// This profile is intentionally not accepted by certificate construction or verification APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerProfile {
    /// Human-facing surface-language producer.
    HumanSurface,
    /// AI-facing MVP core declaration producer.
    AiCoreMvp,
}

/// Deterministic resource limits for producer candidate checking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProducerLimits {
    /// Maximum declarations accepted in a candidate batch.
    pub max_declarations: u32,
    /// Maximum expression nodes accepted in a candidate declaration.
    pub max_expr_nodes: u32,
    /// Maximum level nodes accepted in a candidate declaration.
    pub max_level_nodes: u32,
    /// Maximum dotted-name components accepted in a candidate declaration.
    pub max_name_components: u32,
    /// Maximum reduction steps available to candidate checking.
    pub max_reduction_steps: u64,
    /// Maximum conversion steps available to candidate checking.
    pub max_conversion_steps: u64,
}

/// Untrusted core declaration candidate submitted by a producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreDeclCandidate {
    /// Already elaborated kernel declaration proposed by the producer.
    pub declaration: Decl,
}

/// Batch of untrusted core declaration candidates checked against a current environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateBatch<'a> {
    /// Verified imports available to the candidate batch.
    pub imports: &'a [VerifiedModule],
    /// Previously checked current-module declarations available to later candidates.
    pub prior_current_decls: &'a [CheckedDeclCandidate],
    /// Candidate declarations to check, in input order.
    pub candidates: Vec<CoreDeclCandidate>,
    /// Deterministic resource limits applied to this batch.
    pub limits: ProducerLimits,
}

/// Opaque token for a candidate declaration accepted by producer checking.
///
/// The token has no public constructor. Later producer milestones fill in the private payload and
/// are the only path that may construct accepted tokens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedDeclCandidate {
    _private: (),
}

/// Non-authoritative hash preview computed while checking a producer candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateHashPreview {
    /// Preview of the declaration type hash, when available.
    pub type_hash: Option<Hash>,
    /// Preview of the declaration body or proof hash, when available.
    pub body_hash: Option<Hash>,
    /// Preview of the declaration interface hash, when available.
    pub decl_interface_hash: Option<Hash>,
    /// Preview of the declaration certificate hash, when available.
    pub decl_certificate_hash: Option<Hash>,
}

/// Per-candidate status returned by producer batch checking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateStatus {
    /// Candidate passed producer precheck and became an opaque token.
    Accepted(CheckedDeclCandidate),
    /// Candidate was rejected with a deterministic certificate error.
    Rejected(CertError),
}

/// Result for a producer candidate batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateBatchResult {
    /// One status per input candidate, in the same order.
    pub statuses: Vec<CandidateStatus>,
}
