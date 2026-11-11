//! 文件大小补全：上游 CLI 的 `find --json` 不返回文件大小，
//! 这里在 `find_repo` 之后按所选数据源额外查询仓库文件树，为 `RepoInfo.files`
//! 回填 `size`（字节）。
//!
//! 设计要点（best-effort，绝不阻断主查询）：
//! - 三个源各自有文件树接口：
//!   * HF / HF Mirror：`/api/models/{id}/tree/{rev}?recursive=true&expand=true`（JSON 数组）
//!   * ModelScope：`/api/v1/models/{id}/repo/files?Revision={rev}&Root=&Recursive=true`
//! - 请求失败、超时、解析异常、字段缺失等一律静默降级：保持原有 `size=None`，
//!   前端逻辑（`v-if="formatSize(f.size)"`）自然不显示大小，与旧行为一致。
//! - 支持 HF Token（私有仓库）、代理、超时。
//! - HF 树接口可能分页（每页默认 1000 条，通过 `Link` 头携带下一页），逐页跟进。

use std::time::Duration;

use serde::Deserialize;

use crate::models::{RepoFile, RepoInfo};

/// 拉取超时（单次请求）
const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
/// 分页安全上限，避免异常响应导致无限翻页
const MAX_PAGES: usize = 20;

/// 大小补全所需的上下文（由 find_repo 传入）
#[derive(Debug, Clone, Default)]
pub struct SizeFetchCtx {
    /// 规范化后的源：hf / hf-mirror / modelscope（空表示未知，按 hf 处理）
    pub source: String,
    /// 仓库 id（如 `Qwen/Qwen2.5-7B-Instruct`）
    pub repo_id: String,
    /// 版本 / 分支；为空时由各源用默认分支
    pub revision: String,
    /// HF token（仅写权限仓库需要）
    pub hf_token: String,
    /// ModelScope token
    pub modelscope_token: String,
    /// HTTP/HTTPS 代理
    pub proxy: String,
}

/// 从 JSON 值中提取 {path: size} 映射，忽略非文件项与无效 size。
/// 这是纯函数，便于单元测试。
fn collect_sizes_hf(items: &[serde_json::Value]) -> std::collections::HashMap<String, u64> {
    let mut map = std::collections::HashMap::new();
    for item in items {
        // 仅处理 type == "file"（目录 size 为 0，忽略）
        let is_file = item
            .get("type")
            .and_then(|v| v.as_str())
            .map(|t| t == "file")
            .unwrap_or(true); // 无 type 字段时按文件处理
        if !is_file {
            continue;
        }
        let path = match item.get("path").and_then(|v| v.as_str()) {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => continue,
        };
        if let Some(size) = item.get("size").and_then(json_u64) {
            map.insert(path, size);
        }
    }
    map
}

