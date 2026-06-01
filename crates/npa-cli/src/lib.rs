#![deny(missing_docs)]

//! Contributor-facing NPA command-line parsing.
//!
//! The CLI crate is an untrusted orchestration layer. CLR-04 starts with
//! argument parsing only; later milestones add package loading and command
//! execution behind the parsed command model.

pub mod args;
pub mod diagnostic;
pub mod fs;
pub mod package;
pub mod package_artifacts;
pub mod package_build;
pub mod package_check;
pub mod package_hashes;
pub mod package_verify;
