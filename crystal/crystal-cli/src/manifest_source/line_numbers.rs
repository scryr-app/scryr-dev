//! Manifest assignment line-number extraction.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Scryr constructors commonly assigned to top-level manifest variables.
const SCRYR_TOP_LEVEL_CONSTRUCTORS: &[&str] = &["Diagram", "Forge", "Manifest"];

/// Find simple assignment line numbers for manifest variables.
pub(crate) fn manifest_line_numbers(
    manifest_file: &Path,
) -> Result<HashMap<String, usize>, String> {
    let source = fs::read_to_string(manifest_file).map_err(|error| {
        format!(
            "Failed to read manifest file {} for line metadata: {error}",
            manifest_file.display()
        )
    })?;
    let mut line_numbers = HashMap::new();

    for (index, line) in source.lines().enumerate() {
        if let Some(variable) = assignment_variable_name(line) {
            line_numbers.entry(variable).or_insert(index + 1);
        }
    }

    Ok(line_numbers)
}

/// Return the assigned variable name for simple `name = ...` or `name: Type = ...` lines.
fn assignment_variable_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }

    let equals_index = trimmed.find('=')?;
    let left = trimmed[..equals_index].trim();
    let variable = left.split_once(':').map_or(left, |(name, _)| name).trim();
    if is_python_identifier(variable)
        || is_scry_declaration_name(variable)
            && is_scryr_constructor_assignment(&trimmed[equals_index + 1..])
    {
        Some(variable.to_string())
    } else {
        None
    }
}

/// Return whether text is a Scryr declaration name that Python cannot bind directly.
fn is_scry_declaration_name(value: &str) -> bool {
    value.contains('-')
        && value.chars().all(|character| {
            character == '-' || character == '_' || character.is_ascii_alphanumeric()
        })
}

/// Return whether the assignment right-hand side starts with a Scryr constructor call.
fn is_scryr_constructor_assignment(right: &str) -> bool {
    let right = right.trim_start();
    SCRYR_TOP_LEVEL_CONSTRUCTORS
        .iter()
        .copied()
        .any(|constructor| {
            right
                .strip_prefix(constructor)
                .is_some_and(|rest| rest.trim_start().starts_with('('))
        })
}

/// Return whether text is a plain Python identifier.
fn is_python_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::{assignment_variable_name, manifest_line_numbers};
    use std::path::Path;

    #[test]
    fn assignment_variable_name_supports_plain_and_annotated_assignments() {
        assert_eq!(
            assignment_variable_name("api = Manifest(name='API')"),
            Some("api".to_string())
        );
        assert_eq!(
            assignment_variable_name("api: Manifest = Manifest(name='API')"),
            Some("api".to_string())
        );
        assert_eq!(
            assignment_variable_name("api: Final[Manifest] = Manifest(name='API')"),
            Some("api".to_string())
        );
        assert_eq!(
            assignment_variable_name("ex-northwind-commerce-diagram = Diagram(name='API')"),
            Some("ex-northwind-commerce-diagram".to_string())
        );
        assert_eq!(assignment_variable_name("# api = Manifest()"), None);
        assert_eq!(assignment_variable_name("items[0] = Manifest()"), None);
    }

    #[test]
    fn manifest_line_numbers_include_public_manifest_variables() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("samples/open_saas/index.scry");

        let line_numbers = manifest_line_numbers(&manifest_file)?;

        assert!(line_numbers.contains_key("postgres_block"));
        assert!(line_numbers.contains_key("server_block"));
        Ok(())
    }
}
