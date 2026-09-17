//! Seeding contract shared by `SQLite` and the optional remote Turso test.
use crystal_core::{
    manifest::{ManifestRequestContext, UpsertGeneratedManifestInput},
    persistence::{
        DatabasePool, documents, ensure_table, list_generated_manifest_maps,
        persist_generated_manifest, seed_organization_samples,
    },
};
use serde_json::json;

pub(crate) fn context(org: &str) -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "member".into(),
        clerk_org_id: org.into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    }
}

pub(crate) fn samples() -> Result<Vec<UpsertGeneratedManifestInput>, String> {
    let mut inputs = Vec::new();
    for name in ["first", "second"] {
        let file = format!("{name}.scry");
        let envelope = json!({
            "files": [{"path":file, "content":"# editable sample"}],
            "manifests": [{"name":name, "variable_name":"service", "line_number":1}],
            "diagrams": [
                {"name":name, "variable_name":format!("{name}_diagram"), "manifests":[{"name":name}]},
                {"name":"Sibling", "variable_name":format!("{name}_sibling"), "manifests":[{"name":name}]}
            ]
        });
        let doc = documents::ManifestDocument {
            identifier: format!("{name}_diagram"),
            key: format!("samples/{name}"),
            folder_path: "samples".into(),
            entrypoint: file,
            files: serde_json::from_value(envelope["files"].clone()).map_err(|e| e.to_string())?,
            revision: String::new(),
            writable: true,
            local: false,
        };
        inputs.extend(documents::prepare_save(&doc, &envelope)?);
    }
    Ok(inputs)
}

async fn execute(pool: &DatabasePool, sql: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    match pool {
        DatabasePool::Sqlite(pool) => {
            sqlx::query(sql).execute(pool).await?;
        }
        DatabasePool::Turso(database) => {
            database.connect()?.execute(sql, ()).await?;
        }
    }
    Ok(())
}

pub(crate) async fn count(
    pool: &DatabasePool,
    table: &str,
) -> Result<i64, Box<dyn std::error::Error>> {
    let sql = match table {
        "generated_manifests" => "SELECT COUNT(*) FROM generated_manifests",
        "generated_manifest_uploads" => "SELECT COUNT(*) FROM generated_manifest_uploads",
        "organization_sample_seeds" => "SELECT COUNT(*) FROM organization_sample_seeds",
        _ => return Err("unknown sample test table".into()),
    };
    match pool {
        DatabasePool::Sqlite(pool) => Ok(sqlx::query_scalar(sql).fetch_one(pool).await?),
        DatabasePool::Turso(database) => Ok(database
            .connect()?
            .query(sql, ())
            .await?
            .next()
            .await?
            .ok_or("missing count")?
            .get(0)?),
    }
}

pub(crate) async fn exercise(pool: DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    let org = context("org_samples_a");
    let other = context("org_samples_b");
    let samples = samples()?;
    ensure_table(&pool).await?;

    // A failure after artifacts were inserted must undo the marker, artifacts
    // and ledger together. A subsequent request can retry successfully.
    execute(&pool, "CREATE TRIGGER fail_sample BEFORE INSERT ON generated_manifest_uploads WHEN NEW.scry_identifier = 'second_diagram' BEGIN SELECT RAISE(ABORT, 'seed failure'); END").await?;
    assert!(
        seed_organization_samples(&pool, &org, &samples)
            .await
            .is_err()
    );
    assert_eq!(count(&pool, "organization_sample_seeds").await?, 0);
    assert_eq!(count(&pool, "generated_manifests").await?, 0);
    assert_eq!(count(&pool, "generated_manifest_uploads").await?, 0);
    execute(&pool, "DROP TRIGGER fail_sample").await?;
    seed_organization_samples(&pool, &org, &samples).await?;
    seed_organization_samples(&pool, &org, &samples).await?;
    seed_organization_samples(&pool, &other, &samples).await?;
    assert_eq!(count(&pool, "organization_sample_seeds").await?, 2);
    assert_eq!(count(&pool, "generated_manifest_uploads").await?, 8);

    // Editing one sample updates its sibling diagrams but neither another
    // source in the same folder nor another organization's copy.
    let doc = documents::read_document(&pool, &org.clerk_org_id, "first_diagram").await?;
    let mut envelope: serde_json::Value = serde_json::from_str(&samples[0].content)?;
    let sibling: serde_json::Value = serde_json::from_str(&samples[1].content)?;
    envelope["diagrams"] = json!([envelope["diagrams"][0], sibling["diagrams"][0]]);
    envelope["files"][0]["content"] = json!("# changed by this organization");
    let inputs = documents::prepare_save(&doc, &envelope)?;
    documents::save_document(&pool, &org, &doc, &inputs).await?;
    seed_organization_samples(&pool, &org, &samples).await?;
    let changed = documents::read_document(&pool, &org.clerk_org_id, "first_diagram").await?;
    assert_eq!(changed.files[0].content, "# changed by this organization");
    assert_eq!(
        documents::read_document(&pool, &org.clerk_org_id, "first_sibling")
            .await?
            .files[0]
            .content,
        "# changed by this organization"
    );
    assert_eq!(
        list_generated_manifest_maps(&pool, &org.clerk_org_id)
            .await?
            .len(),
        4
    );
    assert_eq!(
        documents::read_document(&pool, &other.clerk_org_id, "first_diagram")
            .await?
            .files[0]
            .content,
        "# editable sample"
    );
    assert_eq!(
        documents::read_document(&pool, &org.clerk_org_id, "second_diagram")
            .await?
            .files[0]
            .content,
        "# editable sample"
    );

    // Pre-existing identifiers preserve the whole source rather than seeding
    // siblings whose editor would later conflict with that existing source.
    let existing_org = context("org_samples_existing");
    let mut existing = samples[0].clone();
    existing.folder_path = Some("my-project".into());
    persist_generated_manifest(&pool, &existing, &existing_org).await?;
    seed_organization_samples(&pool, &existing_org, &samples).await?;
    let maps = list_generated_manifest_maps(&pool, &existing_org.clerk_org_id).await?;
    assert_eq!(maps.len(), 3);
    assert!(maps.iter().any(|m| m.folder_path == "my-project"));
    assert!(!maps.iter().any(|m| m.scry_identifier == "first_sibling"));

    // Deleting copies does not reset the durable seed marker.
    execute(
        &pool,
        "DELETE FROM generated_manifest_uploads WHERE clerk_org_id = 'org_samples_a'",
    )
    .await?;
    execute(
        &pool,
        "DELETE FROM generated_manifests WHERE clerk_org_id = 'org_samples_a'",
    )
    .await?;
    seed_organization_samples(&pool, &org, &samples).await?;
    assert!(
        list_generated_manifest_maps(&pool, &org.clerk_org_id)
            .await?
            .is_empty()
    );
    Ok(())
}
