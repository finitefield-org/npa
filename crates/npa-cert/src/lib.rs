//! Canonical certificate construction, hashing, encoding, and verification.
//!
//! This crate treats parser, elaborator, tactics, and automation output as untrusted. Its public
//! API accepts already elaborated kernel declarations, emits deterministic canonical certificates,
//! and verifies only canonical certificate bytes against the small Rust kernel.

#![deny(missing_docs)]

mod binary;
mod canonical;
mod hash;
mod inductive;
mod kernel;
mod producer;
mod types;
mod verify;

pub use inductive::{
    classify_inductive_artifact_profile_v1, generate_inductive_artifacts_v1,
    InductiveArtifactProfileCheckV1, UnsupportedMvpRecursorProfileV1,
};
pub use kernel::{builtin_decl_interface_hash, verified_module_to_kernel_decls};
pub use producer::*;
pub use types::*;

pub(crate) use binary::*;
pub(crate) use canonical::*;
pub(crate) use hash::*;
pub(crate) use kernel::{
    add_decl_to_env, add_referenced_builtins_to_env, builtin_is_axiom, cert_to_kernel_decls,
    expr_from_term, level_from_node, name_to_string, universe_names,
    verified_module_referenced_builtin_names,
};
pub(crate) use verify::*;

pub(crate) const FORMAT: &str = "NPA-CERT-0.1";
pub(crate) const CORE_SPEC: &str = "NPA-Core-0.1";

/// Build a canonical module certificate from already elaborated core declarations.
///
/// `imports` must be `VerifiedModule` values returned by this crate's verifier. The resulting
/// certificate contains only trusted canonical payload: source maps, diagnostics, tactic traces,
/// and AI traces are not encoded or hashed.
pub fn build_module_cert(module: CoreModule, imports: &[VerifiedModule]) -> Result<ModuleCert> {
    canonical::build_module_cert_impl(module, imports)
}

/// Encode a module certificate as the canonical `.npcert` binary representation.
///
/// The returned bytes are the exact bytes used by certificate verification and module hashing.
pub fn encode_module_cert(cert: &ModuleCert) -> Result<Vec<u8>> {
    Ok(binary::encode_module_cert_full(cert))
}

/// Decode a `.npcert` byte sequence into a syntactic certificate value.
///
/// This function does not trust or register the result. Use `verify_module_cert` to check
/// canonical encoding, hashes, imports, axiom policy, and kernel validity.
pub fn decode_module_cert(bytes: &[u8]) -> Result<ModuleCert> {
    let mut decoder = binary::Decoder::new(bytes);
    let cert = decoder.module_cert()?;
    if !decoder.is_done() {
        return Err(CertError::DecodeError);
    }
    Ok(cert)
}

/// Verify a canonical module certificate and register the verified module in `session`.
///
/// Verification performs decode, canonical byte round-trip, hash recomputation, import resolution,
/// high-trust policy checks, axiom report recomputation, and Rust kernel checking over decoded
/// core declarations.
pub fn verify_module_cert(
    bytes: &[u8],
    session: &mut VerifierSession,
    policy: &AxiomPolicy,
) -> Result<VerifiedModule> {
    verify::verify_module_cert_impl(bytes, session, policy)
}

/// Return the canonical structural hash for a term table entry in a module certificate.
///
/// The hash is computed from the term structure and referenced level hashes, not from the table
/// index itself.
pub fn term_hash(cert: &ModuleCert, term: TermId) -> Result<Hash> {
    hash::term_hash_impl(cert, term)
}

/// Return canonical bytes for a raw kernel expression.
///
/// This is the kernel core expression view used by higher-level machine APIs before a term is
/// embedded in a certificate module and resolved to certificate `GlobalRef`s.
pub fn core_expr_canonical_bytes(expr: &npa_kernel::Expr) -> Vec<u8> {
    hash::core_expr_canonical_bytes_impl(expr)
}

/// Return the canonical structural hash for a raw kernel expression.
///
/// This hash is computed from [`core_expr_canonical_bytes`] and ignores display-only binder names.
pub fn core_expr_hash(expr: &npa_kernel::Expr) -> Hash {
    hash::core_expr_hash_impl(expr)
}

/// Return canonical bytes for a declaration universe context.
///
/// The input must use sorted, unique universe parameters and normalized constraint levels. The
/// bytes are independent of certificate table indexes and reject unresolved/meta-like universe
/// encodings because the kernel level grammar has no meta constructor.
pub fn universe_constraints_canonical_bytes(
    universe_params: &[String],
    constraints: &[npa_kernel::UniverseConstraint],
) -> Result<Vec<u8>> {
    hash::universe_constraints_canonical_bytes_impl(universe_params, constraints)
}

/// Return the deterministic structural hash for a declaration universe context.
pub fn universe_constraints_hash(
    universe_params: &[String],
    constraints: &[npa_kernel::UniverseConstraint],
) -> Result<Hash> {
    hash::universe_constraints_hash_impl(universe_params, constraints)
}

#[cfg(test)]
mod tests;
