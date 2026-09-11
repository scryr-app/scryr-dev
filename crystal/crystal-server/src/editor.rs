//! Manifest editor operations and explicitly registered local workspaces.
use crystal_core::{
    generated_manifest_envelope::ManifestSourceFile,
    manifest::ManifestRequestContext,
    persistence::{
        DatabasePool,
        documents::{self, ManifestDocument},
    },
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

/// Native validation callback supplied by scryr serve, never by a browser.
pub type ValidateLocal =
    Arc<dyn Fn(Vec<ManifestSourceFile>, String) -> Result<serde_json::Value, String> + Send + Sync>;

/// Local disk access is limited to the entrypoint explicitly selected by scryr serve.
#[derive(Clone)]
pub struct LocalWorkspace {
    file: PathBuf,
    folder: String,
    /// Serializes the watch pipeline with browser saves.
    pub gate: Arc<Mutex<()>>,
    validate: ValidateLocal,
}
impl LocalWorkspace {
    /// Resolve and register one entrypoint.
    ///
    /// # Errors
    /// Returns an error if the entrypoint is unavailable or outside the project.
    pub fn new(root: &Path, file: &Path, validate: ValidateLocal) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let file = file.canonicalize().map_err(|e| e.to_string())?;
        let relative = file.strip_prefix(&root).map_err(|e| e.to_string())?;
        let folder = relative
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
            .replace('\\', "/");
        Ok(Self {
            file,
            folder,
            gate: Arc::new(Mutex::new(())),
            validate,
        })
    }
    fn matches(&self, doc: &ManifestDocument) -> bool {
        doc.folder_path == self.folder
            && self.file.file_name().and_then(|s| s.to_str()) == Some(doc.entrypoint.as_str())
    }
    fn read_files(&self) -> Result<Vec<ManifestSourceFile>, String> {
        let root = self.file.parent().ok_or("Missing source root")?;
        let mut files = Vec::new();
        collect_files(root, root, &mut files)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
    fn write(&self, content: &str) -> Result<(), String> {
        // Re-check the exact registered path; do not follow a newly introduced symlink.
        if self.file.canonicalize().map_err(|e| e.to_string())? != self.file {
            return Err("The registered source path changed".into());
        }
        let temp = self
            .file
            .with_file_name(format!(".scryr-edit-{}", uuid::Uuid::new_v4()));
        let result = (|| {
            use std::io::Write;
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .map_err(|e| e.to_string())?;
            std::fs::set_permissions(
                &temp,
                std::fs::metadata(&self.file)
                    .map_err(|e| e.to_string())?
                    .permissions(),
            )
            .map_err(|e| e.to_string())?;
            output
                .write_all(content.as_bytes())
                .map_err(|e| e.to_string())?;
            output.sync_all().map_err(|e| e.to_string())?;
            std::fs::rename(&temp, &self.file).map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temp);
        }
        result
    }
}
fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<ManifestSourceFile>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if name.starts_with('.')
            || matches!(
                name.as_str(),
                "__pycache__" | "node_modules" | "target" | "dist" | "uv-cache"
            )
        {
            continue;
        }
        if kind.is_dir() {
            collect_files(root, &path, files)?;
        } else if kind.is_file() && path.extension().is_some_and(|e| e == "scry" || e == "py") {
            files.push(ManifestSourceFile {
                path: path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/"),
                content: std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
            });
        }
    }
    Ok(())
}

/// Request marker set by the HTTP handler after checking the browser's origin.
pub(crate) struct EditorRequestAllowed(pub bool);

