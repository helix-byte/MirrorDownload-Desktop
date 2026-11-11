---
name: github-repo-bootstrap
description: 新建项目时自动创建 GitHub 仓库并完成初始化——本地 git 初始化、生成 README/.gitignore/LICENSE、创建远程仓库、推送首个 main、配置默认分支与后续 GitHub Flow 约定。当用户说“新建项目 / 初始化仓库 / 建一个 GitHub 仓库 / 把项目推到 GitHub”时使用。
---

# 角色定位

GitHub 仓库建仓与初始化助手。负责把一个刚起步或已有的本地项目，规范地初始化为一个可协作的 GitHub 仓库，并为后续所有开发铺好 GitHub Flow 的轨道。所有提示与说明使用简体中文。

本技能只负责「从 0 到第一次 main 就绪」；之后的每次改动请交由 `github-flow` 技能执行，提交信息交由 `git-commit` 技能生成。

# 前置检查

1. 确认 `git` 可用：`git --version`。
2. 确认 GitHub 认证凭据（三选一，按优先级）：
   - 环境变量 `GITHUB_TOKEN`（PAT）——同时读取进程级与用户级：
     `[Environment]::GetEnvironmentVariable('GITHUB_TOKEN','User')`（Windows）；
   - `gh` CLI 已登录：`gh auth status`；
   - 无凭据则中止，并提示用户设置 `GITHUB_TOKEN` 或执行 `gh auth login`。
3. 询问并确认以下信息（缺失则提问，不臆测）：
   - 仓库名（repo name）；
   - 可见性：public / private；
   - 所有者：个人账号还是某个 org；
   - 项目一句话描述（description）、可选 Website 与 Topics；
   - 开源许可证类型（如 MIT / Apache-2.0 / BSL 1.1 / Proprietary）。
   - 若用户只给了项目名/技术栈、未提供 description 与 topics：**主动按「描述与话题标签规范」生成建议值并请用户确认**，不得留空草草建仓。
4. 探测网络：先直连测试 `api.github.com`；若失败，按用户提供的代理（如 `http://127.0.0.1:7897`）重试。**注意**：本机常见 `github.com:443` 不通但 `api.github.com` 经代理可通，此时后续推送/合并需改走 GitHub REST API（Git Data API），见「推送策略」。

# 工作流程

## 1. 初始化本地仓库

1. 进入项目根目录；若无则创建。
2. 若尚未是 git 仓库：`git init`，并把默认分支命名为主分支：
   `git symbolic-ref HEAD refs/heads/main`（或 `git init -b main`）。
3. 若已存在 git 仓库但未提交：沿用，不重复初始化。

## 2. 补齐仓库骨架文件（缺失才创建，已存在不覆盖）

1. **README.md**：至少包含：项目名、一句话简介、特性、技术栈、快速开始、目录结构、许可证段。
2. **.gitignore**：按项目语言生成（Node/Vue：`node_modules/`、`dist/`、`*.local`；Rust/Tauri：`target/`、`gen/`；通用：`.tmp/`、`.DS_Store`、日志）。**不要**忽略 README/LICENSE。
3. **LICENSE**：按用户选的许可证生成正文（MIT/Apache-2.0 等）。
   - **许可证一致性规则**：若项目是某个上游项目（不同许可证）的封装/衍生，采用「本项目用 X（如 MIT），上游仍受其许可证约束」的双声明，并在 README「许可证」段同步说明，避免口径不一致。
   - 想让 GitHub 侧栏正确识别许可证徽章：LICENSE 文件内**只放该许可证的标准正文**；额外说明放到 README 或独立的 `NOTICE`/`LICENSES.md`，不要插在正文前。
4. **.gitattributes**（可选但推荐）：统一换行 `* text=auto eol=lf`，并声明二进制类型。
5. 若项目为 Node：在 `package.json` 补 `"license"` 字段；若为 Rust：在 `Cargo.toml` 的 `[package]` 补 `license`/`authors`/`description`。
6. 这些骨架文件通过首个提交纳入版本库。

## 3. 首个提交

1. 按 `git-commit` 技能规范生成首条提交信息，type 用 `init`（🎉）：
   示例：`🎉 init(仓库初始化): 初始化项目结构与基础配置`。
2. 提交：
   - `git add -A`
   - `git commit -m "<按 git-commit 规范生成的多行信息>"`（多行用 `git commit -F <临时文件>` 写入，避免转义问题）。

## 4. 创建远程 GitHub 仓库

**方式 A（首选，凭据为 `gh`）**：
```bash
gh repo create <owner>/<repo> --<public|private> --source=. --remote=origin --push --description "<desc>"
```
`--push` 会把当前 main 推上去并自动建立跟踪。

`gh repo create` **不支持直接设置 Topics**，必须在建仓后补一步：
```bash
gh repo edit <owner>/<repo> --add-topic t1 --add-topic t2 --add-topic t3 ...
# 或一次性：gh repo edit <owner>/<repo> --add-topic "a,b,c"
```

**方式 B（凭据为 `GITHUB_TOKEN`，走 REST API）**：
1. 用 `POST /user/repos`（个人）或 `POST /orgs/<org>/repos`（组织）创建仓库，body 含 `name/description/private/homepage`。
2. 设置 Topics：`PUT /repos/<owner>/<repo>/topics`，body `{"names":[...]}`（topics 规范见「描述与话题标签规范」）。
3. 设置 description/homepage：`PATCH /repos/<owner>/<repo>`（body `{"description":"...","homepage":"..."}`；homepage 无有效地址则传空串）。
4. 推送首个 main：见下方「推送策略」。
5. 添加远端：`git remote add origin https://github.com/<owner>/<repo>.git`（若已存在则 `git remote set-url`）。

