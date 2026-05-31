//! Structured package manifest error types.

/// Result type for package manifest parsing and validation.
pub type PackageManifestResult<T> = Result<T, PackageManifestError>;

/// Stable package manifest error payload.
///
/// Tests should assert these structured fields instead of matching display text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageManifestError {
    /// Stable error category.
    pub kind: PackageManifestErrorKind,
    /// Stable manifest path, for example `$`, `policy.allowed_axioms`, or `modules[0].module`.
    pub path: String,
    /// Field name when the error is attached to one object field.
    pub field: Option<String>,
    /// Stable machine-readable reason code.
    pub reason_code: PackageManifestErrorReason,
    /// Expected value or type when useful.
    pub expected_value: Option<String>,
    /// Actual value or type when useful.
    pub actual_value: Option<String>,
}

impl PackageManifestError {
    /// Build a TOML syntax error.
    pub fn invalid_toml(message: impl Into<String>) -> Self {
        Self {
            kind: PackageManifestErrorKind::TomlSyntax,
            path: "$".to_owned(),
            field: None,
            reason_code: PackageManifestErrorReason::InvalidToml,
            expected_value: None,
            actual_value: Some(message.into()),
        }
    }

    /// Build a duplicate-field error reported by the TOML parser.
    pub fn duplicate_field(message: impl Into<String>) -> Self {
        Self {
            kind: PackageManifestErrorKind::Schema,
            path: "$".to_owned(),
            field: None,
            reason_code: PackageManifestErrorReason::DuplicateField,
            expected_value: None,
            actual_value: Some(message.into()),
        }
    }

