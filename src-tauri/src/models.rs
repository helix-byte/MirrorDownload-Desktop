//! 数据模型：与前端约定的结构化类型，以及 CLI `find --json` 的解析结构。

use serde::{Deserialize, Serialize};

/// 下载源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    Hf,
    HfMirror,
    Modelscope,
}

impl Source {
    #[allow(dead_code)]
    pub fn as_arg(&self) -> &'static str {
        match self {
            Source::Hf => "hf",
            Source::HfMirror => "hf-mirror",
            Source::Modelscope => "modelscope",
        }
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// 默认下载源
    pub source: Source,
    /// 并发文件/线程数
    pub threads: u32,
    /// HuggingFace Token
    pub hf_token: String,
    /// ModelScope Token
    pub modelscope_token: String,
    /// 私有仓库通用 Token
    pub token: String,
    /// HTTP/HTTPS 代理
    pub proxy: String,
    /// 默认输出目录（留空则由 CLI / 设置决定）
    pub output_dir: String,
    /// 目标模式: none / comfyui / lmstudio
    pub target_mode: String,
    /// ComfyUI 根目录
    pub comfyui_root: String,
    /// LM Studio 根目录
    pub lmstudio_root: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            source: Source::HfMirror,
            threads: 5,
            hf_token: String::new(),
            modelscope_token: String::new(),
            token: String::new(),
            proxy: String::new(),
            output_dir: String::new(),
            target_mode: "none".to_string(),
            comfyui_root: String::new(),
            lmstudio_root: String::new(),
        }
    }
}

/// find --json 输出中的单个文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoFile {
    pub path: String,
    #[serde(default)]
    pub lfs: bool,
    /// 文件大小（字节），CLI 未提供时为 None
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// find --json 的完整仓库信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub id: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub revision: String,
    #[serde(default)]
    pub repo_type: String,
    #[serde(default)]
    pub files: Vec<RepoFile>,
}

/// probe 单个源的探测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub source: String,
    pub status: String,
    /// 延迟毫秒；失败为 None
    pub latency_ms: Option<u64>,
    pub best: bool,
}

/// probe 汇总结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeReport {
    pub results: Vec<ProbeResult>,
    pub recommended: String,
}

/// 下载任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 推送给前端的下载进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub task_id: String,
    pub status: TaskStatus,
    /// 当前文件序号（从 1 开始）
    pub file_index: u32,
    /// 总文件数
    pub file_total: u32,
    /// 当前文件名
    pub current_file: String,
    /// 当前文件进度百分比
    pub file_percent: f64,
    /// 当前速度文本（如 "13.27 MiB/s"）
    pub speed: String,
    /// 已下载字节文本
    pub downloaded: String,
    /// 总字节文本
    pub total: String,
    /// 原始日志行
    pub line: String,
}

/// CLI 可用性检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliStatus {
    pub available: bool,
    /// 解析到的 CLI 可执行路径或入口
    pub path: String,
    /// 版本号
    pub version: String,
    /// 错误信息（不可用时）
    pub message: String,
}