/// ModelScope files 项：Path / Type / Size
fn collect_sizes_modelscope(
    items: &[serde_json::Value],
) -> std::collections::HashMap<String, u64> {
    let mut map = std::collections::HashMap::new();
    for item in items {
        // Type 可能为 "blob"/"file"/"tree"；仅接受非目录项
        if let Some(t) = item.get("Type").and_then(|v| v.as_str()) {
            if t.eq_ignore_ascii_case("tree") || t.eq_ignore_ascii_case("directory") {
                continue;
            }
        }
        // ModelScope 字段大小写不统一，兼容 Size / size
        let path = item
            .get("Path")
            .or_else(|| item.get("path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let Some(path) = path.filter(|p| !p.is_empty()) else {
            continue;
        };
        let size = item
            .get("Size")
            .or_else(|| item.get("size"))
            .and_then(json_u64);
        if let Some(size) = size {
            map.insert(path, size);
        }
    }
    map
}

fn json_u64(v: &serde_json::Value) -> Option<u64> {
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    // 兼容以字符串形式返回的大小
    if let Some(s) = v.as_str() {
        return s.trim().parse::<u64>().ok();
    }
    None
}

/// 构造 HTTP 客户端（best-effort，失败返回 None）
fn build_client(proxy: &str) -> Option<reqwest::Client> {
    let mut builder = reqwest::Client::builder().timeout(FETCH_TIMEOUT);
    if !proxy.trim().is_empty() {
        if let Ok(p) = reqwest::Proxy::all(proxy.trim()) {
            builder = builder.proxy(p);
        }
    }
    builder.build().ok()
}

/// 为仓库补全文件大小。原地修改 `info.files` 中匹配 path 的 size。
/// 全程 best-effort：任何环节失败都不返回错误，仅跳过。
pub async fn enrich_sizes(info: &mut RepoInfo, ctx: &SizeFetchCtx) {
    let Some(client) = build_client(&ctx.proxy) else {
        return;
    };

    let sizes = match ctx.source.as_str() {
        "modelscope" => fetch_modelscope_sizes(&client, ctx).await,
        // hf / hf-mirror / 未知 都走 HF 兼容接口
        _ => fetch_hf_sizes(&client, ctx).await,
    };

    let Some(sizes) = sizes else {
        return; // 拉取失败：保持原样
    };
    if sizes.is_empty() {
        return;
    }
    apply_sizes(&mut info.files, &sizes);
}

/// 将 {path: size} 映射应用到文件列表（仅补齐缺失的 size）
fn apply_sizes(files: &mut [RepoFile], sizes: &std::collections::HashMap<String, u64>) {
    for f in files.iter_mut() {
        if f.size.is_some() {
            continue;
        }
        if let Some(&sz) = sizes.get(&f.path) {
            f.size = Some(sz);
        }
    }
}

/// HF / HF Mirror 域名
fn hf_base(source: &str) -> &'static str {
    match source {
        "hf-mirror" => "https://hf-mirror.com",
        _ => "https://huggingface.co",
    }
}

/// HF 树接口默认版本
fn hf_revision(rev: &str) -> &str {
    if rev.trim().is_empty() {
        "main"
    } else {
        rev.trim()
    }
}

async fn fetch_hf_sizes(
    client: &reqwest::Client,
    ctx: &SizeFetchCtx,
) -> Option<std::collections::HashMap<String, u64>> {
    let base = hf_base(&ctx.source);
    let rev = hf_revision(&ctx.revision);
    let mut url = format!(
        "{base}/api/models/{}/tree/{}?recursive=true&expand=true",
        ctx.repo_id, rev
    );

    let mut all: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut page = 0usize;
    loop {
        page += 1;
        if page > MAX_PAGES {
            break;
        }
        let mut req = client.get(&url).header("Accept", "application/json");
        if !ctx.hf_token.trim().is_empty() {
            req = req.bearer_auth(ctx.hf_token.trim());
        }
        let resp = req.send().await.ok()?;
        if !resp.status().is_success() {
            // 首页失败直接放弃；后续页失败则返回已拿到的部分
            return if page == 1 { None } else { Some(all) };
        }
        // 先取 Link 头（下一页），再消费 body
        let next = resp
            .headers()
            .get("link")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_next_link);
        let items: Vec<serde_json::Value> = resp.json().await.ok()?;
        all.extend(collect_sizes_hf(&items));
        match next {
            Some(n) => url = n,
            None => break,
        }
    }
    Some(all)
}

/// 从 `Link: <https://...>; rel="next"` 中取 URL
fn parse_next_link(link: &str) -> Option<String> {
    for part in link.split(',') {
        let part = part.trim();
        if part.contains("rel=\"next\"") {
            let start = part.find('<')?;
            let end = part.find('>')?;
            if end > start {
                return Some(part[start + 1..end].to_string());
            }
        }
    }
    None
}

