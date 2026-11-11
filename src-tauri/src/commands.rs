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

/// 合并后的累积进度（进程结束时用于构造终态事件）
struct MergedProgress {
    file_index: u32,
    file_total: u32,
    current_file: String,
    file_percent: f64,
    downloaded: String,
    total: String,
}

/// 下载完成度判定结果
enum Verdict {
    Completed,
    Cancelled,
    /// exit 0 但文件缺失/为空——实际没下完
    Incomplete,
    Failed,
}

/// 落盘校验上下文：目标根目录 + 期望文件清单
struct VerifyCtx {
    /// 可确定的落地根目录；None 表示无法定位（回退 parser 完整性判定）
    root: Option<std::path::PathBuf>,
    /// 期望存在的文件（相对仓库的子路径）
    files: Vec<String>,
}

impl VerifyCtx {
    fn from_request(req: &DownloadRequest) -> Self {
        let files: Vec<String> = req
            .files
            .iter()
            .filter(|f| !f.is_empty())
            .cloned()
            .collect();
        // 仅在「普通输出目录」模式下可确定根目录；comfyui/lmstudio 由 CLI 自行决定子目录布局，难以定位
        let root = if req.target_mode == "none" || req.target_mode.is_empty() {
            req.output
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(std::path::PathBuf::from)
        } else {
            None
        };
        VerifyCtx { root, files }
    }

    /// 检查所有期望文件是否已在磁盘上存在且大小 > 0。
    /// 返回 Some(true)=全部落盘, Some(false)=有缺失/空文件, None=无法校验（无根目录或无文件清单）
    fn verify(&self) -> Option<bool> {
        let root = self.root.as_ref()?;
        if self.files.is_empty() {
            return None;
        }
        // CLI 通常把仓库文件放到 <root>/<repo 名>/... 下，这里对「根 + 可选一层子目录」做宽松匹配
        for f in &self.files {
            let fname = f.rsplit('/').next().unwrap_or(f);
            let direct = root.join(f);
            let basename_here = root.join(fname);
            let ok = file_nonempty(&direct) || file_nonempty(&basename_here) || find_under(root, fname);
            if !ok {
                return Some(false);
            }
        }
        Some(true)
    }
}

fn file_nonempty(p: &std::path::Path) -> bool {
    std::fs::metadata(p).map(|m| m.is_file() && m.len() > 0).unwrap_or(false)
}

/// 在 root 下（最多两层）递归查找指定文件名是否存在且非空
fn find_under(root: &std::path::Path, fname: &str) -> bool {
    fn walk(dir: &std::path::Path, fname: &str, depth: u8) -> bool {
        let rd = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return false,
        };
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_file() {
                if p.file_name().and_then(|n| n.to_str()) == Some(fname)
                    && std::fs::metadata(&p).map(|m| m.len() > 0).unwrap_or(false)
                {
                    return true;
                }
            } else if p.is_dir() && depth < 2 {
                if walk(&p, fname, depth + 1) {
                    return true;
                }
            }
        }
        false
    }
    walk(root, fname, 0)
}

/// 合并 stdout / stderr 两个解析器的快照，取信息更完整的一方
fn merge_parsers(out: Option<&PullParser>, err: Option<&PullParser>) -> MergedProgress {
    // score：优先看 file_total，再看 file_index，最后看是否有字节文本
    let score = |p: &PullParser| {
        let (fi, ft, _cf, _fp, _sp, dl, _tot) = p.snapshot();
        (ft as u64) * 1_000_000 + (fi as u64) * 1_000 + if dl.is_empty() { 0 } else { 1 }
    };
    let chosen = match (out, err) {
        (Some(o), Some(e)) => {
            if score(e) > score(o) {
                e
            } else {
                o
            }
        }
        (Some(o), None) => o,
        (None, Some(e)) => e,
        (None, None) => {
            // 都没有：返回空快照
            return MergedProgress {
                file_index: 0,
                file_total: 0,
                current_file: String::new(),
                file_percent: 0.0,
                downloaded: String::new(),
                total: String::new(),
            };
        }
    };
    let (file_index, file_total, current_file, file_percent, _speed, downloaded, total) =
        chosen.snapshot();
    MergedProgress {
        file_index,
        file_total,
        current_file,
        file_percent,
        downloaded,
        total,
    }
}

