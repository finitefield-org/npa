//! Package lock data model and canonical JSON parsing.
//!
//! A package lock is generated orchestration metadata. It records source-free
//! certificate identities for package graph verification, but it is not proof
//! evidence by itself.

use std::collections::{BTreeMap, BTreeSet};

use npa_cert::Name;

use crate::{
    error::{PackageLockError, PackageLockResult},
    hash::{format_package_hash, parse_package_hash, PackageHash},
    json::{parse_json, JsonMember, JsonValue},
    manifest::PackageVersion,
    name::{validate_package_id, PackageId},
    path::{validate_package_path, PackagePath},
    schema::PACKAGE_LOCK_SCHEMA,
    validate::validate_package_version,
};

/// Generated `npa.package.lock.v0.1` package lock artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageLockManifest {
    /// Lock schema string; must equal [`PACKAGE_LOCK_SCHEMA`].
    pub schema: String,
    /// Package identity copied from the validated package manifest.
    pub package: PackageId,
    /// Exact package version copied from the validated package manifest.
    pub version: PackageVersion,
    /// Exact manifest path and file hash used to produce the lock.
    pub manifest: PackageLockManifestReference,
    /// Source-free certificate entries sorted canonically when serialized.
    pub entries: Vec<PackageLockEntry>,
}

impl PackageLockManifest {
    /// Serialize the lock as deterministic canonical JSON.
    pub fn canonical_json(&self) -> PackageLockResult<String> {
        validate_package_lock_manifest(self)?;
        Ok(package_lock_json_unchecked(&normalized_package_lock(self)))
    }
}

/// Package manifest identity recorded inside a package lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageLockManifestReference {
    /// Package-relative path to the manifest bytes.
    pub path: PackagePath,
    /// Exact SHA-256 hash of the manifest file bytes.
    pub file_hash: PackageHash,
}

/// Package lock entry origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageLockEntryOrigin {
    /// Certificate belongs to the local package.
    Local,
    /// Certificate belongs to an external hash-pinned package import.
    External,
}

impl PackageLockEntryOrigin {
    /// Return the lock JSON origin string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::External => "external",
        }
    }

    fn parse(value: &str, path: &str) -> PackageLockResult<Self> {
        match value {
            "local" => Ok(Self::Local),
            "external" => Ok(Self::External),
            _ => Err(PackageLockError::invalid_origin(path, value)),
        }
    }
}

/// One source-free certificate identity in a package lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageLockEntry {
    /// Module provided by this certificate entry.
    pub module: Name,
    /// Whether the entry is local to the package or external.
    pub origin: PackageLockEntryOrigin,
    /// Package-relative path to the certificate bytes.
    pub certificate: PackagePath,
    /// Exact SHA-256 hash of the certificate file bytes.
    pub certificate_file_hash: PackageHash,
    /// Canonical export hash declared by the certificate.
    pub export_hash: PackageHash,
    /// Canonical axiom report hash declared by the certificate.
    pub axiom_report_hash: PackageHash,
    /// Canonical certificate hash declared by the certificate.
    pub certificate_hash: PackageHash,
    /// Direct certificate import identities.
    pub imports: Vec<PackageLockImport>,
    /// External package identity; present only when [`Self::origin`] is external.
    pub package: Option<PackageId>,
    /// External package version; present only when [`Self::origin`] is external.
    pub version: Option<PackageVersion>,
}

/// One direct certificate import identity recorded in a package lock entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageLockImport {
    /// Imported module name.
    pub module: Name,
    /// Imported module export hash.
    pub export_hash: PackageHash,
    /// Imported module certificate hash.
    pub certificate_hash: PackageHash,
}

/// Parse and validate a package lock from JSON.
pub fn parse_package_lock_json(source: &str) -> PackageLockResult<PackageLockManifest> {
    let root =
        parse_json(source).map_err(|error| PackageLockError::invalid_json(error.to_string()))?;
    let lock = parse_package_lock_value(&root)?;
    validate_package_lock_manifest(&lock)?;
    Ok(normalized_package_lock(&lock))
}

