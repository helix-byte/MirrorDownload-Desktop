//! 真实 CLI 集成测试：验证后端 resolve_cli / run_capture / 解析链路。
//!
//! 这些用例会真实调用本机 mirrordl CLI（需要网络），默认 `#[ignore]`，
//! 手动运行：`cargo test --test cli_integration -- --ignored --nocapture`

use mirrordl_desktop_lib::testing;

#[test]
#[ignore]
fn real_cli_resolve_and_version() {
    let inv = testing::resolve_cli_for_test().expect("未找到 mirrordl CLI");
    println!("CLI 调用方式：{}", inv_display(&inv));

    let rt = tokio::runtime::Runtime::new().unwrap();
    let out = rt
        .block_on(testing::run_capture_for_test(&inv, &["--version"]))
        .expect("--version 调用失败");
    println!("--version 输出：{out}");
    assert!(out.contains('.') || !out.trim().is_empty());
}

#[test]
#[ignore]
fn real_cli_probe_parses() {
    let inv = testing::resolve_cli_for_test().expect("未找到 mirrordl CLI");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let raw = rt
        .block_on(testing::run_capture_for_test(
            &inv,
            &["probe", "--timeout", "4000"],
        ))
        .expect("probe 调用失败");
    let report = testing::parse_probe_for_test(&raw).expect("probe 解析失败");
    println!("probe 结果：{:?}", report);
    assert!(!report.results.is_empty());
}

#[test]
#[ignore]
fn real_cli_find_parses() {
    let inv = testing::resolve_cli_for_test().expect("未找到 mirrordl CLI");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let raw = rt
        .block_on(testing::run_capture_for_test(
            &inv,
            &["find", "Qwen/Qwen2.5-7B-Instruct", "--json"],
        ))
        .expect("find 调用失败");
    let info = testing::parse_find_json_for_test(&raw).expect("find 解析失败");
    println!("find 结果：id={} files={}", info.id, info.files.len());
    assert_eq!(info.id, "Qwen/Qwen2.5-7B-Instruct");
    assert!(!info.files.is_empty());
}

fn inv_display(inv: &testing::TestInvocation) -> String {
    format!("{inv:?}")
}

#[test]
#[ignore]
fn real_cli_pull_progress_parses() {
    // 真实下载单个小文件，验证 PullParser 能解析出文件进度/速度
    let inv = testing::resolve_cli_for_test().expect("未找到 mirrordl CLI");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let tmp = std::env::temp_dir().join(format!("mirrordl_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp);
    let out = tmp.to_string_lossy().to_string();
    let raw = rt
        .block_on(testing::run_capture_for_test(
            &inv,
            &[
                "pull",
                "Qwen/Qwen2.5-7B-Instruct",
                "--source",
                "hf-mirror",
                "-f",
                ".gitattributes",
                "-o",
                &out,
                "--no-symlinks",
            ],
        ))
        .expect("pull 调用失败");
    let mut parser = testing::PullParser::new("t".to_string());
    let mut saw_file_progress = false;
    let mut saw_speed = false;
    for line in raw.lines() {
        if let Some(ev) = parser.feed(line) {
            if ev.file_total > 0 {
                saw_file_progress = true;
            }
            if !ev.speed.is_empty() {
                saw_speed = true;
            }
        }
    }
    println!("saw_file_progress={saw_file_progress} saw_speed={saw_speed}");
    assert!(saw_file_progress, "未能解析文件级进度");
    let _ = std::fs::remove_dir_all(&tmp);
}
