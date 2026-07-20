use serde::Serialize;

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