/// Validate a package lock data model without reading files or running checkers.
pub fn validate_package_lock_manifest(lock: &PackageLockManifest) -> PackageLockResult<()> {
    if lock.schema != PACKAGE_LOCK_SCHEMA {
        return Err(PackageLockError::unsupported_schema(
            "schema",
            "schema",
            PACKAGE_LOCK_SCHEMA,
            lock.schema.clone(),
        ));
    }
    validate_lock_package_id(&lock.package, "package")?;
    validate_lock_package_version(&lock.version, "version")?;
    validate_lock_path(&lock.manifest.path, "manifest.path")?;

    let mut modules = BTreeMap::<Name, usize>::new();
    let mut certificate_paths = BTreeMap::<String, usize>::new();
    for (entry_index, entry) in lock.entries.iter().enumerate() {
        let entry_path = format!("entries[{entry_index}]");
        validate_lock_module_name(&entry.module, format!("{entry_path}.module"))?;
        validate_lock_path(&entry.certificate, format!("{entry_path}.certificate"))?;
        if modules.insert(entry.module.clone(), entry_index).is_some() {
            return Err(PackageLockError::duplicate_lock_entry(
                format!("{entry_path}.module"),
                entry.module.as_dotted(),
            ));
        }
        if certificate_paths
            .insert(entry.certificate.as_str().to_owned(), entry_index)
            .is_some()
        {
            return Err(PackageLockError::duplicate_certificate_path(
                format!("{entry_path}.certificate"),
                entry.certificate.as_str(),
            ));
        }

        match entry.origin {
            PackageLockEntryOrigin::Local => {
                if let Some(package) = &entry.package {
                    return Err(PackageLockError::local_field_forbidden(
                        format!("{entry_path}.package"),
                        "package",
                        package.as_str(),
                    ));
                }
                if let Some(version) = &entry.version {
                    return Err(PackageLockError::local_field_forbidden(
                        format!("{entry_path}.version"),
                        "version",
                        version.as_str(),
                    ));
                }
            }
            PackageLockEntryOrigin::External => {
                let Some(package) = &entry.package else {
                    return Err(PackageLockError::external_field_required(
                        format!("{entry_path}.package"),
                        "package",
                    ));
                };
                let Some(version) = &entry.version else {
                    return Err(PackageLockError::external_field_required(
                        format!("{entry_path}.version"),
                        "version",
                    ));
                };
                validate_lock_package_id(package, format!("{entry_path}.package"))?;
                validate_lock_package_version(version, format!("{entry_path}.version"))?;
            }
        }

        validate_lock_imports(&entry.imports, &entry_path)?;
    }
    Ok(())
}

fn validate_lock_imports(imports: &[PackageLockImport], entry_path: &str) -> PackageLockResult<()> {
    let mut modules = BTreeSet::<Name>::new();
    for (import_index, import) in imports.iter().enumerate() {
        let import_path = format!("{entry_path}.imports[{import_index}]");
        validate_lock_module_name(&import.module, format!("{import_path}.module"))?;
        if !modules.insert(import.module.clone()) {
            return Err(PackageLockError::duplicate_import(
                format!("{import_path}.module"),
                import.module.as_dotted(),
            ));
        }
    }
    Ok(())
}

fn parse_package_lock_value(value: &JsonValue) -> PackageLockResult<PackageLockManifest> {
    let members = expect_object(value, "$")?;
    reject_unknown_fields("$", members, TOP_LEVEL_FIELDS)?;

    Ok(PackageLockManifest {
        schema: required_string(members, "$", "schema")?,
        package: PackageId::new(required_string(members, "$", "package")?),
        version: PackageVersion::new(required_string(members, "$", "version")?),
        manifest: parse_manifest_reference(required_value(members, "$", "manifest")?)?,
        entries: required_array(members, "$", "entries")?
            .iter()
            .enumerate()
            .map(|(index, value)| parse_entry(index, value))
            .collect::<PackageLockResult<Vec<_>>>()?,
    })
}

fn parse_manifest_reference(value: &JsonValue) -> PackageLockResult<PackageLockManifestReference> {
    let path = "manifest";
    let members = expect_object(value, path)?;
    reject_unknown_fields(path, members, MANIFEST_REFERENCE_FIELDS)?;
    Ok(PackageLockManifestReference {
        path: PackagePath::new(required_string(members, path, "path")?),
        file_hash: required_hash(members, path, "file_hash")?,
    })
}

