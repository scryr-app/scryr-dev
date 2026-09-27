//! Bounded Syft/Grype/Grant adapters linked to one exact inventory artifact.
//!
//! Grant's list JSON flattens SPDX expressions. Policy evaluation therefore uses
//! the original Syft expressions, after verifying Grant enumerated that inventory.
use chrono::{DateTime, Utc};
use crystal_core::{
    collectors::LicensePolicy,
    evidence::{
        DependencyEdge, InventoryResult, LicenseFinding, LicenseResult, Package,
        VulnerabilityFinding, VulnerabilityResult,
    },
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

/// Parse a structured scanner document without accepting empty or oversized output.
fn document(raw: &str) -> Result<Value, String> {
    if raw.len() > 32_000_000 {
        return Err("dependency report exceeds 32 MB".into());
    }
    let value: Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid dependency JSON: {e}"))?;
    if !value.is_object() {
        return Err("dependency report must be a JSON object".into());
    }
    Ok(value)
}

/// Read a required nonempty string; absent fields must never become clean evidence.
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 16_384)
        .ok_or_else(|| format!("dependency report requires nonempty {key}"))
}

/// Read a required bounded array.
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    value[key]
        .as_array()
        .filter(|a| a.len() <= 100_000)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("dependency report requires bounded {key} array"))
}

/// Produce a deterministic artifact or policy fingerprint.
fn hash(raw: &[u8]) -> String {
    Sha256::digest(raw)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        })
}

/// Optional scanner arrays may be omitted, but malformed values are not emptiness.
fn optional_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    if value[key].is_null() {
        Ok(&[])
    } else {
        array(value, key)
    }
}

/// Parse string arrays while retaining the distinction between malformed and empty.
fn strings(value: &Value) -> Result<Vec<String>, String> {
    let values = value.as_array().ok_or("expected a string array")?;
    values
        .iter()
        .map(|v| {
            v.as_str()
                .filter(|s| !s.is_empty() && s.len() <= 16_384)
                .map(str::to_owned)
                .ok_or_else(|| "expected a nonempty string array item".into())
        })
        .collect()
}

/// Deduplicate values into deterministic order.
fn unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Normalize virtual SBOM paths, stripping a known local catalog root.
fn source_path(path: &str, root: Option<&str>) -> Result<String, String> {
    let relative = root
        .and_then(|r| path.strip_prefix(r.trim_end_matches('/')))
        .filter(|p| p.starts_with('/'))
        .unwrap_or(path)
        .trim_start_matches("./")
        .trim_start_matches('/');
    if relative.is_empty() || relative.contains('\0') || relative.split('/').any(|p| p == "..") {
        return Err("invalid dependency source path".into());
    }
    Ok(relative.into())
}

