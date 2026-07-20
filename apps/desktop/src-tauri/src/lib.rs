use serde::Serialize;
mod github;
mod storage;

mod commands {
    use super::VaultStatus;

    #[tauri::command]
    pub fn get_vault_status() -> VaultStatus {
        VaultStatus {
            selected: false,
            path: None,
            document_count: 0,
        }
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
        .invoke_handler(tauri::generate_handler![commands::get_vault_status])
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