async fn fetch_modelscope_sizes(
    client: &reqwest::Client,
    ctx: &SizeFetchCtx,
) -> Option<std::collections::HashMap<String, u64>> {
    let rev = if ctx.revision.trim().is_empty() {
        "master"
    } else {
        ctx.revision.trim()
    };
    let url = format!(
        "https://modelscope.cn/api/v1/models/{}/repo/files?Revision={}&Root=&Recursive=true",
        ctx.repo_id, rev
    );
    let mut req = client.get(&url).header("Accept", "application/json");
    if !ctx.modelscope_token.trim().is_empty() {
        req = req.bearer_auth(ctx.modelscope_token.trim());
    }
    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    // 结构：{ Data: { Files: [ {Path, Type, Size, ...} ] } }
    let files = body
        .get("Data")
        .and_then(|d| d.get("Files"))
        .and_then(|f| f.as_array())?;
    Some(collect_sizes_modelscope(files))
}

// ---------------- ModelScope 反序列化辅助（供集成测试使用） ----------------
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MsFileRaw {
    #[serde(rename = "Path", default)]
    path: String,
    #[serde(rename = "Size", default)]
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(json: &str) -> Vec<serde_json::Value> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn hf_collect_skips_dirs_and_keeps_files() {
        let items = v(
            r#"[
                {"type":"directory","path":"MTP","size":0},
                {"type":"file","path":".gitattributes","size":2009},
                {"type":"file","path":"README.md","size":"14367"}
            ]"#,
        );
        let map = collect_sizes_hf(&items);
        assert_eq!(map.get("MTP"), None);
        assert_eq!(map.get(".gitattributes"), Some(&2009));
        assert_eq!(map.get("README.md"), Some(&14367));
    }

    #[test]
    fn modelscope_collect_handles_case_and_skips_tree() {
        let items = v(
            r#"[
                {"Path":"sub","Type":"tree","Size":0},
                {"Path":"a.gguf","Type":"blob","Size":123},
                {"Path":"b.gguf","Type":"blob","size":456}
            ]"#,
        );
        let map = collect_sizes_modelscope(&items);
        assert_eq!(map.get("sub"), None);
        assert_eq!(map.get("a.gguf"), Some(&123));
        assert_eq!(map.get("b.gguf"), Some(&456));
    }

    #[test]
    fn apply_sizes_only_fills_missing() {
        let mut files = vec![
            RepoFile {
                path: "a.gguf".into(),
                lfs: true,
                size: None,
            },
            RepoFile {
                path: "b.gguf".into(),
                lfs: true,
                size: Some(999),
            },
        ];
        let mut map = std::collections::HashMap::new();
        map.insert("a.gguf".to_string(), 111u64);
        map.insert("b.gguf".to_string(), 222u64);
        apply_sizes(&mut files, &map);
        assert_eq!(files[0].size, Some(111));
        // 已有 size 不被覆盖
        assert_eq!(files[1].size, Some(999));
    }

    #[test]
    fn parse_next_link_extracts_url() {
        let link = r#"<https://huggingface.co/api/models/x/tree/main?recursive=true&expand=true&cursor=abc>; rel="next", <https://huggingface.co/api/models/x/tree/main>; rel="first""#;
        assert_eq!(
            parse_next_link(link).as_deref(),
            Some("https://huggingface.co/api/models/x/tree/main?recursive=true&expand=true&cursor=abc")
        );
        assert!(parse_next_link("").is_none());
    }

    #[test]
    fn json_u64_accepts_number_and_string() {
        let n: serde_json::Value = serde_json::json!(123);
        let s: serde_json::Value = serde_json::json!("456");
        let bad: serde_json::Value = serde_json::json!("x");
        assert_eq!(json_u64(&n), Some(123));
        assert_eq!(json_u64(&s), Some(456));
        assert_eq!(json_u64(&bad), None);
    }

    #[test]
    fn hf_base_and_revision_defaults() {
        assert_eq!(hf_base("hf-mirror"), "https://hf-mirror.com");
        assert_eq!(hf_base("hf"), "https://huggingface.co");
        assert_eq!(hf_base(""), "https://huggingface.co");
        assert_eq!(hf_revision(""), "main");
        assert_eq!(hf_revision("dev"), "dev");
    }
}
