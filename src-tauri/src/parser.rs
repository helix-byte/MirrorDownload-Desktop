//! 解析 CLI 文本输出：find 的 JSON、probe 的表格、pull 的进度行。

use crate::models::{ProbeReport, ProbeResult, ProgressEvent, RepoInfo, TaskStatus};

/// 从 find 输出中剥离前缀提示，提取 JSON 主体并解析。
/// CLI 会先输出一行 "  Querying ... from ..." 再输出 JSON。
pub fn parse_find_json(raw: &str) -> Result<RepoInfo, String> {
    // 找到第一个 '{' 与最后一个 '}'
    let start = raw.find('{').ok_or_else(|| "输出中未找到 JSON".to_string())?;
    let end = raw.rfind('}').ok_or_else(|| "输出 JSON 不完整".to_string())?;
    if end < start {
        return Err("输出 JSON 片段异常".to_string());
    }
    let json = &raw[start..=end];
    serde_json::from_str::<RepoInfo>(json).map_err(|e| format!("解析仓库 JSON 失败：{e}"))
}

/// 解析 probe 表格输出。
/// 样例：
///   Source           Status      Latency
///   ------------------------------------------
///   Hugging Face      FAIL           --- (This operation was aborted)
///   HF Mirror          OK             1s <-- best
///   Recommended source: HF Mirror
pub fn parse_probe(raw: &str) -> Result<ProbeReport, String> {
    let mut results = Vec::new();
    let mut recommended = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Probing") || trimmed.starts_with("---") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Recommended source:") {
            recommended = rest.trim().to_string();
            continue;
        }
        if trimmed.starts_with("Source") && trimmed.contains("Status") {
            continue;
        }
        // 数据行：名称可能有空格，以 OK/FAIL 分界
        let is_ok = trimmed.contains("OK");
        let is_fail = trimmed.contains("FAIL");
        if !is_ok && !is_fail {
            continue;
        }
        // 以状态词切分：源名在左，延迟在右
        let split_token = if is_fail { "FAIL" } else { "OK" };
        let Some(pos) = trimmed.find(split_token) else {
            continue;
        };
        let name = trimmed[..pos].trim().to_string();
        let mut right = trimmed[pos + split_token.len()..].trim().to_string();

        let best = right.contains("<-- best") || right.contains("<--best");
        right = right.replace("<-- best", "").replace("<--best", "");
        let right = right.trim();

        // 解析延迟：形如 "1s" / "2s" / "350ms" / "---"
        let latency_ms = parse_latency(right);
        let status = if is_fail { "FAIL" } else { "OK" }.to_string();
        let source_label = normalize_source_name(&name);

        results.push(ProbeResult {
            source: source_label,
            status,
            latency_ms,
            best,
        });
    }

    if recommended.is_empty() {
        // 取延迟最小的 OK 源
        recommended = results
            .iter()
            .filter(|r| r.status == "OK")
            .min_by_key(|r| r.latency_ms.unwrap_or(u64::MAX))
            .map(|r| r.source.clone())
            .unwrap_or_default();
    }

    if results.is_empty() {
        return Err("未能解析 probe 结果".to_string());
    }
    Ok(ProbeReport {
        results,
        recommended,
    })
}

fn normalize_source_name(name: &str) -> String {
    let n = name.trim();
    if n.eq_ignore_ascii_case("hugging face") || n.eq_ignore_ascii_case("huggingface") {
        "Hugging Face".to_string()
    } else if n.eq_ignore_ascii_case("hf mirror") || n.eq_ignore_ascii_case("hf-mirror") {
        "HF Mirror".to_string()
    } else if n.eq_ignore_ascii_case("modelscope") {
        "ModelScope".to_string()
    } else {
        n.to_string()
    }
}

fn parse_latency(right: &str) -> Option<u64> {
    let r = right.trim();
    if r.starts_with("---") {
        return None;
    }
    // "1s" / "1.2s"
    if let Some(s) = r.strip_suffix('s') {
        if let Ok(v) = s.trim().parse::<f64>() {
            return Some((v * 1000.0) as u64);
        }
    }
    // "350ms"
    if let Some(ms) = r.strip_suffix("ms") {
        if let Ok(v) = ms.trim().parse::<u64>() {
            return Some(v);
        }
    }
    // 纯数字
    r.split_whitespace()
        .next()
        .and_then(|tok| tok.trim_end_matches('s').parse::<u64>().ok())
}

/// 解析 pull 输出的单行，产出可选的进度事件增量。
///
/// 关注的模式：
/// - `[1/2] 50% filename`        -> 文件级进度
/// - `13.27 KiB/s | 663 B / 0 B` -> 速度/字节
/// - `Done filename (a/b)`       -> 单文件完成
/// - `Summary: 2 downloaded ...`  -> 结束汇总
pub struct PullParser {
    pub task_id: String,
    pub file_total: u32,
    file_index: u32,
    current_file: String,
    file_percent: f64,
    speed: String,
    downloaded: String,
    total: String,
}

impl PullParser {
    pub fn new(task_id: String) -> Self {
        PullParser {
            task_id,
            file_total: 0,
            file_index: 0,
            current_file: String::new(),
            file_percent: 0.0,
            speed: String::new(),
            downloaded: String::new(),
            total: String::new(),
        }
    }

