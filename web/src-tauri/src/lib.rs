use orgwise::cli::environment::CliBackend;
use orgwise::command::OrgwiseCommand;
use serde_json::Value;
use std::path::PathBuf;

#[tauri::command]
async fn execute_command(command: OrgwiseCommand) -> Result<Value, String> {
    let backend = CliBackend::new(false);

    backend.load_org_file(&PathBuf::from("/Users/poi/org/todo.org"));

    command
        .execute(&backend)
        .await
        .map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![execute_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
