//! Bounded OpenMetrics/Prometheus text parsing and windowed local measurements.
#![allow(clippy::missing_docs_in_private_items)]
use chrono::{DateTime, Utc};
use crystal_core::collectors::{MetricSelection, OpenmetricsConfig};
use crystal_core::evidence::{MetricLabel, MetricSample, MetricsResult};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(super) struct Snapshot {
    pub samples: BTreeMap<(String, BTreeMap<String, String>), f64>,
    types: BTreeMap<String, String>,
    units: BTreeMap<String, String>,
    pub at: DateTime<Utc>,
    complete: bool,
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_:".contains(c))
}
fn labels(value: &str) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut rest = value.trim();
    while !rest.is_empty() {
        let (name, after) = rest.split_once('=').ok_or("invalid metric label")?;
        let name = name.trim();
        if !valid_name(name) || out.len() >= 20 {
            return Err("invalid or too many metric labels".into());
        }
        let after = after.trim_start();
        if !after.starts_with('"') {
            return Err("metric label value must be quoted".into());
        }
        let mut escaped = false;
        let mut end = None;
        for (i, c) in after.char_indices().skip(1) {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                end = Some(i + 1);
                break;
            }
        }
        let end = end.ok_or("unterminated metric label")?;
        let decoded: String = serde_json::from_str(&after[..end]).map_err(|e| e.to_string())?;
        if decoded.len() > 512 || out.insert(name.into(), decoded).is_some() {
            return Err("duplicate or oversized metric label".into());
        }
        rest = after[end..].trim_start();
        if !rest.is_empty() {
            rest = rest
                .strip_prefix(',')
                .ok_or("invalid label separator")?
                .trim_start();
        }
    }
    Ok(out)
}

