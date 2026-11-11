# AGENTS.md

本文件为 AI 编码助手（DSH / Codex / Claude Code / Cursor 等）提供本仓库的**稳定事实、约定与红线**。开始任何任务前请先阅读，并遵守其中的命令与规范。

---

## 1. 项目概览

- **名称**：Mirror Download 桌面版（`mirrordl-desktop`）
- **定位**：上游 `mirror-downloader` CLI 的图形界面封装，面向国内用户高速下载 HuggingFace / HF Mirror / ModelScope 上的 AI 模型与数据集。
- **技术栈**：Tauri 2（Rust 后端）+ Vue 3 + TypeScript + Vite + Pinia + Vue Router。
- **下载引擎**：上游 Node.js CLI（`mirrordl` / `mirror-downloader`），后端以子进程方式调用并流式解析输出。

## 2. 目录结构

```
src/                    # Vue 前端
├── api/                # Tauri IPC 封装与类型（含浏览器 mock）
├── components/         # UI 组件（browse / download / settings / ui）
├── router/             # 路由
├── stores/             # Pinia：settings / downloads
├── styles/             # 全局深色主题
└── views/              # 页面：Browse / Download / Settings
src-tauri/              # Rust 后端
├── src/
│   ├── cli.rs          # CLI 定位与执行（含错误透传 first_error_line）
│   ├── commands.rs     # Tauri 命令层（IPC 接口、下载子进程管理）
│   ├── parser.rs       # find/probe/pull 输出解析
│   ├── models.rs       # 数据模型
│   ├── settings.rs     # 设置持久化
│   └── lib.rs / main.rs
└── tests/              # 集成测试（真实 CLI，需联网）
.agents/skills/         # 项目级协作技能（github-flow / git-commit / github-repo-bootstrap）
scripts/tauri-build.mjs # 打包入口，规避 Windows MSI 打包竞态
```

## 3. 常用命令

前端 / 桌面：

```bash
npm install            # 安装前端依赖
npm run dev            # 纯浏览器调试（mock 数据，无需 Rust）
npm run build          # 类型检查 + 前端构建（vue-tsc --noEmit && vite build）
npm run tauri:dev      # 桌面开发模式（真实 CLI；改动 Rust/前端后需重启）
npm run tauri:build    # 生产打包（经 scripts/tauri-build.mjs）
```

后端：

```bash
cd src-tauri
cargo test --lib                                       # 后端单元测试
cargo test --test cli_integration -- --ignored --nocapture   # 真实 CLI 集成测试（联网）
```

前置依赖：Node ≥ 18；全局安装上游 CLI `npm install -g mirror-downloader`；Rust 工具链；Windows 需 WebView2。

## 4. 开发与提交规范（强制）

- **分支模型：GitHub Flow**。禁止直接向 `main` 提交；所有改动走特性分支 + PR 合并；合并后删除分支。
  - 分支命名：`feature/` / `fix/` / `docs/` / `refactor/` / `chore/` + 小写短横线描述。
- **提交信息：遵循 `git-commit` 规范**（gitmoji + 约定式提交 + 中文三段式 body「背景/改动/影响」+ footer）。
- 修改前先 `git fetch` 并确认与远端同步；有未提交改动先处理，不得静默丢弃。
- 一次提交保持单一意图；PR 与提交信息风格一致。
- 合并方式：仓库启用 **Squash merge**（历史保持线性）。详见 `.agents/skills/github-flow/SKILL.md`。

## 5. 编码约定

- **前端**：TypeScript 严格模式；组件放 `components/`，页面放 `views/`，状态放 `stores/`（Pinia setup store）。
- **IPC**：前端通过 `src/api/index.ts` 封装调用后端命令；浏览器下走 mock，新增命令时两端（真实 + mock）都要同步。
- **后端**：命令在 `commands.rs`，CLI 调用统一走 `cli.rs`；解析逻辑放 `parser.rs` 并配单元测试。
- 新增 Tauri 命令后需在 `lib.rs` 注册。
- 涉及 CLI 参数拼装时，注意源/分支等默认值不要硬编码（如 repository revision 应留空由 CLI 用各源默认分支）。

## 6. 易错点与注意事项（红线）

1. **下载引擎依赖上游 CLI**：未安装 `mirrordl` 时下载/查询会失败，错误信息需透传 stderr 真实原因（见 `cli.rs::first_error_line`），不要只报笼统"下载失败"。
2. **ModelScope 大文件分片下载**：其服务端对带 `Range` 的请求返回 `302`，上游引擎未跟随会报 `Chunk 0: HTTP 302`——这是已知的源侧/引擎兼容问题，排查下载失败时优先核对数据源。
3. **版本（revision）不要硬编码**：HF 默认 `main`、ModelScope 默认 `master`，正确做法是留空让各源用默认分支。
4. **数据源「跟随设置」**：前端传入空表示跟随设置，必须回退到 `settings.source`，否则会走 CLI 默认源（hf）。
5. **Windows 打包**：`tauri:build` 必须走 `scripts/tauri-build.mjs`（重定向 Bundler 临时目录，规避 WiX `light.exe` 被杀软占用导致的 `LGHT0001`）；不要改回直接 `tauri build`。
6. **不要提交**：`node_modules/`、`dist/`、`src-tauri/target/`（含 `.bundler-tmp`）、`.transferx-downloads/`、`docs/`（内部过程文档与截图）、`.tmp/`、`.env*`、签名密钥 `*.pem`/`*.key`。
7. **不要静默丢弃用户的未提交改动**；破坏性操作（force push、删除分支、重置历史）前先确认。
8. **许可证口径**：本项目 GUI 代码 MIT，上游 CLI 为 BSL 1.1；LICENSE 文件正文保持纯 MIT，说明散见于 README/此处，避免破坏 GitHub 许可证徽章识别。

## 7. 设置存储位置

- Windows：`%APPDATA%\mirrordl-desktop\settings.json`
- Linux：`~/.config/mirrordl-desktop/settings.json`
- macOS：`~/Library/Application Support/mirrordl-desktop/settings.json`

上游 CLI 自身仍读取 `%APPDATA%/mirror-dl/config.toml`；本应用通过命令行参数显式传递源/Token/代理，避免依赖该文件。

## 8. 相关技能

- `.agents/skills/github-flow/SKILL.md`：分支/PR/合并完整流程。
- `.agents/skills/git-commit/SKILL.md`：提交信息生成规范。
- `.agents/skills/github-repo-bootstrap/SKILL.md`：新项目建仓与初始化流程。
