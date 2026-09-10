use tauri::Manager;
use zeroize::Zeroizing;

pub mod db;
pub mod vault;

pub fn ping_logic() -> String {
    "PONG".to_string()
}

#[tauri::command]
fn ping() -> String {
    ping_logic()
}

/// Initialize the local encrypted SQLite database, execute pending migrations,
/// and run mandatory startup verification on the audit log hash chain.
///
/// Fails closed if no passcode provider / environment variable is available.
pub fn init_app_database<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;

    let db_name = std::env::var("LIFEOS_DB_NAME").unwrap_or_else(|_| "lifeos.db".to_string());
    let db_path = app_data_dir.join(db_name);

    // Passcode source boundary: Passcode is retrieved from environment variable
    // and wrapped immediately in Zeroizing<String> so heap memory is zeroized on drop.
    // Fails closed if no passcode is provided.
    // Note: Acquisition of passcodes via host OS keyring integration or unlock UI is deferred as a scoped follow-up.
    let passcode = Zeroizing::new(
        std::env::var("LIFEOS_DB_PASSCODE").map_err(|_| db::DatabaseError::InvalidPasscode)?,
    );

    let _conn = db::initialize_and_verify_database(&db_path, passcode.as_str())?;

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
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_ping_command() {
        let result = ping_logic();
        assert_eq!(result, "PONG");
    }

    #[test]
    fn test_tauri_mock_app_bootstrap_invokes_database_verification() {
        let _guard = TEST_ENV_MUTEX.lock().unwrap();
        let app = tauri::test::mock_app();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::set_var("LIFEOS_DB_NAME", format!("mock_app_{nanos}.db"));
        std::env::set_var("LIFEOS_DB_PASSCODE", "TestMockPasscode123!");

        let result = init_app_database(app.handle());

        std::env::remove_var("LIFEOS_DB_PASSCODE");
        std::env::remove_var("LIFEOS_DB_NAME");

        assert!(
            result.is_ok(),
            "Application bootstrap MUST execute database opening, migration, and audit chain verification"
        );
    }

    #[test]
    fn test_startup_fails_closed_when_passcode_missing() {
        let _guard = TEST_ENV_MUTEX.lock().unwrap();
        let app = tauri::test::mock_app();
        std::env::remove_var("LIFEOS_DB_PASSCODE");
        let result = init_app_database(app.handle());
        assert!(
            result.is_err(),
            "Application startup MUST fail closed when no passcode provider or environment variable is available"
        );
    }
}