/// Normalize the supported Syft JSON inventory; do not infer missing graph edges.
///
/// # Errors
/// Rejects malformed/unsupported documents, duplicate package IDs and bad edges.
pub(super) fn inventory(raw: &str) -> Result<InventoryResult, String> {
    let doc = document(raw)?;
    if text(&doc["descriptor"], "name")? != "syft" {
        return Err("inventory must be produced by Syft".into());
    }
    text(&doc["descriptor"], "version")?;
    let schema = text(&doc["schema"], "version")?;
    let major = schema.split('.').next().and_then(|s| s.parse::<u32>().ok());
    if !matches!(major, Some(12..=16)) {
        return Err(format!("unsupported Syft JSON schema {schema}"));
    }
    if text(&doc["source"], "type")? != "directory" {
        return Err("local inventory must describe a Syft directory source".into());
    }
    let root = Some(text(&doc["source"]["metadata"], "path")?);
    let mut packages = Vec::new();
    let mut ids = BTreeSet::new();
    let mut complete = true;
    for artifact in array(&doc, "artifacts")? {
        let id = text(artifact, "id")?.to_owned();
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate Syft package ID {id}"));
        }
        let name = text(artifact, "name")?.to_owned();
        let version = artifact["version"]
            .as_str()
            .ok_or("Syft package lacks version field")?
            .to_owned();
        let ecosystem = text(artifact, "type")?.to_owned();
        complete &= !version.is_empty() && ecosystem != "unknown";
        let paths = array(artifact, "locations")?
            .iter()
            .map(|location| {
                let path = location["path"]
                    .as_str()
                    .or_else(|| location["realPath"].as_str())
                    .ok_or("Syft location lacks path")?;
                source_path(path, root)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let licenses = array(artifact, "licenses")?
            .iter()
            .map(|license| {
                let expression = license["spdxExpression"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .or_else(|| license["value"].as_str())
                    .or_else(|| license.as_str())
                    .ok_or("Syft license lacks expression or value")?;
                if expression.len() > 4096 {
                    return Err("license expression exceeds 4096 bytes".into());
                }
                Ok(expression.to_owned())
            })
            .collect::<Result<Vec<_>, String>>()?;
        packages.push(Package {
            id,
            name,
            version,
            ecosystem,
            purl: artifact["purl"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_owned),
            paths: unique(paths),
            licenses: unique(licenses),
        });
    }
    let mut edges = BTreeSet::new();
    for relation in array(&doc, "artifactRelationships")? {
        // Source/file ownership relationships are not package dependencies.
        if text(relation, "type")? != "dependency-of" {
            continue;
        }
        let dependency = text(relation, "parent")?;
        let dependent = text(relation, "child")?;
        if !ids.contains(dependency) || !ids.contains(dependent) {
            return Err("Syft dependency edge names an unknown package".into());
        }
        // Syft's dependency-of edge points dependency -> dependent. Scryr's
        // dependency graph is dependent -> dependency, like Manifest.connections.
        edges.insert((dependent.to_owned(), dependency.to_owned()));
    }
    packages.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(InventoryResult {
        packages,
        relationships: edges
            .into_iter()
            .map(|(from, to)| DependencyEdge { from, to })
            .collect(),
        complete,
        artifact_hash: hash(raw.as_bytes()),
    })
}

/// Match Grype package identity against the exact Syft artifact, never by name alone.
fn grype_package<'a>(
    artifact: &Value,
    inventory: &'a InventoryResult,
) -> Result<&'a Package, String> {
    let id = text(artifact, "id")?;
    let package = inventory
        .packages
        .iter()
        .find(|p| p.id == id)
        .ok_or("Grype finding references a different inventory")?;
    if text(artifact, "name")? != package.name
        || artifact["version"].as_str() != Some(package.version.as_str())
        || text(artifact, "type")? != package.ecosystem
    {
        return Err("Grype package identity differs from the inventory".into());
    }
    Ok(package)
}

/// Extract a safe report URL for display.
fn http_url(value: &Value) -> Option<String> {
    value
        .as_str()
        .filter(|s| s.starts_with("https://") || s.starts_with("http://"))
        .map(str::to_owned)
}

/// Supported Grype matching ecosystems; unknown ecosystems cannot imply a clean scan.
fn supported_ecosystem(ecosystem: &str) -> bool {
    matches!(
        ecosystem,
        "apk"
            | "deb"
            | "rpm"
            | "alpm"
            | "portage"
            | "python"
            | "npm"
            | "java-archive"
            | "java"
            | "gem"
            | "go-module"
            | "rust-crate"
            | "dotnet"
            | "php-composer"
            | "dart-pub"
    )
}

/// Merge one advisory group with an existing alias-connected group.
fn merge_finding(target: &mut VulnerabilityFinding, other: VulnerabilityFinding) {
    target.aliases = unique(
        target
            .aliases
            .iter()
            .cloned()
            .chain(other.aliases)
            .chain([other.advisory_id]),
    );
    target.aliases.retain(|id| id != &target.advisory_id);
    target.fix_versions = unique(
        target
            .fix_versions
            .iter()
            .cloned()
            .chain(other.fix_versions),
    );
    if target.severity != other.severity {
        target.severity = "unknown".into();
    }
    if target.url.is_none() {
        target.url = other.url;
    }
}

