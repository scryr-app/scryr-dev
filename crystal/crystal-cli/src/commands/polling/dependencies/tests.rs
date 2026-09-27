//! Provider conversion and subprocess regression coverage, without GitHub credentials.
#![allow(clippy::missing_docs_in_private_items)]
use super::*;

fn manifest(id: &str) -> Value {
    json!({"manifestId":id,"github":{"repoUrl":"https://github.com/example/api"},"dependencies":{"source":{"provider":"github"}}})
}
fn sbom() -> Value {
    json!({"sbom":{"spdxVersion":"SPDX-2.3","packages":[
        {"SPDXID":"SPDXRef-Repository","name":"example/api"},
        {"SPDXID":"A","name":"requests","versionInfo":"2.32.0","licenseConcluded":"Apache-2.0","externalRefs":[{"referenceType":"purl","referenceLocator":"pkg:pypi/requests@2.32.0"}]},
        {"SPDXID":"B","name":"urllib3","versionInfo":"2.2.0"}
    ],"relationships":[
        {"relationshipType":"DEPENDS_ON","spdxElementId":"SPDXRef-Repository","relatedSpdxElement":"A"},
        {"relationshipType":"DEPENDS_ON","spdxElementId":"A","relatedSpdxElement":"B"}
    ]}})
}
fn alert(number: u64) -> Value {
    json!({"number":number,"state":"open","html_url":format!("https://github.com/example/api/security/dependabot/{number}"),
        "dependency":{"package":{"name":"requests","ecosystem":"pip"},"manifest_path":"api/requirements.txt"},
        "security_advisory":{"ghsa_id":"GHSA-test-test-test","cve_id":"CVE-2026-1234","summary":"Example finding"},
        "security_vulnerability":{"severity":"high","vulnerable_version_range":"< 2.33","first_patched_version":{"identifier":"2.33"}}})
}

#[test]
fn selections_are_opt_in_and_shared_by_repository() -> Result<(), String> {
    assert!(targets(&[json!({"github":{"repoUrl":"https://github.com/example/api"}})])?.is_empty());
    let mut inventory_only = manifest("web");
    inventory_only["dependencies"]["source"]["security"] = json!(false);
    let selected = targets(&[manifest("api"), inventory_only, manifest("api")])?;
    let selected = selected.get("example/api").ok_or("missing target")?;
    assert_eq!(selected.inventory.len(), 2);
    assert_eq!(selected.security.len(), 1);
    for field in ["inventory", "security"] {
        let mut invalid = manifest("api");
        invalid["dependencies"]["source"][field] = json!("true");
        assert!(targets(&[invalid]).is_err());
    }
    let mut invalid = manifest("api");
    invalid["dependencies"]["source"] =
        json!({"provider":"github","inventory":false,"security":false});
    assert!(targets(&[invalid]).is_err());
    Ok(())
}

#[test]
fn inventory_excludes_root_and_does_not_invent_relationships() -> Result<(), String> {
    let inventory = parse::inventory(&sbom())?;
    assert_eq!(inventory.packages.len(), 2);
    assert_eq!(inventory.direct_deps, Some(1));
    assert_eq!(inventory.transitive_deps, Some(1));
    assert!(
        inventory
            .packages
            .iter()
            .any(|p| p.license.as_deref() == Some("Apache-2.0"))
    );
    let mut declared = sbom();
    declared["sbom"]["packages"][1]["licenseConcluded"] = json!("NOASSERTION");
    declared["sbom"]["packages"][1]["licenseDeclared"] = json!("MIT");
    assert!(
        parse::inventory(&declared)?
            .packages
            .iter()
            .any(|p| p.license.as_deref() == Some("MIT"))
    );
    let mut incomplete = sbom();
    incomplete["sbom"]["relationships"] = json!([]);
    assert_eq!(parse::inventory(&incomplete)?.direct_deps, None);
    let mut duplicate = sbom();
    duplicate["sbom"]["packages"]
        .as_array_mut()
        .ok_or("packages")?
        .push(json!({"SPDXID":"A","name":"duplicate"}));
    assert!(parse::inventory(&duplicate).is_err());
    assert!(parse::inventory(&json!({"sbom":{}})).is_err());
    let empty = json!({"spdxVersion":"SPDX-2.3","packages":[]});
    assert!(parse::inventory(&empty)?.packages.is_empty());
    Ok(())
}