    /// Build an unknown-field error.
    pub fn unknown_field(path: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            kind: PackageManifestErrorKind::Schema,
            path: path.into(),
            field: Some(field.into()),
            reason_code: PackageManifestErrorReason::UnknownField,
            expected_value: None,
            actual_value: None,
        }
    }

    /// Build a missing-field error.
    pub fn missing_field(path: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            kind: PackageManifestErrorKind::Schema,
            path: path.into(),
            field: Some(field.into()),
            reason_code: PackageManifestErrorReason::MissingField,
            expected_value: None,
            actual_value: None,
        }
    }

    /// Build a wrong-type error.
    pub fn wrong_type(
        path: impl Into<String>,
        field: Option<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            kind: PackageManifestErrorKind::Schema,
            path: path.into(),
            field,
            reason_code: PackageManifestErrorReason::WrongType,
            expected_value: Some(expected.into()),
            actual_value: Some(actual.into()),
        }
    }

    /// Build an invalid-hash-format error.
    pub fn invalid_hash_format(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Hash,
            path,
            None,
            PackageManifestErrorReason::InvalidHashFormat,
            Some("sha256:<64 lowercase hex>".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build an unsupported-schema error.
    pub fn unsupported_schema(
        path: impl Into<String>,
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            PackageManifestErrorKind::UnsupportedVersion,
            path,
            Some(field.into()),
            PackageManifestErrorReason::UnsupportedSchema,
            Some(expected.into()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-package-id error.
    pub fn invalid_package_id(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Domain,
            path,
            None,
            PackageManifestErrorReason::InvalidPackageId,
            Some("lowercase ASCII package id".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-version error.
    pub fn invalid_version(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Domain,
            path,
            None,
            PackageManifestErrorReason::InvalidVersion,
            Some("MAJOR.MINOR.PATCH without leading zeroes".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-profile error.
    pub fn invalid_profile(
        path: impl Into<String>,
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            PackageManifestErrorKind::Domain,
            path,
            Some(field.into()),
            PackageManifestErrorReason::InvalidProfile,
            Some(expected.into()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-module-name error.
    pub fn invalid_module_name(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Domain,
            path,
            None,
            PackageManifestErrorReason::InvalidModuleName,
            Some("canonical dotted name".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-declaration-name error.
    pub fn invalid_declaration_name(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Domain,
            path,
            None,
            PackageManifestErrorReason::InvalidDeclarationName,
            Some("canonical dotted name".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-axiom-name error.
    pub fn invalid_axiom_name(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Domain,
            path,
            None,
            PackageManifestErrorReason::InvalidAxiomName,
            Some("canonical dotted name".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build an invalid-path error.
    pub fn invalid_path(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageManifestErrorKind::Path,
            path,
            None,
            PackageManifestErrorReason::InvalidPath,
            Some("lexical package-relative path".to_owned()),
            Some(actual.into()),
        )
    }

    /// Build a duplicate-module error.
    pub fn duplicate_module(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::duplicate(
            path,
            "module",
            PackageManifestErrorReason::DuplicateModule,
            actual,
        )
    }

    /// Build a duplicate-external-import error.
    pub fn duplicate_external_import(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::duplicate(
            path,
            "module",
            PackageManifestErrorReason::DuplicateExternalImport,
            actual,
        )
    }

    /// Build a duplicate-declaration summary error.
    pub fn duplicate_declaration(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::duplicate(
            path,
            "declaration",
            PackageManifestErrorReason::DuplicateDeclaration,
            actual,
        )
    }

    /// Build a duplicate-axiom error.
    pub fn duplicate_axiom(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::duplicate(
            path,
            "axiom",
            PackageManifestErrorReason::DuplicateAxiom,
            actual,
        )
    }

    /// Build a duplicate-artifact-path error.
    pub fn duplicate_artifact_path(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::duplicate(
            path,
            "artifact_path",
            PackageManifestErrorReason::DuplicateArtifactPath,
            actual,
        )
    }

    /// Build a local/external module collision error.
    pub fn local_external_module_collision(
        path: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::duplicate(
            path,
            "module",
            PackageManifestErrorReason::LocalExternalModuleCollision,
            actual,
        )
    }

    fn duplicate(
        path: impl Into<String>,
        field: impl Into<String>,
        reason_code: PackageManifestErrorReason,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            PackageManifestErrorKind::Duplicate,
            path,
            Some(field.into()),
            reason_code,
            Some("unique value".to_owned()),
            Some(actual.into()),
        )
    }

    fn new(
        kind: PackageManifestErrorKind,
        path: impl Into<String>,
        field: Option<String>,
        reason_code: PackageManifestErrorReason,
        expected_value: Option<String>,
        actual_value: Option<String>,
    ) -> Self {
        Self {
            kind,
            path: path.into(),
            field,
            reason_code,
            expected_value,
            actual_value,
        }
    }
}

impl std::fmt::Display for PackageManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at {}: {}",
            self.kind,
            self.path,
            self.reason_code.as_str()
        )
    }
}

impl std::error::Error for PackageManifestError {}

/// Stable package manifest error category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageManifestErrorKind {
    /// TOML syntax or parser failure before schema validation.
    TomlSyntax,
    /// Closed-object schema, required field, or type validation failure.
    Schema,
    /// Unsupported schema version.
    UnsupportedVersion,
    /// Scalar domain validation failure.
    Domain,
    /// Duplicate package identity failure.
    Duplicate,
    /// Package-relative path validation failure.
    Path,
    /// Hash grammar validation failure.
    Hash,
    /// Import graph validation failure.
    Graph,
    /// Axiom policy validation failure.
    Policy,
}

/// Stable package manifest error reason code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageManifestErrorReason {
    /// TOML syntax is invalid.
    InvalidToml,
    /// A duplicate field was rejected.
    DuplicateField,
    /// A field is not part of the closed schema.
    UnknownField,
    /// A required field is absent.
    MissingField,
    /// A field has the wrong TOML type.
    WrongType,
    /// The schema field has the wrong value.
    WrongSchema,
    /// The schema version is unsupported.
    UnsupportedSchema,
    /// Package id grammar is invalid.
    InvalidPackageId,
    /// Package version grammar is invalid.
    InvalidVersion,
    /// Profile string is invalid.
    InvalidProfile,
    /// Module name grammar is invalid.
    InvalidModuleName,
    /// Declaration name grammar is invalid.
    InvalidDeclarationName,
    /// Axiom name grammar is invalid.
    InvalidAxiomName,
    /// Hash string grammar is invalid.
    InvalidHashFormat,
    /// Package path grammar is invalid.
    InvalidPath,
    /// Module name is duplicated.
    DuplicateModule,
    /// External import module is duplicated.
    DuplicateExternalImport,
    /// Declaration summary name is duplicated.
    DuplicateDeclaration,
    /// Axiom summary name is duplicated.
    DuplicateAxiom,
    /// Artifact path is duplicated.
    DuplicateArtifactPath,
    /// Local module and external import names collide.
    LocalExternalModuleCollision,
    /// Module import cannot be resolved.
    UnknownImport,
    /// Local module import graph has a cycle.
    ImportCycle,
    /// A module axiom is disallowed by package policy.
    DisallowedAxiom,
}

impl PackageManifestErrorReason {
    /// Return the stable wire reason code.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidToml => "invalid_toml",
            Self::DuplicateField => "duplicate_field",
            Self::UnknownField => "unknown_field",
            Self::MissingField => "missing_field",
            Self::WrongType => "wrong_type",
            Self::WrongSchema => "wrong_schema",
            Self::UnsupportedSchema => "unsupported_schema",
            Self::InvalidPackageId => "invalid_package_id",
            Self::InvalidVersion => "invalid_version",
            Self::InvalidProfile => "invalid_profile",
            Self::InvalidModuleName => "invalid_module_name",
            Self::InvalidDeclarationName => "invalid_declaration_name",
            Self::InvalidAxiomName => "invalid_axiom_name",
            Self::InvalidHashFormat => "invalid_hash_format",
            Self::InvalidPath => "invalid_path",
            Self::DuplicateModule => "duplicate_module",
            Self::DuplicateExternalImport => "duplicate_external_import",
            Self::DuplicateDeclaration => "duplicate_declaration",
            Self::DuplicateAxiom => "duplicate_axiom",
            Self::DuplicateArtifactPath => "duplicate_artifact_path",
            Self::LocalExternalModuleCollision => "local_external_module_collision",
            Self::UnknownImport => "unknown_import",
            Self::ImportCycle => "import_cycle",
            Self::DisallowedAxiom => "disallowed_axiom",
        }
    }
}