/// Read supported database provenance and reject explicit scanner validity failures.
fn grype_database(descriptor: &Value) -> Result<(Option<u64>, Option<String>), String> {
    let db = &descriptor["db"];
    if !db.is_object() {
        return Err("Grype output lacks database status".into());
    }
    // Grype DB v6 places provider provenance beside a nested status object.
    // The earlier flat DB status remains a distinct supported scanner shape.
    let db = match db.get("status") {
        None => db,
        Some(status) if status.is_object() => status,
        Some(_) => return Err("Grype output has malformed database status".into()),
    };
    if !db["error"].is_null() && db["error"].as_str() != Some("") {
        return Err("Grype reports a database error".into());
    }
    if !db["valid"].is_null() && db["valid"].as_bool() != Some(true) {
        return Err("Grype reports an invalid database".into());
    }
    let built = db["built"].as_str().or_else(|| db["built_at"].as_str());
    let built = built.and_then(|s| DateTime::parse_from_rfc3339(s).ok());
    let database_age_seconds = built
        .and_then(|time| u64::try_from(Utc::now().signed_duration_since(time).num_seconds()).ok());
    let database_version = db["checksum"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .or_else(|| db["schemaVersion"].as_u64().map(|v| v.to_string()))
        .or_else(|| db["schemaVersion"].as_str().map(str::to_owned));
    Ok((database_age_seconds, database_version))
}

/// Normalize a Grype JSON snapshot, preserving unknown coverage/database metadata.
///
/// # Errors
/// Rejects missing report structure, failed databases and mismatched inventory IDs.
pub(super) fn vulnerabilities(
    raw: &str,
    inventory: &InventoryResult,
) -> Result<VulnerabilityResult, String> {
    let doc = document(raw)?;
    if text(&doc["descriptor"], "name")? != "grype" {
        return Err("vulnerabilities must be produced by Grype".into());
    }
    text(&doc["descriptor"], "version")?;
    let (database_age_seconds, database_version) = grype_database(&doc["descriptor"])?;
    let unfiltered = optional_array(&doc, "ignoredMatches")?.is_empty()
        && optional_array(&doc, "alertsByPackage")?.is_empty();
    let complete = inventory.complete
        && database_age_seconds.is_some()
        && database_version.is_some()
        && unfiltered
        && inventory
            .packages
            .iter()
            .all(|p| supported_ecosystem(&p.ecosystem));
    let mut groups: Vec<Option<VulnerabilityFinding>> = Vec::new();
    let mut aliases: BTreeMap<(String, String), usize> = BTreeMap::new();
    for finding in array(&doc, "matches")? {
        let package = grype_package(&finding["artifact"], inventory)?;
        let vulnerability = &finding["vulnerability"];
        let advisory_id = text(vulnerability, "id")?.to_owned();
        let mut related = BTreeSet::from([advisory_id.clone()]);
        for alias in optional_array(finding, "relatedVulnerabilities")? {
            related.insert(text(alias, "id")?.to_owned());
        }
        for alias in optional_array(vulnerability, "advisories")? {
            related.insert(text(alias, "id")?.to_owned());
        }
        let severity = vulnerability["severity"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown")
            .to_ascii_lowercase();
        let fix_versions = match &vulnerability["fix"]["versions"] {
            Value::Null => vec![],
            values => unique(strings(values)?),
        };
        let mut current = VulnerabilityFinding {
            package_id: package.id.clone(),
            advisory_id,
            aliases: related.iter().cloned().collect(),
            severity,
            fix_versions,
            url: http_url(&vulnerability["dataSource"]),
        };
        let connected: BTreeSet<_> = related
            .iter()
            .filter_map(|id| aliases.get(&(package.id.clone(), id.clone())).copied())
            .collect();
        let index = connected.iter().next().copied().unwrap_or(groups.len());
        for old in connected {
            if let Some(Some(previous)) = groups.get_mut(old).map(Option::take) {
                merge_finding(&mut current, previous);
            }
        }
        for alias in current
            .aliases
            .iter()
            .chain(std::iter::once(&current.advisory_id))
        {
            aliases.insert((package.id.clone(), alias.clone()), index);
        }
        current.aliases.retain(|id| id != &current.advisory_id);
        if index == groups.len() {
            groups.push(Some(current));
        } else {
            groups[index] = Some(current);
        }
    }
    let mut items: Vec<_> = groups.into_iter().flatten().collect();
    items.sort_by(|a, b| (&a.package_id, &a.advisory_id).cmp(&(&b.package_id, &b.advisory_id)));
    Ok(VulnerabilityResult {
        items,
        inventory_hash: inventory.artifact_hash.clone(),
        database_age_seconds,
        database_version,
        complete,
    })
}

/// Three-valued policy outcome; missing evidence never becomes allow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Allow,
    Deny,
    Review,
}
impl Decision {
    /// Every conjunct must be permitted.
    const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Deny, _) | (_, Self::Deny) => Self::Deny,
            (Self::Review, _) | (_, Self::Review) => Self::Review,
            _ => Self::Allow,
        }
    }
    /// One permitted alternative suffices; unknown alternatives require review.
    const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Allow, _) | (_, Self::Allow) => Self::Allow,
            (Self::Review, _) | (_, Self::Review) => Self::Review,
            _ => Self::Deny,
        }
    }
    /// Stable canonical wire decision.
    const fn name(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Review => "review",
        }
    }
}

