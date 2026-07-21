use serde::Serialize;
mod github;
mod linkedin;
mod rag;
mod storage;

mod commands {
    use super::{github::GithubClient, linkedin, rag, storage, VaultStatus};
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ImportSummary {
        pub created: u64,
        pub updated: u64,
        pub unchanged: u64,
        pub failed: u64,
    }

    #[tauri::command]
    pub fn get_vault_status() -> VaultStatus {
        VaultStatus {
            selected: false,
            path: None,
            document_count: 0,
        }
    }

    #[tauri::command]
    pub async fn import_github(vault_path: String, token: String) -> Result<ImportSummary, String> {
        if token.trim().is_empty() {
            return Err("GitHub token is required".into());
        }
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let client = GithubClient::new(token).map_err(|error| error.to_string())?;
        let repositories = client
            .list_starred()
            .await
            .map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for repository in repositories {
            let readme = match client
                .read_readme(&repository.owner.login, &repository.name)
                .await
            {
                Ok(value) => value.unwrap_or_else(|| "README unavailable.\n".into()),
                Err(_) => {
                    summary.failed += 1;
                    continue;
                }
            };
            let content = format!("---\nid: github:{}\ntitle: {}\nsource_kind: github\nsource_uri: {}\n---\n\n# {}\n\n{}\n\n## Repository\n\n{}\n",
                repository.full_name, repository.name, repository.html_url, repository.name,
                repository.description.as_deref().unwrap_or("No description provided."), readme);
            let document = storage::SourceDocument {
                id: format!("github:{}", repository.full_name),
                relative_path: format!(
                    "sources/github/{}--{}.md",
                    repository.owner.login, repository.name
                ),
                title: repository.name,
                source_kind: "github".into(),
                source_uri: Some(repository.html_url),
                content,
                captured_at: chrono::Utc::now().to_rfc3339(),
            };
            match storage::upsert_document(&mut connection, &root, &document)
                .map_err(|error| error.to_string())?
            {
                storage::UpsertResult::Created => summary.created += 1,
                storage::UpsertResult::Updated => summary.updated += 1,
                storage::UpsertResult::Unchanged => summary.unchanged += 1,
            }
        }
        Ok(summary)
    }

    #[tauri::command]
    pub fn import_linkedin_html(
        vault_path: String,
        html_path: String,
    ) -> Result<ImportSummary, String> {
        let html = std::fs::read_to_string(&html_path).map_err(|error| error.to_string())?;
        let posts = linkedin::parse_activity_html(&html);
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let id = post.url.rsplit(':').next().unwrap_or(&post.url).to_string();
            let content = format!("---\nid: linkedin:{id}\ntitle: LinkedIn post {id}\nsource_kind: linkedin\nsource_uri: {}\n---\n\n{}\n", post.url, post.text);
            let document = storage::SourceDocument {
                id: format!("linkedin:{id}"),
                relative_path: format!("sources/linkedin/{id}.md"),
                title: format!("LinkedIn post {id}"),
                source_kind: "linkedin".into(),
                source_uri: Some(post.url),
                content,
                captured_at: chrono::Utc::now().to_rfc3339(),
            };
            match storage::upsert_document(&mut connection, &root, &document)
                .map_err(|error| error.to_string())?
            {
                storage::UpsertResult::Created => summary.created += 1,
                storage::UpsertResult::Updated => summary.updated += 1,
                storage::UpsertResult::Unchanged => summary.unchanged += 1,
            }
        }
        Ok(summary)
    }

    #[tauri::command]
    pub fn search_documents(
        vault_path: String,
        query: String,
        limit: Option<u32>,
    ) -> Result<Vec<storage::SearchResult>, String> {
        let paths = storage::initialize(std::path::Path::new(&vault_path))
            .map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        storage::search(&connection, &query, limit.unwrap_or(20).min(100))
            .map_err(|error| error.to_string())
    }

    #[tauri::command]
    pub fn export_obsidian(vault_path: String, destination: String) -> Result<u64, String> {
        storage::export_markdown(
            std::path::Path::new(&vault_path),
            std::path::Path::new(&destination),
        )
        .map_err(|error| error.to_string())
    }

    #[tauri::command]
    pub fn retrieve_context(
        vault_path: String,
        query: String,
        limit: Option<u32>,
    ) -> Result<rag::RetrievalContext, String> {
        let paths = storage::initialize(std::path::Path::new(&vault_path))
            .map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let results = storage::search(&connection, &query, limit.unwrap_or(8).min(50))
            .map_err(|error| error.to_string())?;
        Ok(rag::build_context(&query, results))
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub selected: bool,
    pub path: Option<String>,
    pub document_count: u64,
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_vault_status,
            commands::import_github,
            commands::import_linkedin_html,
            commands::search_documents,
            commands::export_obsidian,
            commands::retrieve_context
        ])
        .run(tauri::generate_context!())
        .expect("error while running ResearchLedger");
}

#[cfg(test)]
mod tests {
    use super::storage::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "researchledger-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn initializes_and_upserts_idempotently() {
        let root = temp_root();
        let paths = initialize(&root).unwrap();
        let mut db = open(&paths).unwrap();
        let document = SourceDocument {
            id: "github:octo/hello".into(),
            relative_path: "sources/github/octo--hello.md".into(),
            title: "hello".into(),
            source_kind: "github".into(),
            source_uri: Some("https://github.com/octo/hello".into()),
            content: "# hello\n".into(),
            captured_at: "2026-07-20T00:00:00Z".into(),
        };
        assert_eq!(
            upsert_document(&mut db, &root, &document).unwrap(),
            UpsertResult::Created
        );
        assert_eq!(
            upsert_document(&mut db, &root, &document).unwrap(),
            UpsertResult::Unchanged
        );
        assert_eq!(document_count(&db).unwrap(), 1);
        let results = search(&db, "hello", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "github:octo/hello");
        assert!(root.join("sources/github/octo--hello.md").exists());
        let export = temp_root();
        assert_eq!(export_markdown(&root, &export).unwrap(), 1);
        assert!(export.join("sources/github/octo--hello.md").exists());
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(export);
    }

    #[test]
    fn markdown_writer_rejects_paths_outside_vault() {
        let root = temp_root();
        std::fs::create_dir_all(&root).unwrap();
        let outside = root.parent().unwrap().join("outside.md");
        let _ = std::fs::remove_file(&outside);
        let result = write_markdown_atomic(&root, "../outside.md", "blocked");
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert!(!outside.exists());
        let _ = std::fs::remove_dir_all(root);
    }
}
