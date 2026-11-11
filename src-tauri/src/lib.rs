//! Mirror Download 桌面版 Rust 后端入口。

mod cli;
mod commands;
mod models;
mod parser;
mod settings;

use commands::{CliLock, DownloadRegistry};
use tokio::sync::Mutex as AsyncMutex;

/// 供集成测试访问内部实现的薄桥接层（不对外发布语义）。
pub mod testing {
    pub use crate::cli::{resolve_cli, run_capture, CliInvocation as TestInvocation};
    pub use crate::models::RepoInfo as TestRepoInfo;
    pub use crate::parser::{parse_find_json, parse_probe, PullParser};

    pub fn resolve_cli_for_test() -> Option<TestInvocation> {
        resolve_cli()
    }

    pub async fn run_capture_for_test(
        inv: &TestInvocation,
        args: &[&str],
    ) -> Result<String, String> {
        run_capture(inv, args).await
    }

    pub fn parse_find_json_for_test(raw: &str) -> Result<TestRepoInfo, String> {
        parse_find_json(raw)
    }

    pub fn parse_probe_for_test(
        raw: &str,
    ) -> Result<crate::models::ProbeReport, String> {
        parse_probe(raw)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DownloadRegistry::default())
        .manage(CliLock(AsyncMutex::new(())))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings_cmd,
            commands::get_config_dir,
            commands::check_cli_available,
            commands::probe_sources,
            commands::find_repo,
            commands::start_download,
            commands::cancel_download,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
