// Tauri IPC 调用的统一封装与类型定义。
// 通过 globalThis 判断运行在 Tauri 还是浏览器，浏览器下提供 mock 便于纯前端调试。

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type SourceKind = "hf" | "hf-mirror" | "modelscope";

export interface Settings {
  source: SourceKind;
  threads: number;
  hfToken: string;
  modelscopeToken: string;
  token: string;
  proxy: string;
  outputDir: string;
  targetMode: "none" | "comfyui" | "lmstudio";
  comfyuiRoot: string;
  lmstudioRoot: string;
}

export interface RepoFile {
  path: string;
  lfs: boolean;
  size?: number;
}

export interface RepoInfo {
  id: string;
  source: string;
  revision: string;
  repoType: string;
  files: RepoFile[];
}

export interface ProbeResult {
  source: string;
  status: string;
  latencyMs: number | null;
  best: boolean;
}

export interface ProbeReport {
  results: ProbeResult[];
  recommended: string;
}

export interface CliStatus {
  available: boolean;
  path: string;
  version: string;
  message: string;
}

export type TaskStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface ProgressEvent {
  taskId: string;
  status: TaskStatus;
  fileIndex: number;
  fileTotal: number;
  currentFile: string;
  filePercent: number;
  speed: string;
  downloaded: string;
  total: string;
  line: string;
}

export interface DownloadRequest {
  taskId: string;
  repoId: string;
  repoType?: string;
  revision?: string;
  source?: string;
  threads?: number;
  output?: string;
  files: string[];
  include: string[];
  exclude: string[];
  quant?: string;
  targetMode: string;
  comfyuiRoot: string;
  lmstudioRoot: string;
  hfToken: string;
  modelscopeToken: string;
  token: string;
  proxy: string;
}

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// ---------------- 设置 ----------------
export async function getSettings(): Promise<Settings> {
  if (!isTauri) return mockSettings();
  return invoke<Settings>("get_settings");
}

export async function saveSettings(s: Settings): Promise<void> {
  if (!isTauri) return;
  return invoke("save_settings_cmd", { settings: s });
}

export async function getConfigDir(): Promise<string> {
  if (!isTauri) return "%APPDATA%/mirrordl-desktop";
  return invoke<string>("get_config_dir");
}

// ---------------- CLI 检测 ----------------
export async function checkCli(): Promise<CliStatus> {
  if (!isTauri)
    return {
      available: true,
      path: "mirrordl (mock)",
      version: "1.0.3",
      message: "",
    };
  return invoke<CliStatus>("check_cli_available");
}

// ---------------- probe ----------------
export async function probeSources(opts: {
  source?: string;
  proxy?: string;
  timeoutMs?: number;
}): Promise<ProbeReport> {
  if (!isTauri) {
    await delay(700);
    return {
      results: [
        { source: "Hugging Face", status: "FAIL", latencyMs: null, best: false },
        { source: "HF Mirror", status: "OK", latencyMs: 1000, best: true },
        { source: "ModelScope", status: "OK", latencyMs: 2000, best: false },
      ],
      recommended: "HF Mirror",
    };
  }
  return invoke<ProbeReport>("probe_sources", {
    source: opts.source ?? null,
    proxy: opts.proxy ?? null,
    timeoutMs: opts.timeoutMs ?? null,
  });
}

// ---------------- find ----------------
export async function findRepo(opts: {
  repoId: string;
  repoType?: string;
  revision?: string;
  source?: string;
  hfToken?: string;
  modelscopeToken?: string;
  token?: string;
  proxy?: string;
}): Promise<RepoInfo> {
  if (!isTauri) {
    await delay(600);
    return mockRepo(opts.repoId);
  }
  return invoke<RepoInfo>("find_repo", {
    repoId: opts.repoId,
    repoType: opts.repoType ?? null,
    revision: opts.revision ?? null,
    source: opts.source ?? null,
    hfToken: opts.hfToken ?? null,
    modelscopeToken: opts.modelscopeToken ?? null,
    token: opts.token ?? null,
    proxy: opts.proxy ?? null,
  });
}

// ---------------- 下载 ----------------
export async function startDownload(req: DownloadRequest): Promise<void> {
  if (!isTauri) {
    // mock 进度流
    mockDownload(req);
    return;
  }
  return invoke("start_download", { req });
}

export async function cancelDownload(taskId: string): Promise<void> {
  if (!isTauri) return;
  return invoke("cancel_download", { taskId });
}

export async function onDownloadProgress(
  cb: (e: ProgressEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauri) return () => {};
  return listen<ProgressEvent>("download-progress", (ev) => cb(ev.payload));
}

// ---------------- 浏览器 mock ----------------
function delay(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

function mockSettings(): Settings {
  return {
    source: "hf-mirror",
    threads: 5,
    hfToken: "",
    modelscopeToken: "",
    token: "",
    proxy: "",
    outputDir: "",
    targetMode: "none",
    comfyuiRoot: "",
    lmstudioRoot: "",
  };
}

function mockRepo(id: string): RepoInfo {
  const files = [
    ".gitattributes",
    "LICENSE",
    "README.md",
    "config.json",
    "model-00001-of-00004.safetensors",
    "model-00002-of-00004.safetensors",
    "model-00003-of-00004.safetensors",
    "model-00004-of-00004.safetensors",
    "model.safetensors.index.json",
    "tokenizer.json",
    "tokenizer_config.json",
    "vocab.json",
    "qwen2.5-7b-instruct-q4_k_m.gguf",
    "qwen2.5-7b-instruct-q8_0.gguf",
    "qwen2.5-7b-instruct-fp16.gguf",
  ].map((p) => ({ path: p, lfs: p.endsWith(".safetensors") || p.endsWith(".gguf") }));
  return {
    id: id || "Qwen/Qwen2.5-7B-Instruct",
    source: "hf",
    revision: "main",
    repoType: "model",
    files,
  };
}

let mockProgressCb: ((e: ProgressEvent) => void) | null = null;
export function __setMockProgressCb(cb: ((e: ProgressEvent) => void) | null) {
  mockProgressCb = cb;
}

function mockDownload(req: DownloadRequest) {
  const total = req.files.length || 3;
  const names = req.files.length ? req.files : ["a.bin", "b.bin", "c.bin"];
  let i = 0;
  let pct = 0;
  const tick = () => {
    const speed = (Math.random() * 20 + 5).toFixed(2) + " MiB/s";
    const dl = (pct * 0.5).toFixed(1) + " MiB";
    mockProgressCb?.({
      taskId: req.taskId,
      status: "running",
      fileIndex: i + 1,
      fileTotal: total,
      currentFile: names[i],
      filePercent: Math.round(pct),
      speed,
      downloaded: dl,
      total: "50 MiB",
      line: `[${i + 1}/${total}] ${Math.round(pct)}% ${names[i]}`,
    });
    pct += 12;
    if (pct > 100) {
      i++;
      pct = 0;
      if (i >= total) {
        mockProgressCb?.({
          taskId: req.taskId,
          status: "completed",
          fileIndex: total,
          fileTotal: total,
          currentFile: "",
          filePercent: 100,
          speed: "",
          downloaded: "",
          total: "",
          line: "下载完成",
        });
        return;
      }
    }
    setTimeout(tick, 320);
  };
  setTimeout(tick, 200);
}

export { isTauri };