> **验收要求**：description、topics、homepage 三项**必须**在仓库创建后写入，且在「输出汇总」前通过 `GET /repos/<owner>/<repo>` 复核实际值，确认已生效（description 非空、topics 数量符合规范）。

## 5. 描述与话题标签规范

**Description（仓库一句话描述）**
1. 结构：`是什么 + 给谁/解决什么 + 关键能力`，一句话讲清，可参考 README 首段。
2. 长度：≤ 350 字符（GitHub 上限）；中英文均可，但**与 README 简介口径一致**，不矛盾。
3. 示例：`基于 Tauri 2 + Vue 3 + Rust 的 Mirror Download 桌面版，为国内用户高速下载 HuggingFace / HF Mirror / ModelScope 上的 AI 模型与数据集。`

**Topics（话题标签）**
1. 数量：5~10 个，覆盖四类维度：**语言 / 框架 / 领域 / 用途**。
2. 格式：全小写，多词用 `-` 连接（如 `model-download`，不用下划线或空格）；避免无意义词（`app`、`code`、`project`）。
3. 去重、不与仓库名重复堆砌。
4. 示例（Tauri 下载器）：`tauri, vue3, rust, ai, model-download, huggingface, modelscope, llm, desktop-app, mirror`。

**Website（可选）**
1. 有官网/文档站/演示页才填，必须是有效 URL；
2. 无有效地址则**留空**（传空串），不要填占位或无效链接。

**复核方式（建仓后必做）**
- API：`GET /repos/<owner>/<repo>` 读取 `description`/`homepage`/`topics`，确认已写入；
- 或 `gh repo view <owner>/<repo> --json description,repositoryTopics,homepageUrl`。

## 6. 推送策略（关键，适配 github.com:443 不通的环境）

1. **优先常规 git 推送**：`git push -u origin main`。
2. 若 `github.com:443` 的 HTTPS 握手失败（`SSL_ERROR_SYSCALL`）但 `api.github.com` 经代理可通：
   - 改用 **Git Data API** 推送。步骤：
     a. 取本地 `git ls-tree -r HEAD` 得到全部文件与 blob；
     b. 对每个文件 `POST /git/blobs`（base64 内容）获取远端 blob sha；
     c. 递归按目录构建 tree（`040000` 子 tree + `100644` blob），`POST /git/trees`（首次可用无 base_tree）；
     d. `POST /git/commits`（message + tree + parents；首次可无 parent 或以上游初始提交为 parent）；
     e. `POST /git/refs` 创建 `refs/heads/main`（已存在则 `PATCH` 更新，`force:true`）。
   - 此路径下**不要**用 `force push` 覆盖已有他人内容；首次建仓覆盖仅含 LICENSE 的空壳时可与用户确认后执行。
3. 若 `github.com:443` 与 `api.github.com` 都不可通，中止并报告网络问题，不静默失败。

## 7. 配置 GitHub Flow 轨道（初始化收尾）

1. 确认默认分支为 `main`：`PATCH /repos/<owner>/<repo>` body `{"default_branch":"main"}`（或 `gh repo edit`）。
2. （推荐）为 `main` 开启分支保护：`PUT /repos/<owner>/<repo>/branches/main/protection`，要求 PR 评审、禁止直推；若无 Pro/权限不足则记录为「建议项」而非失败。
3. （推荐）仓库合并方式：默认启用 Squash merge，便于 `github-flow` 收尾时合并：
   `PATCH /repos/<owner>/<repo>` body `{"allow_squash_merge":true,"allow_merge_commit":false,"allow_rebase_merge":false}`。
4. 在 README「贡献」或独立 `CONTRIBUTING.md` 写入 GitHub Flow 约定：禁止直推 main、一律走特性分支 + PR、提交遵循 git-commit 规范。

## 8. 输出汇总

1. 汇总：仓库 URL、可见性、默认分支、首个提交 sha、是否启用分支保护/Squash；
2. **汇总已写入并复核的元信息**：description（内容）、topics（数量与列表）、homepage（值或「留空」）；确认全部符合「描述与话题标签规范」，未生效的不得口头宣称已设置；
3. 明确告诉用户：**后续任何改动都使用 `github-flow` 技能，提交信息使用 `git-commit` 技能**；
4. 若建仓过程中走了 API 推送或覆盖了空壳提交，明确说明。

# 与其它技能的协作

- 建仓完成后的每一次改动 → 调用 `github-flow`（分支 → 提交 → PR → 合并 → 清理）。
- 每次提交信息 → 调用 `git-commit`（gitmoji + 约定式，中文）。
- 本技能不负责日常迭代，只负责「0→首次 main 就绪」。

# 禁止行为

1. 未确认仓库名、可见性、许可证就擅自创建远程仓库；
2. 覆盖或删除用户已有的 `.gitignore`/README/LICENSE 内容；
3. 静默丢弃未提交改动；
4. 未确认就 `force push` 覆盖含他人提交的远端；
5. 把杂项许可证说明混入标准许可证正文（破坏徽章识别）而不提示；
6. 令牌明文打印到日志或提交到仓库；
7. 在未取得用户确认时执行破坏性操作（覆盖远端、删除分支、改默认分支）。

# 安全与密钥

1. `GITHUB_TOKEN` 仅从环境变量或 `gh` 凭据读取，不得写入任何文件、不得 echo 明文；
2. 生成的提交不含凭据；`.env` 类文件必须进 `.gitignore`；
3. 若检测到疑似密钥硬编码，提示用户移除后再提交。

# 输出要求

1. 每个关键步骤（初始化、提交、建远程、推送、配置保护）执行前简要说明将执行的命令；
2. 破坏性操作先征得确认；
3. 结束时按「输出汇总」给出完整结果与后续约定提醒。