fn parse_entry(index: usize, value: &JsonValue) -> PackageLockResult<PackageLockEntry> {
    let path = format!("entries[{index}]");
    let members = expect_object(value, &path)?;
    reject_unknown_fields(&path, members, ENTRY_FIELDS)?;
    let origin_path = field_path(&path, "origin");
    let origin =
        PackageLockEntryOrigin::parse(&required_string(members, &path, "origin")?, &origin_path)?;

    Ok(PackageLockEntry {
        module: Name::from_dotted(required_string(members, &path, "module")?),
        origin,
        certificate: PackagePath::new(required_string(members, &path, "certificate")?),
        certificate_file_hash: required_hash(members, &path, "certificate_file_hash")?,
        export_hash: required_hash(members, &path, "export_hash")?,
        axiom_report_hash: required_hash(members, &path, "axiom_report_hash")?,
        certificate_hash: required_hash(members, &path, "certificate_hash")?,
        imports: required_array(members, &path, "imports")?
            .iter()
            .enumerate()
            .map(|(import_index, value)| parse_import(&path, import_index, value))
            .collect::<PackageLockResult<Vec<_>>>()?,
        package: optional_string(members, &path, "package")?.map(PackageId::new),
        version: optional_string(members, &path, "version")?.map(PackageVersion::new),
    })
}

fn parse_import(
    entry_path: &str,
    import_index: usize,
    value: &JsonValue,
) -> PackageLockResult<PackageLockImport> {
    let path = format!("{entry_path}.imports[{import_index}]");
    let members = expect_object(value, &path)?;
    reject_unknown_fields(&path, members, IMPORT_FIELDS)?;
    Ok(PackageLockImport {
        module: Name::from_dotted(required_string(members, &path, "module")?),
        export_hash: required_hash(members, &path, "export_hash")?,
        certificate_hash: required_hash(members, &path, "certificate_hash")?,
    })
}

const TOP_LEVEL_FIELDS: &[&str] = &["schema", "package", "version", "manifest", "entries"];
const MANIFEST_REFERENCE_FIELDS: &[&str] = &["path", "file_hash"];
const ENTRY_FIELDS: &[&str] = &[
    "module",
    "origin",
    "package",
    "version",
    "certificate",
    "certificate_file_hash",
    "export_hash",
    "axiom_report_hash",
    "certificate_hash",
    "imports",
];
const IMPORT_FIELDS: &[&str] = &["module", "export_hash", "certificate_hash"];

fn expect_object<'a>(value: &'a JsonValue, path: &str) -> PackageLockResult<&'a [JsonMember]> {
    value
        .object_members()
        .ok_or_else(|| PackageLockError::wrong_type(path, None, "object", value.kind().as_str()))
}

fn reject_unknown_fields(
    path: &str,
    members: &[JsonMember],
    allowed: &[&str],
) -> PackageLockResult<()> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for member in members {
        *counts.entry(member.key()).or_insert(0) += 1;
    }

    if let Some((field, _)) = counts.iter().find(|(_, count)| **count > 1) {
        return Err(PackageLockError::duplicate_field(path, *field));
    }
    if let Some((field, _)) = counts
        .iter()
        .find(|(field, _)| !allowed.iter().any(|allowed| allowed == *field))
    {
        return Err(PackageLockError::unknown_field(path, *field));
    }
    Ok(())
}

fn required_value<'a>(
    members: &'a [JsonMember],
    path: &str,
    field: &str,
) -> PackageLockResult<&'a JsonValue> {
    members
        .iter()
        .find(|member| member.key() == field)
        .map(JsonMember::value)
        .ok_or_else(|| PackageLockError::missing_field(path, field))
}

fn required_string(members: &[JsonMember], path: &str, field: &str) -> PackageLockResult<String> {
    let value = required_value(members, path, field)?;
    value.string_value().map(ToOwned::to_owned).ok_or_else(|| {
        PackageLockError::wrong_type(
            field_path(path, field),
            Some(field.to_owned()),
            "string",
            value.kind().as_str(),
        )
    })
}

fn optional_string(
    members: &[JsonMember],
    path: &str,
    field: &str,
) -> PackageLockResult<Option<String>> {
    let Some(value) = members
        .iter()
        .find(|member| member.key() == field)
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    value
        .string_value()
        .map(|value| Some(value.to_owned()))
        .ok_or_else(|| {
            PackageLockError::wrong_type(
                field_path(path, field),
                Some(field.to_owned()),
                "string",
                value.kind().as_str(),
            )
        })
}

fn required_array<'a>(
    members: &'a [JsonMember],
    path: &str,
    field: &str,
) -> PackageLockResult<&'a [JsonValue]> {
    let value = required_value(members, path, field)?;
    value.array_elements().ok_or_else(|| {
        PackageLockError::wrong_type(
            field_path(path, field),
            Some(field.to_owned()),
            "array",
            value.kind().as_str(),
        )
    })
}

fn required_hash(
    members: &[JsonMember],
    path: &str,
    field: &str,
) -> PackageLockResult<PackageHash> {
    let field_path = field_path(path, field);
    let value = required_string(members, path, field)?;
    parse_package_hash(&value, &field_path)
        .map_err(|_| PackageLockError::invalid_hash_format(field_path, value))
}