#[test]
fn security_requires_complete_valid_pages_and_preserves_remediation() -> Result<(), String> {
    let security = parse::security(&json!([[alert(1)], [alert(2)]]), "example/api")?;
    assert_eq!(security.alerts.len(), 2);
    assert_eq!(
        security.alerts[0].first_patched_version.as_deref(),
        Some("2.33")
    );
    assert_eq!(security.alerts[0].cve_id.as_deref(), Some("CVE-2026-1234"));
    assert!(
        parse::security(&json!([[]]), "example/api")?
            .alerts
            .is_empty()
    );
    for value in [
        json!([]),
        json!([{}]),
        json!([[alert(1)], [alert(1)]]),
        json!([[{"number":1}]]),
    ] {
        assert!(parse::security(&value, "example/api").is_err());
    }
    let mut invalid = alert(1);
    invalid["html_url"] = json!("https://evil.example/alert/1");
    assert!(parse::security(&json!([[invalid]]), "example/api").is_err());
    Ok(())
}

#[test]
fn asynchronous_report_urls_cannot_escape_the_selected_repository() {
    let valid = "https://api.github.com/repos/example/api/dependency-graph/sbom/fetch-report/4bab1a7e-da63-4828-9488-44e0e01a7c1b";
    assert!(export_path(&json!({"sbom_url":valid}), "example/api").is_ok());
    assert!(
        export_path(
            &json!({"sbom_url":valid.replace("example/api", "Example/API")}),
            "example/api"
        )
        .is_ok()
    );
    for url in [
        valid.replace("api.github.com", "evil.example"),
        valid.replace("example/api", "other/repo"),
        format!("{valid}?token=secret"),
        valid.replace("https:", "http:"),
    ] {
        assert!(export_path(&json!({"sbom_url":url}), "example/api").is_err());
    }
}

#[cfg(unix)]
fn fake_gh() -> Result<(tempfile::TempDir, Gh), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("gh");
    let quoted = directory
        .path()
        .display()
        .to_string()
        .replace('\'', "'\\''");
    std::fs::write(
        &executable,
        format!(
            r#"#!/bin/sh
set -eu
cd '{quoted}'
printf '%s\n' "$6" >> calls
case "$6" in
  */generate-report)
    if test -f legacy; then echo 'gh: Not Found (HTTP 404)' >&2; exit 1; fi
    cat generated.json ;;
  */fetch-report/*)
    if test -f pending; then rm pending; exit 0; fi
    cat sbom.json ;;
  */dependency-graph/sbom) cat sbom.json ;;
  */dependabot/alerts\?*)
    test "$7" = --paginate
    test "$8" = --slurp
    if test -f denied; then echo 'HTTP 403 private-detail' >&2; exit 1; fi
    cat alerts.json ;;
  *) exit 2 ;;
esac
"#
        ),
    )?;
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))?;
    std::fs::write(directory.path().join("sbom.json"), sbom().to_string())?;
    std::fs::write(
        directory.path().join("alerts.json"),
        json!([[alert(1)], [alert(2)]]).to_string(),
    )?;
    std::fs::write(directory.path().join("generated.json"), json!({"sbom_url":"https://api.github.com/repos/example/api/dependency-graph/sbom/fetch-report/4bab1a7e-da63-4828-9488-44e0e01a7c1b"}).to_string())?;
    Ok((directory, Gh { executable }))
}

