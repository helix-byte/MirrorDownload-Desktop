//! Tauri 命令层：暴露给前端的全部 IPC 接口。

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex as AsyncMutex;

use crate::cli;
use crate::models::*;
use crate::parser::{self, PullParser};
use crate::settings;

/// 运行中的下载任务登记表：task_id -> 取消标志
pub struct DownloadRegistry {
    cancels: Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>,
}

impl Default for DownloadRegistry {
    fn default() -> Self {
        DownloadRegistry {
            cancels: Mutex::new(HashMap::new()),
        }
    }
}

/// 全局异步锁，避免并发 probe/find 过多挤占网络
pub struct CliLock(pub AsyncMutex<()>);

// ---------------- 设置相关 ----------------

#[tauri::command]
pub fn get_settings() -> Settings {
    settings::load_settings()
}

#[tauri::command]
pub fn save_settings_cmd(settings: Settings) -> Result<(), String> {
    settings::save_settings(&settings)
}

#[tauri::command]
pub fn get_config_dir() -> String {
    settings::config_dir_display()
}

// ---------------- CLI 检测 ----------------

#[tauri::command]
pub async fn check_cli_available() -> CliStatus {
    cli::check_cli().await
}

// ---------------- probe ----------------

#[tauri::command]
pub async fn probe_sources(
    source: Option<String>,
    proxy: Option<String>,
    timeout_ms: Option<u64>,
    _lock: State<'_, CliLock>,
) -> Result<ProbeReport, String> {
    let _guard = _lock.0.lock().await;
    let inv = cli::resolve_cli().ok_or_else(|| "未找到 mirrordl CLI".to_string())?;

    let timeout = timeout_ms.unwrap_or(5000).to_string();
    let mut args: Vec<String> = vec!["probe".to_string(), "--timeout".to_string(), timeout];
    if let Some(s) = source {
        if !s.is_empty() {
            args.push("-s".into());
            args.push(s);
        }
    }
    if let Some(px) = proxy {
        if !px.is_empty() {
            args.push("--proxy".into());
            args.push(px);
        }
    }
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let raw = cli::run_capture(&inv, &arg_refs).await?;
    parser::parse_probe(&raw)
}

// ---------------- find ----------------

#[tauri::command]
pub async fn find_repo(
    repo_id: String,
    repo_type: Option<String>,
    revision: Option<String>,
    source: Option<String>,
    hf_token: Option<String>,
    modelscope_token: Option<String>,
    token: Option<String>,
    proxy: Option<String>,
    _lock: State<'_, CliLock>,
) -> Result<RepoInfo, String> {
    let _guard = _lock.0.lock().await;
    let inv = cli::resolve_cli().ok_or_else(|| "未找到 mirrordl CLI".to_string())?;

    let mut args: Vec<String> = vec!["find".into(), repo_id, "--json".into()];
    if let Some(rt) = repo_type.filter(|s| !s.is_empty()) {
        args.push("--repo-type".into());
        args.push(rt);
    }
    if let Some(rev) = revision.filter(|s| !s.is_empty()) {
        args.push("--revision".into());
        args.push(rev);
    }
    if let Some(s) = source.filter(|s| !s.is_empty()) {
        args.push("-s".into());
        args.push(s);
    }
    if let Some(t) = hf_token.filter(|s| !s.is_empty()) {
        args.push("--hf-token".into());
        args.push(t);
    }
    if let Some(t) = modelscope_token.filter(|s| !s.is_empty()) {
        args.push("--modelscope-token".into());
        args.push(t);
    }
    if let Some(t) = token.filter(|s| !s.is_empty()) {
        args.push("--token".into());
        args.push(t);
    }
    if let Some(px) = proxy.filter(|s| !s.is_empty()) {
        args.push("--proxy".into());
        args.push(px);
    }
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let raw = cli::run_capture(&inv, &arg_refs).await?;
    parser::parse_find_json(&raw)
}

// ---------------- 下载 ----------------

/// 下载参数（由前端组装）
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub task_id: String,
    pub repo_id: String,
    pub repo_type: Option<String>,
    pub revision: Option<String>,
    pub source: Option<String>,
    pub threads: Option<u32>,
    pub output: Option<String>,
    pub files: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub quant: Option<String>,
    /// target_mode: none / comfyui / lmstudio
    pub target_mode: String,
    pub comfyui_root: String,
    pub lmstudio_root: String,
    pub hf_token: String,
    pub modelscope_token: String,
    pub token: String,
    pub proxy: String,
}