/// Bounded SPDX syntax tree; WITH is an indivisible license-plus-exception atom.
#[derive(Debug, PartialEq, Eq)]
enum Expression {
    License(String),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}
impl Expression {
    /// Normalize whitespace/parentheses for exact expression rules.
    fn canonical(&self) -> String {
        match self {
            Self::License(s) => s.clone(),
            Self::And(a, b) => format!("({} AND {})", a.canonical(), b.canonical()),
            Self::Or(a, b) => format!("({} OR {})", a.canonical(), b.canonical()),
        }
    }
    /// Evaluate with AND precedence and explicit exception permissions.
    fn evaluate(
        &self,
        allow: &BTreeSet<String>,
        deny: &BTreeSet<String>,
        unknown: Decision,
    ) -> Decision {
        let key = self.canonical();
        if deny.contains(&key) {
            return Decision::Deny;
        }
        if allow.contains(&key) {
            return Decision::Allow;
        }
        match self {
            Self::License(_) => unknown,
            Self::And(a, b) => a
                .evaluate(allow, deny, unknown)
                .and(b.evaluate(allow, deny, unknown)),
            Self::Or(a, b) => a
                .evaluate(allow, deny, unknown)
                .or(b.evaluate(allow, deny, unknown)),
        }
    }
}

/// Tokenize and parse SPDX's AND/OR/WITH grammar with strict resource bounds.
fn expression(value: &str) -> Result<Expression, String> {
    if value.is_empty() || value.len() > 4096 {
        return Err("empty or oversized SPDX expression".into());
    }
    let spaced = value.replace('(', " ( ").replace(')', " ) ");
    let tokens: Vec<_> = spaced.split_whitespace().collect();
    if tokens.len() > 512 {
        return Err("SPDX expression exceeds token limit".into());
    }
    let mut cursor = 0;
    let parsed = parse_or(&tokens, &mut cursor, 0)?;
    if cursor != tokens.len() {
        return Err("trailing SPDX expression tokens".into());
    }
    Ok(parsed)
}

/// Parse alternatives, whose precedence is lower than conjunctions.
fn parse_or(tokens: &[&str], cursor: &mut usize, depth: usize) -> Result<Expression, String> {
    let mut left = parse_and(tokens, cursor, depth)?;
    while tokens.get(*cursor) == Some(&"OR") {
        *cursor += 1;
        left = Expression::Or(Box::new(left), Box::new(parse_and(tokens, cursor, depth)?));
    }
    Ok(left)
}

/// Parse conjunctions of license atoms or parenthesized expressions.
fn parse_and(tokens: &[&str], cursor: &mut usize, depth: usize) -> Result<Expression, String> {
    let mut left = parse_atom(tokens, cursor, depth)?;
    while tokens.get(*cursor) == Some(&"AND") {
        *cursor += 1;
        left = Expression::And(Box::new(left), Box::new(parse_atom(tokens, cursor, depth)?));
    }
    Ok(left)
}

/// A simple SPDX identifier, including user `LicenseRef` identifiers and plus notation.
fn identifier(token: &str) -> bool {
    !token.is_empty()
        && !matches!(token, "AND" | "OR" | "WITH" | "NONE" | "NOASSERTION")
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | ':'))
}

/// Parse one parenthesized expression or indivisible license/exception pair.
fn parse_atom(tokens: &[&str], cursor: &mut usize, depth: usize) -> Result<Expression, String> {
    if depth > 64 {
        return Err("SPDX expression exceeds nesting limit".into());
    }
    let token = *tokens.get(*cursor).ok_or("incomplete SPDX expression")?;
    *cursor += 1;
    if token == "(" {
        let inner = parse_or(tokens, cursor, depth + 1)?;
        if tokens.get(*cursor) != Some(&")") {
            return Err("unclosed SPDX expression".into());
        }
        *cursor += 1;
        return Ok(inner);
    }
    if !identifier(token) {
        return Err("unsupported SPDX license token".into());
    }
    let mut license = token.to_owned();
    if tokens.get(*cursor) == Some(&"WITH") {
        *cursor += 1;
        let exception = *tokens.get(*cursor).ok_or("missing SPDX exception")?;
        if !identifier(exception) {
            return Err("invalid SPDX exception".into());
        }
        *cursor += 1;
        license.push_str(" WITH ");
        license.push_str(exception);
    }
    Ok(Expression::License(license))
}

