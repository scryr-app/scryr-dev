//! Editing contract tests run against both storage drivers.
use crystal_core::manifest::ManifestRequestContext;
use crystal_core::persistence::documents::*;
use crystal_core::persistence::{
    DatabasePool, persist_generated_manifest, read_generated_manifest_json_by_scry_identifier,
};
use serde_json::json;

fn context(org: &str) -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "editor".into(),
        clerk_org_id: org.into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    }
}
fn envelope(name: &str) -> serde_json::Value {
    json!({"files":[{"path":"index.scry","content":format!("api = Manifest(name='{name}')")},{"path":"helper.py","content":"VALUE = 1"}],
        "manifests":[{"name":name,"variable_name":"api","line_number":1},{"name":"Worker","variable_name":"worker","line_number":2}],
        "forges":[{"name":"Forge","variable_name":"forge","line_number":3}],
        "diagrams":[{"name":"Main","variable_name":"main_diagram","manifests":[{"name":name}]},{"name":"Jobs","variable_name":"jobs_diagram","manifests":[{"name":"Worker"}]}]})
}
async fn execute(pool: &DatabasePool, sql: &'static str) -> Result<(), String> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        DatabasePool::Turso(database) => {
            database
                .connect()
                .map_err(|e| e.to_string())?
                .execute(sql, ())
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub(crate) async fn exercise(pool: DatabasePool) -> Result<(), String> {
    let org = context("org_a");
    let original = envelope("Before");
    let doc = ManifestDocument {
        identifier: "main_diagram".into(),
        key: "main".into(),
        folder_path: "project".into(),
        entrypoint: "index.scry".into(),
        files: serde_json::from_value(original["files"].clone()).map_err(|e| e.to_string())?,
        revision: String::new(),
        writable: true,
        local: false,
    };
    for input in prepare_save(&doc, &original)? {
        persist_generated_manifest(&pool, &input, &org)
            .await
            .map_err(|e| e.to_string())?;
    }
    let doc = read_document(&pool, "org_a", "main_diagram").await?;
    assert!(read_document(&pool, "org_b", "main_diagram").await.is_err());
    for input in prepare_save(&doc, &original)? {
        persist_generated_manifest(&pool, &input, &context("org_b"))
            .await
            .map_err(|e| e.to_string())?;
    }
    assert!(
        save_document(
            &pool,
            &context("org_b"),
            &doc,
            &prepare_save(&doc, &original)?
        )
        .await
        .is_err()
    );
    let modified = envelope("After");
    let inputs = prepare_save(&doc, &modified)?;
    save_document(&pool, &org, &doc, &inputs).await?;
    let blocks = read_generated_manifest_json_by_scry_identifier(&pool, "org_a", "main_diagram")
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(blocks.as_array().map(Vec::len), Some(1));
    assert_eq!(blocks[0]["name"], "After");
    let sibling = read_document(&pool, "org_a", "jobs_diagram").await?;
    assert!(sibling.files[0].content.contains("After"));
    save_document(&pool, &org, &sibling, &prepare_save(&sibling, &modified)?).await?;
    assert_eq!(
        read_document(&pool, "org_a", "main_diagram").await?.key,
        "main"
    );
    assert!(
        read_document(&pool, "org_b", "main_diagram").await?.files[0]
            .content
            .contains("Before")
    );
    assert!(
        save_document(&pool, &org, &doc, &inputs).await.is_err(),
        "stale revisions must fail"
    );
    let current = read_document(&pool, "org_a", "main_diagram").await?;
    // Fail the second diagram write after the first succeeds; neither source nor
    // diagram data may partially commit on either driver.
    execute(&pool, "CREATE TRIGGER fail_editor BEFORE UPDATE ON generated_manifests WHEN NEW.scry_identifier = 'jobs_diagram' BEGIN SELECT RAISE(ABORT, 'forced editor failure'); END").await?;
    let failed = prepare_save(&current, &envelope("Should roll back"))?;
    assert!(save_document(&pool, &org, &current, &failed).await.is_err());
    assert_eq!(
        read_document(&pool, "org_a", "main_diagram")
            .await?
            .revision,
        current.revision
    );
    execute(&pool, "DROP TRIGGER fail_editor").await?;
    let mut unauthorized = context("org_a");
    unauthorized.clerk_org_role = Some("org:member".into());
    assert!(
        save_document(&pool, &unauthorized, &current, &inputs)
            .await
            .is_err()
    );
    let mut invalid = envelope("Invalid");
    invalid["diagrams"][1]["manifests"] = json!([{"name":"Missing"}]);
    assert!(prepare_save(&current, &invalid).is_err());
    let mut changed_helper = modified.clone();
    changed_helper["files"][1]["content"] = json!("unexpected");
    assert!(prepare_save(&current, &changed_helper).is_err());
    let mut renamed = modified.clone();
    renamed["diagrams"] =
        json!([{"name":"Renamed","variable_name":"renamed","manifests":[{"name":"After"}]}]);
    save_document(&pool, &org, &current, &prepare_save(&current, &renamed)?).await?;
    assert!(read_document(&pool, "org_a", "jobs_diagram").await.is_err());
    assert!(read_document(&pool, "org_a", "renamed").await.is_ok());
    Ok(())
}
