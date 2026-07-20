use serde::Serialize;
mod github;
mod storage;

mod commands {
    use super::{github::GithubClient, storage, VaultStatus};
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
        let client = GithubClient::new(token).map_err(|error| format!("{error:?}"))?;
        let repositories = client
            .list_starred()
            .await
            .map_err(|error| format!("{error:?}"))?;
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
            commands::import_github
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
        assert!(root.join("sources/github/octo--hello.md").exists());
        let _ = std::fs::remove_dir_all(root);
    }
}
