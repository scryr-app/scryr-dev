//! Strict parsers for test artifacts, without executing the test runner.
#![allow(clippy::missing_docs_in_private_items)]
use crystal_core::evidence::{CoverageResult, TestCase, TestResult};
use std::collections::{BTreeMap, HashSet};

pub(super) fn junit(files: &[String], suite: &str) -> Result<TestResult, String> {
    let mut result = TestResult {
        suite: suite.into(),
        passing: 0,
        failing: 0,
        errors: 0,
        skipped: 0,
        duration_seconds: 0.0,
        cases: Vec::new(),
    };
    let mut identities = HashSet::new();
    for file in files {
        let doc = roxmltree::Document::parse(file).map_err(|e| e.to_string())?;
        if !matches!(
            doc.root_element().tag_name().name(),
            "testsuites" | "testsuite"
        ) {
            return Err("expected JUnit testsuite(s)".into());
        }
        let mut count = 0;
        for case in doc.descendants().filter(|n| n.has_tag_name("testcase")) {
            let name = case
                .attribute("name")
                .ok_or("JUnit testcase missing name")?;
            let case_suite = case
                .attribute("classname")
                .or_else(|| case.parent().and_then(|p| p.attribute("name")))
                .unwrap_or(suite);
            if !identities.insert((case_suite.to_owned(), name.to_owned())) {
                return Err(
                    "duplicate testcase identity; report overlapping suites separately".into(),
                );
            }
            count += 1;
            let status = if case.children().any(|n| n.has_tag_name("error")) {
                result.errors += 1;
                "error"
            } else if case.children().any(|n| n.has_tag_name("failure")) {
                result.failing += 1;
                "failed"
            } else if case.children().any(|n| n.has_tag_name("skipped")) {
                result.skipped += 1;
                "skipped"
            } else {
                result.passing += 1;
                "passed"
            };
            let duration = case
                .attribute("time")
                .unwrap_or("0")
                .parse::<f64>()
                .map_err(|_| "invalid testcase duration")?;
            if !duration.is_finite() || duration < 0.0 {
                return Err("invalid testcase duration".into());
            }
            result.duration_seconds += duration;
            let message = case
                .children()
                .find(|n| {
                    n.has_tag_name("error")
                        || n.has_tag_name("failure")
                        || n.has_tag_name("skipped")
                })
                .and_then(|n| n.attribute("message").or_else(|| n.text()))
                .map(|s| s.chars().take(2000).collect());
            result.cases.push(TestCase {
                name: name.into(),
                suite: case_suite.into(),
                status: status.into(),
                message,
                duration_seconds: duration,
            });
            if result.cases.len() > 10_000 {
                return Err(
                    "JUnit exceeds 10000 test cases; split suites into separate collectors".into(),
                );
            }
        }
        if count == 0
            && doc
                .descendants()
                .any(|n| n.attribute("tests").is_some_and(|s| s != "0"))
        {
            return Err("JUnit summary has tests but no testcase details".into());
        }
    }
    if files.is_empty() {
        return Err("no test report artifacts were found".into());
    }
    Ok(result)
}

pub(super) fn coverage(
    files: &[String],
    format: &str,
    suite: &str,
) -> Result<CoverageResult, String> {
    let mut lines: BTreeMap<(String, u64), bool> = BTreeMap::new();
    for file in files {
        match format {
            "cobertura" => {
                let doc = roxmltree::Document::parse(file).map_err(|e| e.to_string())?;
                if !doc.root_element().has_tag_name("coverage") {
                    return Err("expected Cobertura coverage".into());
                }
                for class in doc.descendants().filter(|n| n.has_tag_name("class")) {
                    let path = class
                        .attribute("filename")
                        .ok_or("coverage class missing filename")?;
                    for line in class
                        .children()
                        .filter(|n| n.has_tag_name("lines"))
                        .flat_map(|n| n.children())
                        .filter(|n| n.has_tag_name("line"))
                    {
                        let number = line
                            .attribute("number")
                            .ok_or("missing line number")?
                            .parse()
                            .map_err(|_| "invalid line number")?;
                        let hits: u64 = line
                            .attribute("hits")
                            .ok_or("missing hits")?
                            .parse()
                            .map_err(|_| "invalid hits")?;
                        *lines.entry((path.into(), number)).or_default() |= hits > 0;
                    }
                }
                if !doc.descendants().any(|n| n.has_tag_name("line"))
                    && doc
                        .root_element()
                        .attribute("lines-valid")
                        .is_some_and(|v| v != "0")
                {
                    return Err("coverage summary requires line details".into());
                }
            }
            "lcov" => {
                let mut path = None;
                for line in file.lines() {
                    if let Some(p) = line.strip_prefix("SF:") {
                        path = Some(p.to_owned());
                    } else if line == "end_of_record" {
                        path = None;
                    } else if let Some(data) = line.strip_prefix("DA:") {
                        let mut parts = data.split(',');
                        let number = parts
                            .next()
                            .ok_or("missing LCOV line")?
                            .parse()
                            .map_err(|_| "invalid LCOV line")?;
                        let hits: u64 = parts
                            .next()
                            .ok_or("missing LCOV count")?
                            .parse()
                            .map_err(|_| "invalid LCOV count")?;
                        *lines
                            .entry((path.clone().ok_or("DA before SF")?, number))
                            .or_default() |= hits > 0;
                    }
                }
                if !file.lines().any(|l| l.starts_with("SF:")) {
                    return Err("expected LCOV source records".into());
                }
            }
            _ => return Err("unknown coverage integration".into()),
        }
    }
    if files.is_empty() {
        return Err("no coverage artifacts were found".into());
    }
    Ok(CoverageResult {
        suite: suite.into(),
        covered: lines.values().filter(|v| **v).count() as u64,
        total: lines.len() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn junit_valid_failures_and_corrupt_artifacts() -> Result<(), String> {
        let result = junit(&[r#"<testsuites><testsuite name="a"><testcase name="ok" time="1"/><testcase name="bad"><failure message="failed"/></testcase><testcase name="error"><error/></testcase><testcase name="skip"><skipped/></testcase></testsuite></testsuites>"#.into()], "unit")?;
        assert_eq!(
            (
                result.passing,
                result.failing,
                result.errors,
                result.skipped
            ),
            (1, 1, 1, 1)
        );
        assert_eq!(result.cases[1].message.as_deref(), Some("failed"));
        assert!(junit(&["<testsuite tests=\"2\"/>".into()], "unit").is_err());
        assert!(
            junit(
                &["<testsuite><testcase name=\"x\" time=\"NaN\"/></testsuite>".into()],
                "unit"
            )
            .is_err()
        );
        assert!(junit(&[], "unit").is_err());
        Ok(())
    }
    #[test]
    fn coverage_deduplicates_source_lines() -> Result<(), String> {
        let report = coverage(
            &[
                "SF:a.py\nDA:1,0\nDA:2,1\nend_of_record".into(),
                "SF:a.py\nDA:1,1\nend_of_record".into(),
            ],
            "lcov",
            "unit",
        )?;
        assert_eq!((report.covered, report.total), (2, 2));
        assert!(coverage(&["invalid".into()], "lcov", "unit").is_err());
        Ok(())
    }
}