/// Editing service shared by all workers of a server.
#[derive(Clone, Default)]
pub(crate) struct EditorService {
    pub local: Option<LocalWorkspace>,
}
impl EditorService {
    pub(crate) async fn read(
        &self,
        pool: &DatabasePool,
        context: &ManifestRequestContext,
        identifier: &str,
    ) -> Result<ManifestDocument, String> {
        let _guard = if let Some(local) = &self.local {
            Some(local.gate.lock().await)
        } else {
            None
        };
        self.read_unlocked(pool, context, identifier).await
    }
    async fn read_unlocked(
        &self,
        pool: &DatabasePool,
        context: &ManifestRequestContext,
        identifier: &str,
    ) -> Result<ManifestDocument, String> {
        let mut doc = documents::read_document(pool, &context.clerk_org_id, identifier).await?;
        doc.writable = context.can_write_generated_manifests();
        if let Some(local) = &self.local {
            doc.local = local.matches(&doc);
            // Historical diagrams can be viewed, but cannot write another project.
            doc.writable &= doc.local;
            if doc.local {
                doc.files = local.read_files()?;
                doc.revision = documents::revision(&(doc.revision, &doc.files));
            }
        }
        Ok(doc)
    }
    pub(crate) async fn save(
        &self,
        pool: &DatabasePool,
        context: &ManifestRequestContext,
        identifier: &str,
        revision: &str,
        envelope: serde_json::Value,
    ) -> Result<ManifestDocument, String> {
        let _guard = if let Some(local) = &self.local {
            Some(local.gate.lock().await)
        } else {
            None
        };
        let doc = self.read_unlocked(pool, context, identifier).await?;
        if !doc.writable {
            return Err("This source is not writable in the current session".into());
        }
        if doc.revision != revision {
            return Err("Source changed since it was loaded. Reload before saving.".into());
        }
        let mut inputs = documents::prepare_save(&doc, &envelope)?;
        let stored = documents::read_document(pool, &context.clerk_org_id, identifier).await?;
        let stored_revision = if doc.local {
            documents::revision(&(&stored.revision, &doc.files))
        } else {
            stored.revision.clone()
        };
        if stored_revision != revision {
            return Err("Source changed while preparing the save. Reload before saving.".into());
        }
        let mut restore = None;
        if let Some(local) = &self.local {
            let previous = doc
                .files
                .iter()
                .find(|f| f.path == doc.entrypoint)
                .ok_or("Missing entrypoint")?
                .content
                .clone();
            let files =
                serde_json::from_value(envelope["files"].clone()).map_err(|e| e.to_string())?;
            let entrypoint = doc.entrypoint.clone();
            let validate = local.validate.clone();
            let validated = tokio::task::spawn_blocking(move || validate(files, entrypoint))
                .await
                .map_err(|e| e.to_string())
                .and_then(|value| value)?;
            inputs = documents::prepare_save(&doc, &validated)?;
            // Validation runs against a temporary source tree, never the live file.
            // Re-check all imports as well as the entrypoint before committing.
            if documents::revision(&local.read_files()?) != documents::revision(&doc.files) {
                return Err("Source changed during validation. Reload before saving.".into());
            }
            let next = validated["files"]
                .as_array()
                .and_then(|files| files.iter().find(|f| f["path"] == doc.entrypoint))
                .and_then(|f| f["content"].as_str())
                .ok_or("Missing validated entrypoint source")?;
            local.write(next)?;
            restore = Some((previous, next.to_owned()));
        }
        if let Err(error) = documents::save_document(pool, context, &stored, &inputs).await {
            if let (Some(local), Some((previous, written))) = (&self.local, restore) {
                if std::fs::read_to_string(&local.file).map_err(|e| e.to_string())? != written {
                    return Err(format!(
                        "{error}; source changed externally and was left untouched"
                    ));
                }
                local
                    .write(&previous)
                    .map_err(|restore| format!("{error}; restoring source failed: {restore}"))?;
            }
            return Err(error);
        }
        let selected = inputs
            .iter()
            .find(|i| i.scry_identifier.as_deref() == Some(identifier))
            .or_else(|| inputs.first())
            .and_then(|i| i.scry_identifier.as_deref())
            .ok_or("Missing saved diagram")?;
        self.read_unlocked(pool, context, selected).await
    }
}
