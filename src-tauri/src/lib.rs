use tauri::Manager;

pub mod db;

pub fn ping_logic() -> String {
    "PONG".to_string()
}

#[tauri::command]
fn ping() -> String {
    ping_logic()
}

/// Initialize the local encrypted SQLite database, execute pending migrations,
/// and run mandatory startup verification on the audit log hash chain.
pub fn init_app_database<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;

    let db_path = app_data_dir.join("lifeos.db");

    // Passcode source boundary: Passcode is retrieved from environment variable or dev fallback.
    // Note: Acquisition of passcodes via host OS keyring integration or unlock UI is deferred as a scoped follow-up.
    let passcode = std::env::var("LIFEOS_DB_PASSCODE")
        .unwrap_or_else(|_| "LIFEOS_DEV_SECURE_PASSCODE_123!".to_string());

    let _conn = db::initialize_and_verify_database(&db_path, &passcode)?;

    Ok(())
}

pub fn build_tauri_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
        .setup(|app| {
            init_app_database(app.handle())?;
            Ok(())
        })
}

pub fn run() {
    build_tauri_app()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_command() {
        let result = ping_logic();
        assert_eq!(result, "PONG");
    }

    #[test]
    fn test_tauri_mock_app_bootstrap_invokes_database_verification() {
        let app = tauri::test::mock_app();
        let result = init_app_database(app.handle());
        assert!(
            result.is_ok(),
            "Application bootstrap MUST execute database opening, migration, and audit chain verification"
        );
    }
}