/// 判定下载是否真正完成
fn evaluate_completion(exit_ok: bool, merged: &MergedProgress, ctx: &VerifyCtx) -> Verdict {
    if !exit_ok {
        return Verdict::Failed;
    }
    // 1) 优先：文件落盘校验（最可信）
    match ctx.verify() {
        Some(true) => return Verdict::Completed,
        Some(false) => return Verdict::Incomplete,
        None => {}
    }
    // 2) 回退：parser 是否报告全部文件完成
    if merged.file_total > 0 {
        if merged.file_index >= merged.file_total {
            return Verdict::Completed;
        }
        return Verdict::Incomplete;
    }
    // 3) 完全无法得知（无字节/无文件数）：尊重 exit code，但属于尽力而为
    Verdict::Completed
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

    // 落盘校验所需信息（在移入异步任务前取出）
    let verify_ctx = VerifyCtx::from_request(&req);

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
        // stdout 解析器在结束时会返回自身，以便合并累积进度
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
            parser
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
            parser
        });

        // 等待进程结束或被取消
        let status = tokio::select! {
            st = child.wait() => st.ok(),
            _ = cancel_rx => {
                let _ = child.kill().await;
                None
            }
        };

        let out_parser = stdout_task.await.ok();
        let err_parser = stderr_task.await.ok();

        // 合并两个解析器的累积进度：取信息更完整的一方
        let merged = merge_parsers(out_parser.as_ref(), err_parser.as_ref());

        let cancelled = status.is_none();
        let exit_ok = status.map(|s| s.success()).unwrap_or(false);

        // 依据「文件是否真实落盘」判定完成度（无法定位目录时回退 parser 完整性）
        let verdict = if cancelled {
            Verdict::Cancelled
        } else {
            evaluate_completion(exit_ok, &merged, &verify_ctx)
        };

        let final_status = match verdict {
            Verdict::Completed => TaskStatus::Completed,
            Verdict::Cancelled => TaskStatus::Cancelled,
            Verdict::Incomplete => TaskStatus::Failed,
            Verdict::Failed => TaskStatus::Failed,
        };

        let line = match verdict {
            Verdict::Completed => "下载完成".to_string(),
            Verdict::Cancelled => "已取消".to_string(),
            Verdict::Incomplete => {
                "下载未完成：部分文件缺失或为空，已中止".to_string()
            }
            Verdict::Failed => "下载失败".to_string(),
        };

        // 保留累积的文件/字节信息，不再清零；仅完成时强制 100%
        let file_percent = if final_status == TaskStatus::Completed {
            100.0
        } else {
            merged.file_percent
        };
        let ev = ProgressEvent {
            task_id: tid.clone(),
            status: final_status,
            file_index: merged.file_index,
            file_total: merged.file_total,
            current_file: merged.current_file,
            file_percent,
            speed: String::new(),
            downloaded: merged.downloaded,
            total: merged.total,
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

#[cfg(test)]
mod verify_tests {
    use super::*;

    fn mk_ctx(root: Option<std::path::PathBuf>, files: Vec<&str>) -> VerifyCtx {
        VerifyCtx {
            root,
            files: files.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn merged(fi: u32, ft: u32) -> MergedProgress {
        MergedProgress {
            file_index: fi,
            file_total: ft,
            current_file: String::new(),
            file_percent: if ft > 0 { fi as f64 / ft as f64 * 100.0 } else { 0.0 },
            downloaded: String::new(),
            total: String::new(),
        }
    }

    #[test]
    fn exit_fail_is_failed() {
        let ctx = mk_ctx(None, vec!["a.bin"]);
        assert!(matches!(
            evaluate_completion(false, &merged(1, 1), &ctx),
            Verdict::Failed
        ));
    }

    #[test]
    fn unreachable_dir_falls_back_to_parser_incomplete() {
        // 无根目录 + parser 报告 1/2 -> Incomplete（不谎报完成）
        let ctx = mk_ctx(None, vec!["a.bin", "b.bin"]);
        assert!(matches!(
            evaluate_completion(true, &merged(1, 2), &ctx),
            Verdict::Incomplete
        ));
    }

    #[test]
    fn unreachable_dir_parser_all_done_is_completed() {
        let ctx = mk_ctx(None, vec!["a.bin", "b.bin"]);
        assert!(matches!(
            evaluate_completion(true, &merged(2, 2), &ctx),
            Verdict::Completed
        ));
    }

    #[test]
    fn no_info_respects_exit_code() {
        let ctx = mk_ctx(None, vec![]);
        assert!(matches!(
            evaluate_completion(true, &merged(0, 0), &ctx),
            Verdict::Completed
        ));
    }

    #[test]
    fn disk_files_present_is_completed() {
        let dir = std::env::temp_dir().join(format!("mirrordl_test_{}", std::process::id()));
        let sub = dir.join("repo");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("a.bin"), b"x").unwrap();
        std::fs::write(sub.join("b.bin"), b"yy").unwrap();
        let ctx = mk_ctx(Some(dir.clone()), vec!["a.bin", "b.bin"]);
        assert!(matches!(
            evaluate_completion(true, &merged(0, 0), &ctx),
            Verdict::Completed
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn disk_file_missing_is_incomplete_even_if_exit_ok() {
        let dir = std::env::temp_dir().join(format!("mirrordl_test_miss_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.bin"), b"x").unwrap();
        // b.bin 缺失
        let ctx = mk_ctx(Some(dir.clone()), vec!["a.bin", "b.bin"]);
        assert!(matches!(
            evaluate_completion(true, &merged(2, 2), &ctx),
            Verdict::Incomplete
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_file_is_incomplete() {
        let dir = std::env::temp_dir().join(format!("mirrordl_test_empty_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.bin"), b"").unwrap(); // 0 字节
        let ctx = mk_ctx(Some(dir.clone()), vec!["a.bin"]);
        assert!(matches!(
            evaluate_completion(true, &merged(1, 1), &ctx),
            Verdict::Incomplete
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