/// Resolve Grant's regenerated package IDs back to exact inventory identities.
fn grant_packages<'a>(
    finding: &Value,
    inventory: &'a InventoryResult,
) -> Result<Vec<&'a Package>, String> {
    text(finding, "id")?;
    let name = text(finding, "name")?;
    let kind = text(finding, "type")?;
    let version = finding["version"]
        .as_str()
        .ok_or("Grant package lacks version")?;
    let locations = strings(&finding["locations"])?;
    let mut candidates: Vec<_> = inventory
        .packages
        .iter()
        .filter(|p| p.name == name && p.version == version && p.ecosystem == kind)
        .collect();
    if let Some(group) = finding["group"].as_str().filter(|s| !s.is_empty()) {
        let prefix = format!("pkg:maven/{group}/{name}@");
        candidates.retain(|p| {
            p.purl
                .as_deref()
                .is_some_and(|url| url.starts_with(&prefix))
        });
    }
    if candidates.len() > 1 {
        candidates.retain(|p| {
            p.paths.iter().any(|path| {
                locations.iter().any(|other| {
                    other.trim_start_matches("./").trim_start_matches('/') == path
                        || other.ends_with(&format!("/{path}"))
                })
            })
        });
        if candidates.is_empty() {
            return Err("Grant package identity is ambiguous within inventory".into());
        }
    }
    if candidates.is_empty() {
        return Err("Grant findings reference a different inventory".into());
    }
    Ok(candidates)
}

/// Apply project policy to a package's original unflattened license assertions.
fn license_finding(
    package: &Package,
    allow: &BTreeSet<String>,
    deny: &BTreeSet<String>,
    unknown: Decision,
) -> LicenseFinding {
    // Separate Syft license assertions are jointly applicable. An OR inside
    // one assertion remains a choice, not a flattened set of obligations.
    let combined = (!package.licenses.is_empty()).then(|| {
        package
            .licenses
            .iter()
            .map(|v| format!("({v})"))
            .collect::<Vec<_>>()
            .join(" AND ")
    });
    let (decision, reason) = match combined.as_deref().map(expression) {
        None => (unknown, "No license evidence in the inventory".into()),
        Some(Err(error)) => (
            unknown,
            format!("Unrecognized license expression uses the unknown-license policy: {error}"),
        ),
        Some(Ok(value)) => {
            let decision = value.evaluate(allow, deny, unknown);
            (
                decision,
                match decision {
                    Decision::Allow => {
                        "Declared license obligations satisfy the explicit project policy"
                    }
                    Decision::Deny => "Declared license obligations are denied by project policy",
                    Decision::Review => "License or exception is not covered by project policy",
                }
                .into(),
            )
        }
    };
    LicenseFinding {
        package_id: package.id.clone(),
        expression: combined,
        decision: decision.name().into(),
        reason,
    }
}

