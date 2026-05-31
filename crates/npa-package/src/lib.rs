//! Package manifest metadata parsing and validation for external NPA libraries.
//!
//! This crate is an untrusted package tooling layer. It may reject malformed
//! package metadata before build or verification commands run, but theorem
//! acceptance remains with canonical certificates, the Rust kernel verdict, and
//! source-free checker verdicts.

#![deny(missing_docs)]

pub mod error;
pub mod graph;
pub mod hash;
pub mod manifest;
pub mod name;
pub mod path;
pub mod schema;
pub mod validate;
