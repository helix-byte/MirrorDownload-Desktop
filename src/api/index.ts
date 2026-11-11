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
  /** 已下载字节数（可选，后端未提供时为 undefined） */
  downloadedBytes?: number;
  /** 总字节数（可选） */
  totalBytes?: number;
  /** 当前文件已下载字节数（可选） */
  fileBytesDone?: number;
  /** 当前文件总字节数（可选） */
  fileBytesTotal?: number;
  /** 单个文件的平均速度（字节/秒，可选） */
  speedBps?: number;
  /** 文件清单及各文件累计状态（可选，后端可逐步提供） */
  files?: FileProgressItem[];
}

/** 单个文件的分项进度 */
export interface FileProgressItem {
  path: string;
  size?: number;
  /** 该文件已下载字节数 */
  done?: number;
  /** 0-100 */
  percent?: number;
  /** 该文件状态 */
  status?: "pending" | "running" | "completed" | "failed" | "skipped";
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
  ].map((p) => {
    const lfs = p.endsWith(".safetensors") || p.endsWith(".gguf");
    // 为 mock 提供稳定伪随机大小（同一文件名总得到同样的值），便于浏览器调试时演示大小列
    let h = 0;
    for (let k = 0; k < p.length; k++) h = (h * 31 + p.charCodeAt(k)) >>> 0;
    let size: number;
    if (lfs) {
      // 大文件：1.5 ~ 15 GB 之间
      size = Math.round((1.5 + (h % 135) / 10) * 1024 * 1024 * 1024);
    } else {
      // 小文件：0.3 ~ 60 KiB 之间
      size = Math.round((0.3 + (h % 600) / 10) * 1024);
    }
    return { path: p, lfs, size };
  });
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
  const names = req.files.length ? req.files : ["a.bin", "b.bin", "c.bin"];
  const total = names.length;
  // 为每个文件生成一个稳定的伪随机大小（0.5~50 MiB），便于展示分项与总量
  const sizes = names.map((n, idx) => {
    let h = 0;
    for (let k = 0; k < n.length; k++) h = (h * 31 + n.charCodeAt(k)) >>> 0;
    const mib = 0.5 + ((h + idx * 977) % 495) / 10;
    return Math.round(mib * 1024 * 1024);
  });
  const totalBytes = sizes.reduce((a, b) => a + b, 0);

  let i = 0; // 当前文件索引
  let doneInFile = 0; // 当前文件已下载字节
  let doneAll = 0; // 全部已下载字节

  const emitFiles = (): FileProgressItem[] =>
    names.map((p, idx) => {
      const size = sizes[idx];
      if (idx < i) return { path: p, size, done: size, percent: 100, status: "completed" };
      if (idx === i)
        return {
          path: p,
          size,
          done: doneInFile,
          percent: Math.round((doneInFile / size) * 100),
          status: "running",
        };
      return { path: p, size, done: 0, percent: 0, status: "pending" };
    });

  const fmt = (bytes: number) => {
    if (bytes >= 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MiB";
    if (bytes >= 1024) return (bytes / 1024).toFixed(0) + " KiB";
    return bytes + " B";
  };

  const realistic =
    typeof window !== "undefined" && window.location.href.includes("realistic");

  const tick = () => {
    if (i >= total) return;
    const size = sizes[i];
    // 每次前进 4%~18%
    const step = Math.round(size * (0.04 + Math.random() * 0.14));
    doneInFile = Math.min(size, doneInFile + step);
    doneAll = sizes.slice(0, i).reduce((a, b) => a + b, 0) + doneInFile;
    const speedBps = (Math.random() * 18 + 6) * 1024 * 1024;
    const speed = (speedBps / (1024 * 1024)).toFixed(2) + " MiB/s";
    const filePercent = Math.round((doneInFile / size) * 100);

    if (realistic) {
      // 模拟真实后端：仅提供 fileIndex/filePercent 与字符串字节，无 files/无字节字段
      mockProgressCb?.({
        taskId: req.taskId,
        status: "running",
        fileIndex: i + 1,
        fileTotal: total,
        currentFile: names[i],
        filePercent,
        speed,
        downloaded: fmt(doneInFile),
        total: fmt(size),
        line: `[${i + 1}/${total}] ${filePercent}% ${names[i]}`,
      });
    } else {
      mockProgressCb?.({
        taskId: req.taskId,
        status: "running",
        fileIndex: i + 1,
        fileTotal: total,
        currentFile: names[i],
        filePercent,
        speed,
        speedBps,
        downloaded: fmt(doneAll),
        total: fmt(totalBytes),
        line: `[${i + 1}/${total}] ${filePercent}% ${names[i]}`,
        downloadedBytes: doneAll,
        totalBytes,
        fileBytesDone: doneInFile,
        fileBytesTotal: size,
        files: emitFiles(),
      });
    }

    if (doneInFile >= size) {
      i++;
      doneInFile = 0;
      if (i >= total) {
        doneAll = totalBytes;
        if (realistic) {
          // 模拟真实后端完成事件：保留最终 fileIndex/fileTotal，但无 files/无字节字段
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
        } else {
          mockProgressCb?.({
            taskId: req.taskId,
            status: "completed",
            fileIndex: total,
            fileTotal: total,
            currentFile: "",
            filePercent: 100,
            speed: "",
            downloaded: fmt(totalBytes),
            total: fmt(totalBytes),
            line: "下载完成",
            downloadedBytes: totalBytes,
            totalBytes,
            files: names.map((p, idx) => ({
              path: p,
              size: sizes[idx],
              done: sizes[idx],
              percent: 100,
              status: "completed" as const,
            })),
          });
        }
        return;
      }
    }
    setTimeout(tick, 320);
  };
  setTimeout(tick, 200);
}

export { isTauri };
