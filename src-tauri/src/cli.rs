//! CLI 定位与执行封装。
//!
//! 上游镜像下载器是 Node.js 包 `mirror-downloader`，提供 `mirrordl` 命令。
//! 本模块负责：
//! 1. 在系统 PATH 中查找 `mirrordl` / `mirror-downloader`；
//! 2. 若找不到命令，则回退为 `node <全局包>/dist/cli.js` 调用方式；
//! 3. 统一封装子进程创建，便于上层进行流式读取。

use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

use crate::models::CliStatus;

/// CLI 的调用方式
#[derive(Debug, Clone)]
pub enum CliInvocation {
    /// 直接调用命令（如 `mirrordl`）
    Direct { program: String },
    /// 通过 node 调用入口脚本
    Node { node: String, cli_js: String },
}

impl CliInvocation {
    /// 构造一个已配置串行输出管道的基础 Command
    pub fn command(&self) -> Command {
        match self {
            CliInvocation::Direct { program } => {
                let mut c = Command::new(program);
                c.stdout(Stdio::piped()).stderr(Stdio::piped());
                c
            }
            CliInvocation::Node { node, cli_js } => {
                let mut c = Command::new(node);
                c.arg(cli_js)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                c
            }
        }
    }

    /// 人类可读的路径描述，供前端展示
    pub fn display_path(&self) -> String {
        match self {
            CliInvocation::Direct { program } => program.clone(),
            CliInvocation::Node { node, cli_js } => format!("{node} {cli_js}"),
        }
    }
}

/// 在 npm 全局根目录中定位 dist/cli.js
fn find_global_cli_js() -> Option<PathBuf> {
    // 常见全局 node_modules 位置
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. 环境变量可指定的前缀
    if let Ok(prefix) = std::env::var("MIRRORDL_CLI_JS") {
        let p = PathBuf::from(prefix);
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. 基于当前 node 可执行文件位置推断全局目录
    if let Ok(node_path) = which::which("node") {
        if let Some(node_dir) = node_path.parent() {
            // Windows: <nodejs>\node_modules\mirror-downloader\dist\cli.js
            candidates.push(node_dir.join("node_modules/mirror-downloader/dist/cli.js"));
            // Windows: <nodejs>\node_modules 或 <nodejs>\lib\node_modules (nvm 等)
            candidates.push(node_dir.join("lib/node_modules/mirror-downloader/dist/cli.js"));
            if let Some(parent) = node_dir.parent() {
                candidates.push(parent.join("lib/node_modules/mirror-downloader/dist/cli.js"));
            }
        }
    }

    // 3. 基于 APPDATA（Windows npm 全局）
    if let Ok(appdata) = std::env::var("APPDATA") {
        candidates.push(
            PathBuf::from(appdata)
                .join("npm/node_modules/mirror-downloader/dist/cli.js"),
        );
    }

    candidates.into_iter().find(|p| p.is_file())
}

/// 定位可用于执行的 CLI 调用方式
pub fn resolve_cli() -> Option<CliInvocation> {
    // 优先直接命令
    if let Ok(p) = which::which("mirrordl") {
        return Some(CliInvocation::Direct {
            program: p.to_string_lossy().to_string(),
        });
    }
    if let Ok(p) = which::which("mirror-downloader") {
        return Some(CliInvocation::Direct {
            program: p.to_string_lossy().to_string(),
        });
    }
    // 回退：node + 全局 cli.js
    if let (Ok(node), Some(cli_js)) = (which::which("node"), find_global_cli_js()) {
        return Some(CliInvocation::Node {
            node: node.to_string_lossy().to_string(),
            cli_js: cli_js.to_string_lossy().to_string(),
        });
    }
    None
}

/// 检测 CLI 是否可用并返回版本
pub async fn check_cli() -> CliStatus {
    let Some(inv) = resolve_cli() else {
        return CliStatus {
            available: false,
            path: String::new(),
            version: String::new(),
            message: "未找到 mirrordl CLI。请先执行 `npm install -g mirror-downloader`，并确保 node 在 PATH 中。"
                .to_string(),
        };
    };

    match run_capture(&inv, &["--version"]).await {
        Ok(out) => {
            let version = out.trim().lines().last().unwrap_or("").trim().to_string();
            CliStatus {
                available: true,
                path: inv.display_path(),
                version,
                message: String::new(),
            }
        }
        Err(e) => CliStatus {
            available: false,
            path: inv.display_path(),
            version: String::new(),
            message: format!("CLI 调用失败：{e}"),
        },
    }
}

/// 执行并一次性捕获全部输出（用于 find --json、--version 等短命令）
pub async fn run_capture(inv: &CliInvocation, args: &[&str]) -> Result<String, String> {
    let mut cmd = inv.command();
    cmd.args(args);
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("启动 CLI 失败：{e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // 进程失败时，优先把 stderr 中的真实原因暴露给上层，避免只留下残缺的 stdout
    // （例如 CLI 遇到 404 后崩溃，stdout 仅剩提示行，若直接返回会误报“未找到 JSON”）。
    if !output.status.success() {
        if let Some(msg) = first_error_line(&stderr) {
            return Err(msg);
        }
        if stdout.trim().is_empty() {
            return Err(format!("CLI 退出码 {:?}", output.status.code()));
        }
    }
    // 合并 stderr 中的提示（CLI 常把进度写 stderr）
    if stdout.trim().is_empty() {
        Ok(stderr)
    } else {
        Ok(stdout)
    }
}

/// 从 stderr 中提取第一条可读的错误信息，过滤掉空行与崩溃断言等噪声。
fn first_error_line(stderr: &str) -> Option<String> {
    stderr
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .find(|l| {
            // 跳过 libuv/node 崩溃断言这类无意义噪声
            let lower = l.to_lowercase();
            !lower.contains("assertion failed")
                && !lower.contains("uv_handle")
                && !lower.contains("handle->flags")
        })
        .map(|l| l.trim_start_matches('❌').trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::first_error_line;

    #[test]
    fn picks_real_error_and_skips_assertion_noise() {
        let stderr = "  ❌ Repository not found or inaccessible: Failed to list files\nAssertion failed: !(handle->flags & UV_HANDLE_CLOSING), file src\\win\\async.c, line 94\n";
        assert_eq!(
            first_error_line(stderr).as_deref(),
            Some("Repository not found or inaccessible: Failed to list files")
        );
    }

    #[test]
    fn returns_none_when_only_noise() {
        let stderr = "\nAssertion failed: !(handle->flags & UV_HANDLE_CLOSING)\n";
        assert!(first_error_line(stderr).is_none());
    }

    #[test]
    fn returns_none_for_empty() {
        assert!(first_error_line("   \n\t\n").is_none());
    }
}
