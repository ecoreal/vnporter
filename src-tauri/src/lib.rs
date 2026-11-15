// 五阶段移植流水线的纯逻辑核心（workspace 成员 ../core）：
// detect → unpack → adapt → repack → sign。
// Tauri 命令层将在后续里程碑中消费它（MVP 仅为骨架 + 真实 detect）。
pub use vnporter_core;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn greet_mentions_name() {
        assert!(greet("vnporter").contains("vnporter"));
    }
}