#[cfg(unix)]
#[tokio::test]
async fn subprocess_handles_async_legacy_exports_and_denied_alerts()
-> Result<(), Box<dyn std::error::Error>> {
    let (directory, gh) = fake_gh()?;
    std::fs::write(directory.path().join("pending"), "")?;
    assert_eq!(
        inventory(&gh, "example/api", &mut PendingExports::default())
            .await?
            .packages
            .len(),
        2
    );
    std::fs::write(directory.path().join("legacy"), "")?;
    assert_eq!(
        inventory(&gh, "example/api", &mut PendingExports::default())
            .await?
            .packages
            .len(),
        2
    );
    assert_eq!(security(&gh, "example/api").await?.alerts.len(), 2);
    std::fs::write(directory.path().join("denied"), "")?;
    let error = security(&gh, "example/api")
        .await
        .err()
        .ok_or("expected error")?;
    assert!(error.contains("Dependabot alerts read access"));
    assert!(!error.contains("private-detail"));
    Ok(())
}

#[derive(Default)]
struct State {
    snapshots: Vec<Value>,
    statuses: Vec<Value>,
}
async fn graphql(
    state: actix_web::web::Data<std::sync::Mutex<State>>,
    body: actix_web::web::Json<Value>,
) -> actix_web::HttpResponse {
    let Ok(mut state) = state.lock() else {
        return actix_web::HttpResponse::InternalServerError().finish();
    };
    let data = if body["query"]
        .as_str()
        .is_some_and(|q| q.contains("recordGithubDependencies"))
    {
        state.snapshots.push(body["variables"].clone());
        json!({"recordGithubDependencies":true})
    } else {
        state.statuses.push(body["variables"].clone());
        json!({"recordProviderSync":true})
    };
    actix_web::HttpResponse::Ok().json(json!({"data":data}))
}

#[cfg(unix)]
#[actix_web::test]
async fn shared_collection_preserves_partial_success_and_recovers()
-> Result<(), Box<dyn std::error::Error>> {
    let state = actix_web::web::Data::new(std::sync::Mutex::new(State::default()));
    let cloned = state.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("http://{}/graphql", listener.local_addr()?);
    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(cloned.clone())
            .route("/graphql", actix_web::web::post().to(graphql))
    })
    .listen(listener)?
    .run();
    let handle = server.handle();
    let task = actix_web::rt::spawn(server);
    let client = Client::new(&endpoint, None).await?;
    let (directory, gh) = fake_gh()?;
    let targets = targets(&[manifest("api"), manifest("web")])?;
    assert_eq!(
        collect(&client, &gh, &targets, &mut PendingExports::default())
            .await?
            .snapshots,
        4
    );
    let calls = std::fs::read_to_string(directory.path().join("calls"))?;
    assert_eq!(
        calls
            .lines()
            .filter(|line| line.contains("/dependabot/"))
            .count(),
        1
    );
    std::fs::write(directory.path().join("denied"), "")?;
    assert!(
        collect(&client, &gh, &targets, &mut PendingExports::default())
            .await
            .is_err()
    );
    {
        let state = state.lock().map_err(|_| "state")?;
        assert_eq!(state.snapshots.len(), 6);
        assert_eq!(
            state
                .statuses
                .iter()
                .filter(|s| s["error"].is_string())
                .count(),
            2
        );
        assert!(state.snapshots[4]["snapshot"]["security"].is_null());
        drop(state);
    }
    std::fs::remove_file(directory.path().join("denied"))?;
    assert_eq!(
        collect(&client, &gh, &targets, &mut PendingExports::default())
            .await?
            .snapshots,
        4
    );
    handle.stop(true).await;
    task.await??;
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn slow_exports_resume_the_same_report_after_a_collection_deadline()
-> Result<(), Box<dyn std::error::Error>> {
    let (directory, gh) = fake_gh()?;
    let mut pending = PendingExports::default();
    std::fs::write(directory.path().join("pending"), "")?;
    assert!(
        inventory_with_timeout(&gh, "example/api", &mut pending, Duration::from_millis(200))
            .await
            .is_err()
    );
    assert_eq!(pending.len(), 1);
    assert_eq!(
        inventory(&gh, "example/api", &mut pending)
            .await?
            .packages
            .len(),
        2
    );
    assert!(pending.is_empty());
    let calls = std::fs::read_to_string(directory.path().join("calls"))?;
    assert_eq!(
        calls
            .lines()
            .filter(|line| line.ends_with("generate-report"))
            .count(),
        1
    );
    Ok(())
}
