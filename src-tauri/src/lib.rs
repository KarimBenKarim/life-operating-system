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
    fn inspect_sqlcipher_details() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();

        // Test keying with raw hex key
        let hex_key = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let pragma_key = format!("PRAGMA key = \"x'{}'\";", hex_key);
        conn.execute_batch(&pragma_key).unwrap();

        // Check if DB is functional with this key
        conn.execute("CREATE TABLE t (x TEXT);", []).unwrap();
        conn.execute("INSERT INTO t VALUES ('hello');", []).unwrap();

        let val: String = conn.query_row("SELECT x FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(val, "hello");

        // Inspect cipher pragmas
        let cipher: String = conn.query_row("PRAGMA cipher", [], |r| r.get(0)).unwrap_or_else(|_| "unknown".into());
        let page_size: i32 = conn.query_row("PRAGMA page_size", [], |r| r.get(0)).unwrap();
        let kdf_iter: i32 = conn.query_row("PRAGMA kdf_iter", [], |r| r.get(0)).unwrap_or(-1);

        println!("Cipher: {}", cipher);
        println!("Page size: {}", page_size);
        println!("KDF iter: {}", kdf_iter);
    }
}
