# Mirror Download 桌面版

[Mirror Download](https://github.com/mirrordownload/mirrordownload) 的图形界面版本，基于 **Tauri 2 + Vue 3 + Rust** 构建，为国内用户下载 HuggingFace / HF Mirror / ModelScope 上的 AI 模型与数据集提供现代化的桌面体验。

> 核心下载能力复用上游 `mirror-downloader` CLI（`mirrordl`）：多源并发、断点续传、IDM 级自适应并发引擎（8~64 条动态连接）。

---

## 特性

- 🧭 **源探测**：一键探测 HuggingFace / HF Mirror / ModelScope 各源延迟与可用性，自动推荐最优源
- 🔍 **模型浏览**：输入仓库 ID 查询文件列表，支持按文件名过滤、量化类型（q4_k_m / q8_0 / fp16 / awq / gguf 等）筛选
- ⬇️ **下载队列**：多任务并行，实时显示文件级进度、下载速度、已下载/总字节
- 🎯 **目标适配**：一键下载到 ComfyUI 目录（自动分类）或 LM Studio 目录
- ⚙️ **设置持久化**：下载源、线程数、Token（HF / ModelScope / 私有仓库）、代理、输出目录
- 🌙 **深色现代化界面**：侧边栏导航，原生桌面体验

---

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面外壳 | Tauri 2（Rust） |
| 前端 | Vue 3 + TypeScript + Vite + Pinia + Vue Router |
| 后端 | Rust（`tokio` 异步子进程 + 流式读取） |
| 下载引擎 | 上游 `mirror-downloader` CLI（Node.js） |

### 架构

```
Vue 前端 (视图 / Pinia store / api 封装)
      │  Tauri IPC (invoke / event)
      ▼
Rust 后端 (commands / cli / parser / settings)
      │  spawn 子进程 + 逐行读取 stdout/stderr
      ▼
mirrordl CLI (probe / find / pull)
      │
      ▼
@transferx/downloader 自适应并发引擎 → 镜像源
```

---

## 前置依赖

1. **Node.js ≥ 18**（用于运行上游 CLI）
2. **全局安装上游 CLI**：
   ```bash
   npm install -g mirror-downloader
   ```
   后端会自动在 PATH 中查找 `mirrordl` / `mirror-downloader`；若未在 PATH，也会回退到 npm 全局目录的 `dist/cli.js`（通过 `node` 调用）。

3. 构建桌面应用还需 **Rust 工具链** 与 **Tauri 系统依赖**（Windows 需 WebView2）。

---

## 开发

```bash
# 安装前端依赖
npm install

# 纯前端浏览器调试（含 mock 数据，无需 Tauri/Rust）
npm run dev

# 桌面应用开发模式（Tauri 窗口 + 真实 CLI）
npm run tauri:dev

# 生产构建（生成安装包）
npm run tauri:build
```

### 设置存储位置

应用设置以 JSON 保存在系统配置目录：

- Windows：`%APPDATA%\mirrordl-desktop\settings.json`
- Linux：`~/.config/mirrordl-desktop/settings.json`
- macOS：`~/Library/Application Support/mirrordl-desktop/settings.json`

> 上游 CLI 自身仍读取 `%APPDATA%/mirror-dl/config.toml`（或对应平台的 `config.toml`）。本应用通过命令行参数显式传递源 / Token / 代理等，避免依赖该文件。

---

## 测试

```bash
# 后端解析器单元测试
cd src-tauri && cargo test

# 真实 CLI 集成测试（需联网 + 已安装 CLI）
cd src-tauri && cargo test --test cli_integration -- --ignored --nocapture
```

---

## 目录结构

```
├── src/                     # Vue 前端
│   ├── api/                 # Tauri IPC 封装与类型（含浏览器 mock）
│   ├── stores/              # Pinia：settings / downloads
│   ├── views/               # 下载 / 模型浏览 / 设置
│   ├── components/          # UI 组件
│   ├── router/              # 路由
│   └── styles/              # 全局深色主题
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── cli.rs           # CLI 定位与执行
│   │   ├── parser.rs        # find/probe/pull 输出解析
│   │   ├── commands.rs      # Tauri 命令
│   │   ├── models.rs        # 数据模型
│   │   ├── settings.rs      # 设置持久化
│   │   └── lib.rs / main.rs
│   ├── tests/               # 集成测试
│   └── tauri.conf.json
└── agents/skills/           # 协作规范（github-flow / git-commit）
```

---

## 许可证

上游 `mirror-downloader` 采用 BSL 1.1（2030-01-01 后转 Apache 2.0）。本项目作为其 GUI 封装，使用时请遵循上游许可证条款。