pub(super) fn parse(
    text: &str,
    content_type: &str,
    max_series: usize,
    at: DateTime<Utc>,
) -> Result<Snapshot, String> {
    if text.len() > 4_000_000 {
        return Err("metrics response exceeds 4 MB".into());
    }
    let openmetrics = content_type.starts_with("application/openmetrics-text");
    if !openmetrics && !content_type.is_empty() && !content_type.starts_with("text/plain") {
        return Err("unsupported metrics exposition content type".into());
    }
    if openmetrics && content_type.contains("version=") && !content_type.contains("version=1.0.0") {
        return Err("supported OpenMetrics text version is 1.0.0".into());
    }
    if !openmetrics && content_type.contains("version=") && !content_type.contains("version=0.0.4")
    {
        return Err("supported Prometheus text version is 0.0.4".into());
    }
    if openmetrics && text.lines().last().map(str::trim) != Some("# EOF") {
        return Err("OpenMetrics response is missing # EOF".into());
    }
    let mut snapshot = Snapshot {
        at,
        complete: true,
        ..Snapshot::default()
    };
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(metadata) = line
            .strip_prefix("# TYPE ")
            .or_else(|| line.strip_prefix("# UNIT "))
        {
            let (name, value) = metadata.split_once(' ').ok_or("invalid metric metadata")?;
            if !valid_name(name) {
                return Err("invalid metric name".into());
            }
            if line.starts_with("# TYPE") {
                snapshot.types.insert(name.into(), value.trim().into());
            } else {
                snapshot.units.insert(name.into(), value.trim().into());
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let end = line
            .find(|c: char| c == '{' || c.is_whitespace())
            .ok_or("metric sample has no value")?;
        let name = &line[..end];
        if !valid_name(name) {
            return Err("invalid metric sample name".into());
        }
        let (sample_labels, rest) = if line[end..].starts_with('{') {
            let mut quoted = false;
            let mut escaped = false;
            let mut close = None;
            for (i, c) in line[end + 1..].char_indices() {
                if escaped {
                    escaped = false;
                } else if c == '\\' && quoted {
                    escaped = true;
                } else if c == '"' {
                    quoted = !quoted;
                } else if c == '}' && !quoted {
                    close = Some(end + 1 + i);
                    break;
                }
            }
            let close = close.ok_or("unterminated metric labels")?;
            (labels(&line[end + 1..close])?, &line[close + 1..])
        } else {
            (BTreeMap::new(), &line[end..])
        };
        let value = rest
            .split_whitespace()
            .next()
            .ok_or("missing metric value")?
            .parse::<f64>()
            .map_err(|_| "unsupported or malformed metric value")?;
        if !value.is_finite() {
            snapshot.complete = false;
            continue;
        }
        if snapshot
            .samples
            .insert((name.into(), sample_labels), value)
            .is_some()
        {
            return Err("duplicate metric sample".into());
        }
        if snapshot.samples.len() > max_series {
            return Err(format!(
                "metrics series limit ({max_series}) exceeded; select a bounded exporter endpoint"
            ));
        }
    }
    Ok(snapshot)
}

fn unit(snapshot: &Snapshot, metric: &str) -> Option<String> {
    snapshot.units.get(metric).cloned().or_else(|| {
        if metric.ends_with("_bytes") {
            Some("bytes".into())
        } else if metric.ends_with("_seconds") {
            Some("seconds".into())
        } else {
            None
        }
    })
}
fn elapsed(current: &Snapshot, previous: &Snapshot) -> Option<f64> {
    let duration = current
        .at
        .signed_duration_since(previous.at)
        .to_std()
        .ok()?
        .as_secs_f64();
    (duration > 0.0 && duration <= 300.0).then_some(duration)
}
fn delta(current: f64, previous: f64) -> Option<f64> {
    if current < 0.0 || previous < 0.0 {
        None
    } else {
        Some(if current >= previous {
            current - previous
        } else {
            current
        })
    }
}
fn rate(current: &Snapshot, previous: &Snapshot, metric: &str) -> Option<f64> {
    let duration = elapsed(current, previous)?;
    let mut total = 0.0;
    let mut count = 0;
    for (key, value) in current
        .samples
        .iter()
        .filter(|((name, _), _)| name == metric)
    {
        total += delta(*value, *previous.samples.get(key)?)?;
        count += 1;
    }
    (count > 0).then_some(total / duration)
}
fn quantile(current: &Snapshot, previous: &Snapshot, metric: &str, percentile: f64) -> Option<f64> {
    elapsed(current, previous)?;
    let name = format!("{metric}_bucket");
    let mut groups: BTreeMap<BTreeMap<String, String>, BTreeMap<String, f64>> = BTreeMap::new();
    for (key, value) in current.samples.iter().filter(|((n, _), _)| n == &name) {
        let mut labels = key.1.clone();
        let bound = labels.remove("le")?;
        groups
            .entry(labels)
            .or_default()
            .insert(bound, delta(*value, *previous.samples.get(key)?)?);
    }
    let expected: BTreeSet<_> = groups.values().next()?.keys().cloned().collect();
    if groups
        .values()
        .any(|g| g.keys().cloned().collect::<BTreeSet<_>>() != expected)
    {
        return None;
    }
    let mut buckets: Vec<(f64, f64)> = expected
        .iter()
        .map(|bound| {
            let boundary = if bound == "+Inf" {
                f64::INFINITY
            } else {
                bound.parse().ok()?
            };
            Some((
                boundary,
                groups
                    .values()
                    .map(|g| g.get(bound).copied().unwrap_or(0.0))
                    .sum(),
            ))
        })
        .collect::<Option<_>>()?;
    buckets.sort_by(|a, b| a.0.total_cmp(&b.0));
    if buckets.len() < 2
        || buckets.last()?.0 != f64::INFINITY
        || buckets.windows(2).any(|w| w[1].1 < w[0].1)
    {
        return None;
    }
    let total = buckets.last()?.1;
    if total <= 0.0 {
        return None;
    }
    let rank = total * percentile / 100.0;
    let (mut lower, mut previous_count) = (0.0, 0.0);
    for (bound, count) in buckets {
        if count >= rank {
            if bound.is_infinite() {
                return Some(lower);
            }
            if (count - previous_count).abs() < f64::EPSILON {
                return Some(bound);
            }
            return Some(
                (bound - lower).mul_add((rank - previous_count) / (count - previous_count), lower),
            );
        }
        lower = bound;
        previous_count = count;
    }
    None
}

pub(super) fn measurements(
    config: &OpenmetricsConfig,
    current: &Snapshot,
    previous: Option<&Snapshot>,
) -> MetricsResult {
    let mut result = MetricsResult {
        samples: vec![],
        scraped_at: current.at,
        complete: current.complete,
    };
    if config.series.is_empty() {
        result.samples = current
            .samples
            .iter()
            .map(|((name, labels), value)| MetricSample {
                name: name.clone(),
                title: None,
                labels: labels
                    .iter()
                    .map(|(name, value)| MetricLabel {
                        name: name.clone(),
                        value: value.clone(),
                    })
                    .collect(),
                value: *value,
                unit: unit(current, name),
                metric_type: current
                    .types
                    .get(name)
                    .or_else(|| {
                        name.strip_suffix("_total")
                            .and_then(|base| current.types.get(base))
                    })
                    .cloned()
                    .unwrap_or_else(|| "untyped".into()),
            })
            .collect();
        return result;
    }
    for selection in &config.series {
        let (metric, title, value, metric_type, selected_unit) = match selection {
            MetricSelection::CounterRate { metric, title } => (
                metric,
                title,
                previous.and_then(|p| rate(current, p, metric)),
                "rate",
                Some("events/s".into()),
            ),
            MetricSelection::Gauge { metric, title } => {
                let values: Vec<_> = current
                    .samples
                    .iter()
                    .filter(|((name, _), _)| name == metric)
                    .map(|(_, v)| *v)
                    .collect();
                (
                    metric,
                    title,
                    (!values.is_empty()).then(|| values.iter().sum()),
                    "gauge",
                    unit(current, metric),
                )
            }
            MetricSelection::HistogramPercentile {
                metric,
                title,
                percentile,
            } => (
                metric,
                title,
                previous.and_then(|p| quantile(current, p, metric, *percentile)),
                "histogram_percentile",
                unit(current, metric),
            ),
        };
        if let Some(value) = value.filter(|v| v.is_finite()) {
            result.samples.push(MetricSample {
                name: metric.clone(),
                title: title.clone(),
                labels: vec![],
                value,
                unit: selected_unit,
                metric_type: metric_type.into(),
            });
        } else {
            result.complete = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resets_labels_bounds_and_first_sample() -> Result<(), String> {
        let before = parse(
            "requests_total{route=\"/a\"} 20\nrequests_total{route=\"/b\"} 4",
            "text/plain",
            10,
            Utc::now(),
        )?;
        let after = parse(
            "requests_total{route=\"/a\"} 3\nrequests_total{route=\"/b\"} 8",
            "text/plain",
            10,
            before.at + chrono::Duration::seconds(1),
        )?;
        assert_eq!(rate(&after, &before, "requests_total"), Some(7.0));
        assert!(parse("x 1\nx 2", "text/plain", 10, before.at).is_err());
        assert!(
            parse(
                "x 1",
                "application/openmetrics-text; version=1.0.0",
                10,
                before.at
            )
            .is_err()
        );
        assert!(parse("a 1\nb 2", "text/plain", 1, before.at).is_err());
        let mut config: OpenmetricsConfig =
            serde_json::from_value(serde_json::json!({"endpoint":"http://127.0.0.1/metrics"}))
                .map_err(|e| e.to_string())?;
        config.series.push(MetricSelection::CounterRate {
            metric: "requests_total".into(),
            title: None,
        });
        let initial = measurements(&config, &before, None);
        assert!(!initial.complete && initial.samples.is_empty());
        Ok(())
    }
    #[test]
    fn histogram_uses_bucket_deltas() -> Result<(), String> {
        let at = Utc::now();
        let before = parse(
            "latency_bucket{le=\"0.1\"} 1\nlatency_bucket{le=\"1\"} 2\nlatency_bucket{le=\"+Inf\"} 2",
            "text/plain",
            10,
            at,
        )?;
        let after = parse(
            "latency_bucket{le=\"0.1\"} 6\nlatency_bucket{le=\"1\"} 12\nlatency_bucket{le=\"+Inf\"} 12",
            "text/plain",
            10,
            at + chrono::Duration::seconds(30),
        )?;
        assert_eq!(quantile(&after, &before, "latency", 50.0), Some(0.1));
        Ok(())
    }
}
