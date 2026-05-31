//! Package manifest parsing entry points.

/// Parse package manifest TOML into a structured value without reading files.
pub fn parse_toml_value(source: &str) -> Result<toml::Value, toml::de::Error> {
    source.parse()
}

#[cfg(test)]
mod tests {
    use super::parse_toml_value;

    #[test]
    fn package_manifest_skeleton_uses_structured_toml_parser() {
        let parsed = parse_toml_value("schema = \"npa.package.v0.1\"").unwrap();
        assert_eq!(parsed["schema"].as_str(), Some("npa.package.v0.1"));
    }
}
