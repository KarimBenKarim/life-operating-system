pub fn ping_logic() -> String {
    "PONG".to_string()
}

#[tauri::command]
fn ping() -> String {
    ping_logic()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
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
}
