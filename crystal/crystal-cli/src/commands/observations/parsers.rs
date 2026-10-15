//! Strict parsers for common CI artifact formats.
use crystal_core::reports::ReportData;
use std::collections::BTreeMap;

pub(super) fn tests(files: &[String], format: &str) -> Result<ReportData, String> {
    if format != "junit" {
        return Err("supported test format: junit".into());
    }
    let (mut passing, mut failing, mut errors, mut skipped, mut duration) = (0, 0, 0, 0, 0.0);
    let mut identities = std::collections::HashSet::new();
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
            if let Some(name) = case.attribute("name") {
                let identity = (
                    case.attribute("classname").unwrap_or("").to_owned(),
                    name.to_owned(),
                );
                if !identities.insert(identity) {
                    return Err(
                        "duplicate testcase identity; report overlapping shards separately".into(),
                    );
                }
            }
            count += 1;
            if case.children().any(|n| n.has_tag_name("error")) {
                errors += 1;
            } else if case.children().any(|n| n.has_tag_name("failure")) {
                failing += 1;
            } else if case.children().any(|n| n.has_tag_name("skipped")) {
                skipped += 1;
            } else {
                passing += 1;
            }
            let t = case
                .attribute("time")
                .unwrap_or("0")
                .parse::<f64>()
                .map_err(|_| "invalid testcase duration")?;
            if !t.is_finite() || t < 0.0 {
                return Err("invalid testcase duration".into());
            }
            duration += t;
        }
        if count == 0
            && doc
                .descendants()
                .any(|n| n.attribute("tests").is_some_and(|s| s != "0"))
        {
            return Err("JUnit summary has tests but no testcase details".into());
        }
    }
    Ok(ReportData::Tests {
        passing,
        failing,
        errors,
        skipped,
        duration,
    })
}

pub(super) fn coverage(files: &[String], format: &str) -> Result<ReportData, String> {
    // Deduplicate overlapping source lines instead of averaging file percentages.
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
                if !doc.descendants().any(|node| node.has_tag_name("line"))
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
                        let n = parts
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
                            .entry((path.clone().ok_or("DA before SF")?, n))
                            .or_default() |= hits > 0;
                    }
                }
                if !file.lines().any(|l| l.starts_with("SF:")) {
                    return Err("expected LCOV source records".into());
                }
            }
            _ => return Err("supported coverage formats: cobertura, lcov".into()),
        }
    }
    Ok(ReportData::Coverage {
        covered: lines.values().filter(|hit| **hit).count() as u64,
        total: lines.len() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_junit_counts_cases_once() -> Result<(), String> {
        let xml = r#"<testsuites tests="4"><testsuite tests="4"><testcase time="1"/><testcase><failure/></testcase><testcase><error/></testcase><testcase><skipped/></testcase></testsuite></testsuites>"#;
        let ReportData::Tests {
            passing,
            failing,
            errors,
            skipped,
            duration,
        } = super::tests(&[xml.into()], "junit")?
        else {
            return Err("wrong type".into());
        };
        assert_eq!((passing, failing, errors, skipped), (1, 1, 1, 1));
        assert!((duration - 1.0).abs() < f64::EPSILON);
        assert!(super::tests(&["<testsuite tests=\"2\"/>".into()], "junit").is_err());
        assert!(
            super::tests(
                &["<testsuite><testcase time=\"NaN\"/></testsuite>".into()],
                "junit"
            )
            .is_err()
        );
        Ok(())
    }
    #[test]
    fn coverage_merges_lines_without_averaging() -> Result<(), String> {
        let ReportData::Coverage { covered, total } = coverage(
            &[
                "SF:a.py\nDA:1,0\nDA:2,1\nend_of_record".into(),
                "SF:a.py\nDA:1,1\nend_of_record".into(),
            ],
            "lcov",
        )?
        else {
            return Err("wrong type".into());
        };
        assert_eq!((covered, total), (2, 2));
        assert!(coverage(&["not a report".into()], "lcov").is_err());
        let ReportData::Coverage{covered,total}=coverage(&[r#"<coverage><packages><package><classes><class filename="a.py"><lines><line number="1" hits="0"/><line number="2" hits="2"/></lines></class></classes></package></packages></coverage>"#.into()],"cobertura")? else {return Err("wrong type".into())};
        assert_eq!((covered, total), (1, 2));
        Ok(())
    }
}