fn validate_lock_module_name(name: &Name, path: impl Into<String>) -> PackageLockResult<()> {
    let path = path.into();
    if name.is_canonical() {
        Ok(())
    } else {
        Err(PackageLockError::invalid_module_name(
            path,
            name.as_dotted(),
        ))
    }
}

fn validate_lock_package_id(id: &PackageId, path: impl Into<String>) -> PackageLockResult<()> {
    let path = path.into();
    validate_package_id(id, &path)
        .map_err(|_| PackageLockError::invalid_package_id(path, id.as_str()))
}

fn validate_lock_package_version(
    version: &PackageVersion,
    path: impl Into<String>,
) -> PackageLockResult<()> {
    let path = path.into();
    validate_package_version(version, &path)
        .map_err(|_| PackageLockError::invalid_version(path, version.as_str()))
}

fn validate_lock_path(path: &PackagePath, error_path: impl Into<String>) -> PackageLockResult<()> {
    let error_path = error_path.into();
    validate_package_path(path, &error_path)
        .map_err(|_| PackageLockError::invalid_path(error_path, path.as_str()))
}

fn normalized_package_lock(lock: &PackageLockManifest) -> PackageLockManifest {
    let mut normalized = lock.clone();
    normalized
        .entries
        .sort_by(|left, right| left.module.cmp(&right.module));
    for entry in &mut normalized.entries {
        entry
            .imports
            .sort_by(|left, right| left.module.cmp(&right.module));
    }
    normalized
}

fn package_lock_json_unchecked(lock: &PackageLockManifest) -> String {
    json_object_in_order(vec![
        ("schema", json_string(&lock.schema)),
        ("package", json_string(lock.package.as_str())),
        ("version", json_string(lock.version.as_str())),
        ("manifest", manifest_reference_json(&lock.manifest)),
        (
            "entries",
            json_array(lock.entries.iter().map(entry_json_unchecked).collect()),
        ),
    ])
}

fn manifest_reference_json(manifest: &PackageLockManifestReference) -> String {
    json_object_in_order(vec![
        ("path", json_string(manifest.path.as_str())),
        ("file_hash", hash_json(manifest.file_hash)),
    ])
}

fn entry_json_unchecked(entry: &PackageLockEntry) -> String {
    let mut fields = vec![
        ("module", json_string(&entry.module.as_dotted())),
        ("origin", json_string(entry.origin.as_str())),
    ];
    if entry.origin == PackageLockEntryOrigin::External {
        fields.push((
            "package",
            json_string(
                entry
                    .package
                    .as_ref()
                    .expect("validated external entry has package")
                    .as_str(),
            ),
        ));
        fields.push((
            "version",
            json_string(
                entry
                    .version
                    .as_ref()
                    .expect("validated external entry has version")
                    .as_str(),
            ),
        ));
    }
    fields.extend([
        ("certificate", json_string(entry.certificate.as_str())),
        (
            "certificate_file_hash",
            hash_json(entry.certificate_file_hash),
        ),
        ("export_hash", hash_json(entry.export_hash)),
        ("axiom_report_hash", hash_json(entry.axiom_report_hash)),
        ("certificate_hash", hash_json(entry.certificate_hash)),
        (
            "imports",
            json_array(entry.imports.iter().map(import_json).collect()),
        ),
    ]);
    json_object_in_order(fields)
}

fn import_json(import: &PackageLockImport) -> String {
    json_object_in_order(vec![
        ("module", json_string(&import.module.as_dotted())),
        ("export_hash", hash_json(import.export_hash)),
        ("certificate_hash", hash_json(import.certificate_hash)),
    ])
}

fn json_object_in_order(fields: Vec<(&str, String)>) -> String {
    let mut out = String::new();
    out.push('{');
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&json_string(key));
        out.push(':');
        out.push_str(value);
    }
    out.push('}');
    out
}

fn json_array(values: Vec<String>) -> String {
    let mut out = String::new();
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

fn hash_json(hash: PackageHash) -> String {
    json_string(&format_package_hash(&hash))
}

fn json_string(value: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{000c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            '\u{0000}'..='\u{001f}' => {
                out.push_str("\\u00");
                out.push(hex_digit((ch as u8) >> 4));
                out.push(hex_digit((ch as u8) & 0x0f));
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex digit out of range"),
    }
}

fn field_path(path: &str, field: &str) -> String {
    if path == "$" {
        field.to_owned()
    } else {
        format!("{path}.{field}")
    }
}