/// Validate Grant's full list response and evaluate the original SPDX expressions.
///
/// # Errors
/// Rejects malformed/error/filtered output, foreign packages and invalid policies.
pub(super) fn licenses(
    raw: &str,
    inventory: &InventoryResult,
    policy: &LicensePolicy,
) -> Result<LicenseResult, String> {
    let doc = document(raw)?;
    if text(&doc, "tool")? != "grant" {
        return Err("license report must be produced by Grant".into());
    }
    text(&doc, "version")?;
    let targets = array(&doc["run"], "targets")?;
    if targets.len() != 1 {
        return Err("Grant must report exactly one inventory target".into());
    }
    let evaluation = &targets[0]["evaluation"];
    if text(evaluation, "status")? != "unevaluated" {
        return Err("expected successful unfiltered Grant list output".into());
    }
    let findings = array(&evaluation["findings"], "packages")?;
    let total = evaluation["summary"]["packages"]["total"]
        .as_u64()
        .ok_or("Grant summary lacks package total")?;
    if total != u64::try_from(findings.len()).map_err(|e| e.to_string())? {
        return Err("Grant package summary is incomplete".into());
    }
    let mut observed = BTreeSet::new();
    for finding in findings {
        if text(finding, "decision")? != "unevaluated" {
            return Err("Grant list must not substitute ambient policy decisions".into());
        }
        for license in array(finding, "licenses")? {
            text(license, "id")?;
        }
        for package in grant_packages(finding, inventory)? {
            observed.insert(package.id.as_str());
        }
    }
    if observed.len() != inventory.packages.len() {
        return Err("Grant omitted packages from the supplied inventory".into());
    }
    let allow: BTreeSet<_> = policy
        .allow
        .iter()
        .map(|v| expression(v).map(|e| e.canonical()))
        .collect::<Result<_, _>>()?;
    let deny: BTreeSet<_> = policy
        .deny
        .iter()
        .map(|v| expression(v).map(|e| e.canonical()))
        .collect::<Result<_, _>>()?;
    if !allow.is_disjoint(&deny) {
        return Err("license policy both allows and denies an expression".into());
    }
    let unknown = match policy.unknown.as_str() {
        "review" => Decision::Review,
        "deny" => Decision::Deny,
        _ => return Err("invalid unknown-license policy".into()),
    };
    let items = inventory
        .packages
        .iter()
        .map(|package| license_finding(package, &allow, &deny, unknown))
        .collect();
    Ok(LicenseResult {
        items,
        policy_revision: hash(&serde_json::to_vec(policy).map_err(|e| e.to_string())?),
        inventory_hash: inventory.artifact_hash.clone(),
        complete: inventory.complete,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn syft() -> Value {
        json!({"descriptor":{"name":"syft","version":"1.33.0"},"schema":{"version":"16.1.3"},
            "source":{"type":"directory","metadata":{"path":"/workspace"}},
            "artifacts":[
                {"id":"p1","name":"widget","version":"1.0","type":"python","purl":"pkg:pypi/widget@1.0",
                    "locations":[{"path":"/workspace/uv.lock"}],"licenses":[{"spdxExpression":"MIT OR GPL-3.0-only","value":"MIT OR GPL-3.0-only"}]},
                {"id":"p2","name":"support","version":"2.0","type":"python","locations":[{"path":"uv.lock"}],"licenses":[]}],
            "artifactRelationships":[{"parent":"p2","child":"p1","type":"dependency-of"},{"parent":"p1","child":"file-id","type":"contains"}]})
    }

    fn grant() -> Value {
        json!({"tool":"grant","version":"0.4.0","run":{"targets":[{"source":{"type":"file","ref":"inventory.syft.json"},"evaluation":{
        "status":"unevaluated","summary":{"packages":{"total":3}},"findings":{"packages":[
            {"id":"python:widget@1.0","name":"widget","type":"python","version":"1.0","decision":"unevaluated","locations":["uv.lock"],"licenses":[{"id":"MIT"},{"id":"GPL-3.0-only"}]},
            {"id":"python:widget@1.0","name":"widget","type":"python","version":"1.0","decision":"unevaluated","locations":["uv.lock"],"licenses":[{"id":"MIT"},{"id":"GPL-3.0-only"}]},
            {"id":"python:support@2.0","name":"support","type":"python","version":"2.0","decision":"unevaluated","locations":["uv.lock"],"licenses":[]}
        ]}}}]}})
    }

    fn grype() -> Value {
        json!({"descriptor":{"name":"grype","version":"0.104.0","db":{"built":"2026-01-01T00:00:00Z","schemaVersion":6,"checksum":"sha256:db","error":null}},"matches":[
            {"artifact":{"id":"p1","name":"widget","version":"1.0","type":"python"},"vulnerability":{"id":"GHSA-123","severity":"High","dataSource":"https://advisories.example/GHSA-123","fix":{"versions":["1.1"]}},"relatedVulnerabilities":[{"id":"CVE-2026-123"}]},
            {"artifact":{"id":"p1","name":"widget","version":"1.0","type":"python"},"vulnerability":{"id":"CVE-2026-123","severity":"High","fix":{"versions":["1.1"]}},"relatedVulnerabilities":[{"id":"GHSA-123"}]}
        ]})
    }

    #[test]
    fn inventory_keeps_identity_expressions_and_only_true_dependency_edges() -> Result<(), String> {
        let raw = syft().to_string();
        let result = inventory(&raw)?;
        assert_eq!(result.packages.len(), 2);
        assert_eq!(result.packages[0].licenses, ["MIT OR GPL-3.0-only"]);
        assert_eq!(result.packages[0].paths, ["uv.lock"]);
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].from, "p1");
        assert_eq!(result.relationships[0].to, "p2");
        assert_eq!(result.artifact_hash, hash(raw.as_bytes()));
        assert!(result.complete);
        Ok(())
    }

    #[test]
    fn malformed_or_unknown_inventory_never_becomes_empty_success() {
        for raw in ["", "{}", "[]", r#"{"artifacts":[]}"#] {
            assert!(inventory(raw).is_err());
        }
        let mut doc = syft();
        doc["schema"]["version"] = json!("999.0");
        assert!(inventory(&doc.to_string()).is_err());
        let mut doc = syft();
        doc["artifacts"][1]["id"] = json!("p1");
        assert!(inventory(&doc.to_string()).is_err());
        let mut doc = syft();
        doc["artifactRelationships"][0]["parent"] = json!("missing");
        assert!(inventory(&doc.to_string()).is_err());
        let mut doc = syft();
        doc["source"] = Value::Null;
        assert!(inventory(&doc.to_string()).is_err());
    }

    #[test]
    fn grype_aliases_are_deduplicated_per_affected_package() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        let result = vulnerabilities(&grype().to_string(), &input)?;
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].package_id, "p1");
        assert_eq!(result.items[0].aliases.len(), 1);
        assert_eq!(result.items[0].fix_versions, ["1.1"]);
        assert_eq!(result.items[0].severity, "high");
        assert_eq!(result.inventory_hash, input.artifact_hash);
        assert!(result.database_age_seconds.is_some());
        assert!(result.complete);
        Ok(())
    }

    #[test]
    fn grype_rejects_foreign_inventory_and_database_errors() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        let mut doc = grype();
        doc["matches"][0]["artifact"]["id"] = json!("other");
        assert!(vulnerabilities(&doc.to_string(), &input).is_err());
        let mut doc = grype();
        doc["descriptor"]["db"]["error"] = json!("database unavailable");
        assert!(vulnerabilities(&doc.to_string(), &input).is_err());
        assert!(vulnerabilities("{\"matches\":[]}", &input).is_err());
        Ok(())
    }

    #[test]
    fn modern_grype_database_status_preserves_validity_and_provenance() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        let mut doc = grype();
        // Shape emitted by grype/cmd/grype/cli/commands/root.go::dbInfo and
        // grype/vulnerability/provider.go::ProviderStatus.MarshalJSON.
        doc["descriptor"]["db"] = json!({
            "status": {"schemaVersion":"6.0.3", "built":"2026-01-01T00:00:00Z",
                "valid":true, "from":"https://grype.anchore.io/databases"},
            "providers": {"github":{"captured":"2026-01-01T00:00:00Z","input":"sha256:data"}}
        });
        let result = vulnerabilities(&doc.to_string(), &input)?;
        assert_eq!(result.database_version.as_deref(), Some("6.0.3"));
        assert!(result.database_age_seconds.is_some());
        assert!(result.complete);
        doc["descriptor"]["db"]["status"]["valid"] = json!(false);
        assert!(vulnerabilities(&doc.to_string(), &input).is_err());
        doc["descriptor"]["db"]["status"] = Value::Null;
        assert!(vulnerabilities(&doc.to_string(), &input).is_err());
        Ok(())
    }

    #[test]
    fn scanner_documents_without_database_evidence_are_not_clean() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        // Upstream grype/presenter/json/testdata/snapshot/TestEmptyJsonPresenter.golden.
        let upstream_empty = r#"{"matches":[],"source":{"type":"unknown","target":"unknown"},"distro":{"name":"centos","version":"8.0","idLike":["rhel"]},"descriptor":{"name":"grype","version":"[not provided]","timestamp":""}}"#;
        assert!(vulnerabilities(upstream_empty, &input).is_err());
        let mut doc = grype();
        doc["ignoredMatches"] = json!({"unexpected":"shape"});
        assert!(vulnerabilities(&doc.to_string(), &input).is_err());
        let mut doc = grype();
        doc["matches"][0]["relatedVulnerabilities"] = json!({});
        assert!(vulnerabilities(&doc.to_string(), &input).is_err());
        Ok(())
    }

    #[test]
    fn unknown_or_filtered_scan_coverage_is_partial() -> Result<(), String> {
        let mut input = inventory(&syft().to_string())?;
        input.packages[1].ecosystem = "unsupported".into();
        let result = vulnerabilities(&grype().to_string(), &input)?;
        assert!(!result.complete);
        let input = inventory(&syft().to_string())?;
        let mut doc = grype();
        doc["descriptor"]["db"]["built"] = Value::Null;
        let result = vulnerabilities(&doc.to_string(), &input)?;
        assert!(result.database_age_seconds.is_none());
        assert!(!result.complete);
        let mut doc = grype();
        doc["ignoredMatches"] = json!([{}]);
        assert!(!vulnerabilities(&doc.to_string(), &input)?.complete);
        Ok(())
    }

    #[test]
    fn grant_duplicate_license_rows_preserve_one_policy_result_per_package() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        let policy = LicensePolicy {
            allow: vec!["MIT".into()],
            deny: vec!["GPL-3.0-only".into()],
            unknown: "review".into(),
        };
        let result = licenses(&grant().to_string(), &input, &policy)?;
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].decision, "allow");
        assert_eq!(
            result.items[0].expression.as_deref(),
            Some("(MIT OR GPL-3.0-only)")
        );
        assert_eq!(result.items[1].decision, "review");
        assert_eq!(result.inventory_hash, input.artifact_hash);
        Ok(())
    }

    #[test]
    fn grant_error_or_missing_rows_cannot_show_policy_pass() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        let mut doc = grant();
        doc["run"]["targets"][0]["evaluation"]["status"] = json!("error");
        assert!(licenses(&doc.to_string(), &input, &LicensePolicy::default()).is_err());
        let mut doc = grant();
        doc["run"]["targets"][0]["evaluation"]["findings"]["packages"] = json!([]);
        doc["run"]["targets"][0]["evaluation"]["summary"]["packages"]["total"] = json!(0);
        assert!(licenses(&doc.to_string(), &input, &LicensePolicy::default()).is_err());
        assert!(licenses("{}", &input, &LicensePolicy::default()).is_err());
        Ok(())
    }

    #[test]
    fn grant_merged_identity_links_absolute_locations_to_inventory_packages() -> Result<(), String>
    {
        let mut doc = syft();
        let mut duplicate = doc["artifacts"][0].clone();
        duplicate["id"] = json!("p3");
        duplicate["locations"] = json!([{"path":"/workspace/nested/uv.lock"}]);
        doc["artifacts"]
            .as_array_mut()
            .ok_or("fixture artifacts")?
            .push(duplicate);
        let input = inventory(&doc.to_string())?;
        let mut doc = grant();
        for i in 0..2 {
            doc["run"]["targets"][0]["evaluation"]["findings"]["packages"][i]["locations"] =
                json!(["/workspace/uv.lock", "/workspace/nested/uv.lock"]);
        }
        let result = licenses(&doc.to_string(), &input, &LicensePolicy::default())?;
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[2].package_id, "p3");
        Ok(())
    }

    #[test]
    fn spdx_and_or_precedence_and_exceptions_are_not_flattened() -> Result<(), String> {
        let allow = BTreeSet::from([
            "MIT".into(),
            "Apache-2.0".into(),
            "GPL-2.0-only WITH Classpath-exception-2.0".into(),
        ]);
        let deny = BTreeSet::from(["GPL-3.0-only".into(), "GPL-2.0-only".into()]);
        for (value, expected) in [
            ("MIT OR GPL-3.0-only", Decision::Allow),
            ("MIT AND GPL-3.0-only", Decision::Deny),
            ("MIT OR Apache-2.0 AND GPL-3.0-only", Decision::Allow),
            ("(MIT OR Apache-2.0) AND GPL-3.0-only", Decision::Deny),
            ("MIT AND LicenseRef-Unknown", Decision::Review),
            ("GPL-3.0-only OR LicenseRef-Unknown", Decision::Review),
            ("GPL-2.0-only WITH Classpath-exception-2.0", Decision::Allow),
            ("MIT WITH Unlisted-exception", Decision::Review),
        ] {
            assert_eq!(
                expression(value)?.evaluate(&allow, &deny, Decision::Review),
                expected,
                "{value}"
            );
        }
        Ok(())
    }

    #[test]
    fn unsupported_expression_syntax_is_rejected_without_panics() {
        for value in [
            "",
            "MIT AND (",
            "MIT OR",
            "(MIT) WITH exception",
            "MIT Apache-2.0",
            "MIT WITH",
            "NONE",
            "NOASSERTION",
            "MIT && Apache-2.0",
        ] {
            assert!(expression(value).is_err(), "{value}");
        }
        assert!(expression(&format!("{}MIT{}", "(".repeat(100), ")".repeat(100))).is_err());
    }

    #[test]
    fn unknown_policy_and_policy_revision_are_explicit() -> Result<(), String> {
        let input = inventory(&syft().to_string())?;
        let review = licenses(&grant().to_string(), &input, &LicensePolicy::default())?;
        let deny = licenses(
            &grant().to_string(),
            &input,
            &LicensePolicy {
                unknown: "deny".into(),
                ..LicensePolicy::default()
            },
        )?;
        assert_eq!(review.items[1].decision, "review");
        assert_eq!(deny.items[1].decision, "deny");
        assert_ne!(review.policy_revision, deny.policy_revision);
        assert_eq!(review.inventory_hash, deny.inventory_hash);
        Ok(())
    }
}