fn build_download_args(req: &DownloadRequest) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = vec!["pull".into(), req.repo_id.clone()];
    if let Some(rt) = req.repo_type.as_ref().filter(|s| !s.is_empty()) {
        args.push("--repo-type".into());
        args.push(rt.clone());
    }
    if let Some(rev) = req.revision.as_ref().filter(|s| !s.is_empty()) {
        args.push("--revision".into());
        args.push(rev.clone());
    }
    if let Some(s) = req.source.as_ref().filter(|s| !s.is_empty()) {
        args.push("-s".into());
        args.push(s.clone());
    }
    if let Some(t) = req.threads {
        args.push("-t".into());
        args.push(t.to_string());
    }
    if let Some(q) = req.quant.as_ref().filter(|s| !s.is_empty()) {
        args.push("-q".into());
        args.push(q.clone());
    }
    for f in &req.files {
        if !f.is_empty() {
            args.push("-f".into());
            args.push(f.clone());
        }
    }
    for inc in &req.include {
        if !inc.is_empty() {
            args.push("--include".into());
            args.push(inc.clone());
        }
    }
    for exc in &req.exclude {
        if !exc.is_empty() {
            args.push("--exclude".into());
            args.push(exc.clone());
        }
    }

    // 目标模式
    match req.target_mode.as_str() {
        "comfyui" => {
            if req.comfyui_root.is_empty() {
                return Err("未设置 ComfyUI 根目录".into());
            }
            args.push("--comfyui".into());
            args.push(req.comfyui_root.clone());
        }
        "lmstudio" => {
            if req.lmstudio_root.is_empty() {
                args.push("--lmstudio".into());
            } else {
                args.push("--lmstudio".into());
                args.push(req.lmstudio_root.clone());
            }
        }
        _ => {
            if let Some(out) = req.output.as_ref().filter(|s| !s.is_empty()) {
                args.push("-o".into());
                args.push(out.clone());
            }
        }
    }

    if !req.hf_token.is_empty() {
        args.push("--hf-token".into());
        args.push(req.hf_token.clone());
    }
    if !req.modelscope_token.is_empty() {
        args.push("--modelscope-token".into());
        args.push(req.modelscope_token.clone());
    }
    if !req.token.is_empty() {
        args.push("--token".into());
        args.push(req.token.clone());
    }
    if !req.proxy.is_empty() {
        args.push("--proxy".into());
        args.push(req.proxy.clone());
    }

    // 避开 Windows 符号链接权限问题
    args.push("--no-symlinks".into());

    Ok(args)
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    registry: State<'_, DownloadRegistry>,
    req: DownloadRequest,
) -> Result<(), String> {
    let inv = cli::resolve_cli().ok_or_else(|| "未找到 mirrordl CLI".to_string())?;
    let args = build_download_args(&req)?;
    let task_id = req.task_id.clone();

    // 注册取消通道
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    {
        let mut map = registry.cancels.lock().unwrap();
        map.insert(task_id.clone(), cancel_tx);
    }

    let app_handle = app.clone();
    let tid = task_id.clone();

    // 立即发送 queued -> running
    let queued = ProgressEvent {
        task_id: tid.clone(),
        status: TaskStatus::Running,
        file_index: 0,
        file_total: 0,
        current_file: String::new(),
        file_percent: 0.0,
        speed: String::new(),
        downloaded: String::new(),
        total: String::new(),
        line: "开始下载…".to_string(),
    };
    let _ = app.emit("download-progress", &queued);

    tauri::async_runtime::spawn(async move {
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut cmd = inv.command();
        cmd.args(&arg_refs);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let parser = PullParser::new(tid.clone());
                let ev = parser.terminal(TaskStatus::Failed, &format!("启动失败：{e}"));
                let _ = app_handle.emit("download-progress", &ev);
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // stdout 与 stderr 各用独立解析器，逐行解析并推送进度事件
        let app_out = app_handle.clone();
        let tid_out = tid.clone();
        let stdout_task = tokio::spawn(async move {
            let mut parser = PullParser::new(tid_out);
            if let Some(out) = stdout {
                let mut lines = BufReader::new(out).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(ev) = parser.feed(&line) {
                        let _ = app_out.emit("download-progress", &ev);
                    }
                }
            }
        });

        let app_err = app_handle.clone();
        let tid_err = tid.clone();
        let stderr_task = tokio::spawn(async move {
            let mut parser = PullParser::new(tid_err);
            if let Some(err) = stderr {
                let mut lines = BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(ev) = parser.feed(&line) {
                        let _ = app_err.emit("download-progress", &ev);
                    }
                }
            }
        });

        // 等待进程结束或被取消
        let status = tokio::select! {
            st = child.wait() => st.ok(),
            _ = cancel_rx => {
                let _ = child.kill().await;
                None
            }
        };

        let _ = stdout_task.await;
        let _ = stderr_task.await;

        let cancelled = status.is_none();
        let ok = status.map(|s| s.success()).unwrap_or(false);
        let final_status = if cancelled {
            TaskStatus::Cancelled
        } else if ok {
            TaskStatus::Completed
        } else {
            TaskStatus::Failed
        };
        let line = match final_status {
            TaskStatus::Completed => "下载完成".to_string(),
            TaskStatus::Cancelled => "已取消".to_string(),
            _ => "下载失败".to_string(),
        };
        let ev = ProgressEvent {
            task_id: tid.clone(),
            status: final_status,
            file_index: 0,
            file_total: 0,
            current_file: String::new(),
            file_percent: 100.0,
            speed: String::new(),
            downloaded: String::new(),
            total: String::new(),
            line,
        };
        let _ = app_handle.emit("download-progress", &ev);
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_download(registry: State<'_, DownloadRegistry>, task_id: String) -> Result<(), String> {
    let mut map = registry.cancels.lock().unwrap();
    if let Some(tx) = map.remove(&task_id) {
        let _ = tx.send(());
        Ok(())
    } else {
        Err("任务不存在或已结束".to_string())
    }
}