    /// 输入一行原始输出，返回可能应推送的进度事件（无变化返回 None）
    pub fn feed(&mut self, line: &str) -> Option<ProgressEvent> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut changed = false;

        // 文件级：[i/n] 50% name
        if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                let range = &trimmed[1..close];
                if let Some((a, b)) = range.split_once('/') {
                    if let (Ok(i), Ok(n)) = (a.trim().parse::<u32>(), b.trim().parse::<u32>()) {
                        self.file_index = i;
                        self.file_total = n;
                        changed = true;
                    }
                }
                let rest = trimmed[close + 1..].trim();
                // rest: "50% filename"
                if let Some(pct_end) = rest.find('%') {
                    if let Ok(p) = rest[..pct_end].trim().parse::<f64>() {
                        self.file_percent = p;
                        changed = true;
                    }
                    self.current_file = rest[pct_end + 1..].trim().to_string();
                }
            }
        }

        // 速度行：可能带前导 spinner 字符，形如 "X | 13.27 KiB/s | 663 B / 0 B"
        if trimmed.contains("/s") && trimmed.contains('|') {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() >= 2 {
                self.speed = parts[1].trim().to_string();
                changed = true;
            }
            if parts.len() >= 3 {
                let byte_part = parts[2].trim();
                if let Some((dl, tot)) = byte_part.split_once('/') {
                    self.downloaded = dl.trim().to_string();
                    self.total = tot.trim().to_string();
                }
            }
        }

        // 单文件完成
        if trimmed.starts_with("Done ") {
            if let Some(paren) = trimmed.find('(') {
                self.current_file = trimmed[5..paren].trim().to_string();
            }
            self.file_percent = 100.0;
            changed = true;
        }

        if changed {
            Some(ProgressEvent {
                task_id: self.task_id.clone(),
                status: TaskStatus::Running,
                file_index: self.file_index,
                file_total: self.file_total,
                current_file: self.current_file.clone(),
                file_percent: self.file_percent,
                speed: self.speed.clone(),
                downloaded: self.downloaded.clone(),
                total: self.total.clone(),
                line: line.to_string(),
            })
        } else {
            None
        }
    }

    /// 构造一个终态事件
    pub fn terminal(&self, status: TaskStatus, line: &str) -> ProgressEvent {
        let completed = status == TaskStatus::Completed;
        ProgressEvent {
            task_id: self.task_id.clone(),
            status,
            file_index: self.file_index,
            file_total: self.file_total,
            current_file: self.current_file.clone(),
            file_percent: if completed {
                100.0
            } else {
                self.file_percent
            },
            speed: self.speed.clone(),
            downloaded: self.downloaded.clone(),
            total: self.total.clone(),
            line: line.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_find_json_with_prefix() {
        let raw = "  Querying Qwen/Qwen2.5-7B-Instruct from hf-mirror...\n\n{\n  \"id\": \"Qwen/Qwen2.5-7B-Instruct\",\n  \"source\": \"hf\",\n  \"revision\": \"main\",\n  \"repoType\": \"model\",\n  \"files\": [{\"path\": \"a.txt\", \"lfs\": false}]\n}\n";
        let info = parse_find_json(raw).unwrap();
        assert_eq!(info.id, "Qwen/Qwen2.5-7B-Instruct");
        assert_eq!(info.files.len(), 1);
    }

    #[test]
    fn test_parse_probe_table() {
        let raw = "\nProbing upstream sources...\n\n  Source           Status      Latency\n  ------------------------------------------\n  Hugging Face      FAIL           --- (This operation was aborted)\n  HF Mirror          OK             1s <-- best\n  ModelScope         OK             2s\n\n  Recommended source: HF Mirror\n";
        let report = parse_probe(raw).unwrap();
        assert_eq!(report.results.len(), 3);
        assert_eq!(report.recommended, "HF Mirror");
        let hf = report.results.iter().find(|r| r.source == "Hugging Face").unwrap();
        assert_eq!(hf.status, "FAIL");
        assert!(hf.latency_ms.is_none());
        let mirror = report.results.iter().find(|r| r.source == "HF Mirror").unwrap();
        assert_eq!(mirror.latency_ms, Some(1000));
        assert!(mirror.best);
    }

    #[test]
    fn test_pull_parser_file_progress() {
        let mut p = PullParser::new("t1".into());
        let ev = p.feed("[1/2] 50% .gitattributes").unwrap();
        assert_eq!(ev.file_index, 1);
        assert_eq!(ev.file_total, 2);
        assert_eq!(ev.file_percent, 50.0);
        assert_eq!(ev.current_file, ".gitattributes");
    }

    #[test]
    fn test_pull_parser_speed_line() {
        let mut p = PullParser::new("t1".into());
        let ev = p.feed("? | 13.27 KiB/s | 663 B / 0 B").unwrap();
        assert_eq!(ev.speed, "13.27 KiB/s");
        assert_eq!(ev.downloaded, "663 B");
        assert_eq!(ev.total, "0 B");
    }
}
