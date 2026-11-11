import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  startDownload,
  cancelDownload,
  onDownloadProgress,
  isTauri,
  __setMockProgressCb,
  type DownloadRequest,
  type ProgressEvent,
  type TaskStatus,
  type FileProgressItem,
} from "../api";

export interface DownloadTask {
  id: string;
  repoId: string;
  files: string[];
  quant?: string;
  status: TaskStatus;
  fileIndex: number;
  fileTotal: number;
  currentFile: string;
  filePercent: number;
  speed: string;
  speedBps?: number;
  downloaded: string;
  total: string;
  line: string;
  createdAt: number;
  startedAt?: number;
  error?: string;
  /** 逐文件分项进度 */
  filesDetail: FileProgressItem[];
  /** 已下载字节/总字节（用于计算队列总体进度） */
  downloadedBytes?: number;
  totalBytes?: number;
  /** 速度历史（字节/秒），用于迷你趋势图 */
  speedHistory: number[];
  /** 预估剩余秒数，null 表示无法估算 */
  etaSeconds: number | null;
}

/** 速度历史采样上限 */
const SPEED_HISTORY_MAX = 24;

let unlisten: (() => void) | null = null;

export const useDownloadsStore = defineStore("downloads", () => {
  const tasks = ref<DownloadTask[]>([]);
  const ready = ref(false);

  const activeCount = computed(
    () => tasks.value.filter((t) => t.status === "running" || t.status === "queued").length,
  );

  /** 队列总体进度（0-100），基于所有任务的总字节/已下载字节 */
  const overallProgress = computed(() => {
    let total = 0;
    let done = 0;
    for (const t of tasks.value) {
      const tb = t.totalBytes ?? 0;
      const db = t.downloadedBytes ?? 0;
      total += tb;
      done += Math.min(db, tb || db);
    }
    if (total <= 0) return 0;
    return Math.max(0, Math.min(100, (done / total) * 100));
  });

  /** 队列当前总速度（字节/秒），累加运行中任务的上一次采样 */
  const overallSpeedBps = computed(() => {
    let sum = 0;
    for (const t of tasks.value) {
      if (t.status === "running" && t.speedBps) sum += t.speedBps;
    }
    return sum;
  });

  /** 队列总已下载/总字节（原始字节） */
  const overallBytes = computed(() => {
    let total = 0;
    let done = 0;
    for (const t of tasks.value) {
      total += t.totalBytes ?? 0;
      done += t.downloadedBytes ?? 0;
    }
    return { done, total };
  });

  function pushSpeedHistory(t: DownloadTask, bps: number | undefined) {
    const v = bps && bps > 0 ? bps : 0;
    t.speedHistory.push(v);
    if (t.speedHistory.length > SPEED_HISTORY_MAX) {
      t.speedHistory.splice(0, t.speedHistory.length - SPEED_HISTORY_MAX);
    }
  }

  /** 依据剩余字节与近期平均速度估算 ETA（秒） */
  function computeEta(t: DownloadTask) {
    if (t.status === "completed") {
      t.etaSeconds = 0;
      return;
    }
    if (t.status !== "running") {
      t.etaSeconds = null;
      return;
    }
    const total = t.totalBytes ?? 0;
    const done = t.downloadedBytes ?? 0;
    const remain = Math.max(0, total - done);
    if (!remain || !t.speedBps || t.speedBps <= 0) {
      t.etaSeconds = null;
      return;
    }
    // 使用最近采样均值更平滑
    const hist = t.speedHistory.filter((x) => x > 0);
    const avg = hist.length
      ? hist.reduce((a, b) => a + b, 0) / hist.length
      : t.speedBps;
    t.etaSeconds = avg > 0 ? remain / avg : null;
  }

  function applyEvent(e: ProgressEvent) {
    const t = tasks.value.find((x) => x.id === e.taskId);
    if (!t) return;
    if (t.status === "queued" && e.status === "running" && !t.startedAt) {
      t.startedAt = Date.now();
    }
    t.status = e.status;
    t.fileIndex = e.fileIndex;
    t.fileTotal = e.fileTotal;
    t.currentFile = e.currentFile;
    t.filePercent = e.filePercent;
    t.speed = e.speed;
    t.speedBps = e.speedBps;
    t.downloaded = e.downloaded;
    t.total = e.total;
    t.line = e.line;
    if (typeof e.downloadedBytes === "number") t.downloadedBytes = e.downloadedBytes;
    if (typeof e.totalBytes === "number") t.totalBytes = e.totalBytes;

    // 合并逐文件明细：如有则覆盖，但保留已完成文件的历史
    if (e.files && e.files.length) {
      t.filesDetail = e.files;
    } else if (t.filesDetail.length) {
      // 后端未提供 files 时，依据 fileIndex/filePercent 维护最小明细
      const idx = Math.max(0, Math.min(t.filesDetail.length - 1, e.fileIndex - 1));
      for (let k = 0; k < t.filesDetail.length; k++) {
        const f = t.filesDetail[k];
        if (k < idx) f.status = "completed", f.percent = 100;
        else if (k === idx && e.status === "running") {
          f.status = "running";
          f.percent = e.filePercent;
        }
      }
    }

    // 速度历史
    if (e.status === "running") {
      pushSpeedHistory(t, e.speedBps);
    } else if (e.status === "completed") {
      pushSpeedHistory(t, 0);
    }

    if (e.status === "failed") t.error = e.line;
    computeEta(t);
  }

  async function init() {
    if (ready.value) return;
    if (isTauri) {
      unlisten = await onDownloadProgress(applyEvent);
    } else {
      __setMockProgressCb(applyEvent);
    }
    ready.value = true;
  }

  /** 释放进度事件监听（应用卸载时调用） */
  function dispose() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    if (!isTauri) __setMockProgressCb(null);
    ready.value = false;
  }

  async function enqueue(req: DownloadRequest) {
    const task: DownloadTask = {
      id: req.taskId,
      repoId: req.repoId,
      files: req.files,
      quant: req.quant,
      status: "queued",
      fileIndex: 0,
      fileTotal: req.files.length || 0,
      currentFile: "",
      filePercent: 0,
      speed: "",
      downloaded: "",
      total: "",
      line: "排队中…",
      createdAt: Date.now(),
      filesDetail: req.files.map((p) => ({
        path: p,
        status: "pending" as const,
        percent: 0,
        done: 0,
      })),
      speedHistory: [],
      etaSeconds: null,
    };
    tasks.value.unshift(task);
    await startDownload(req);
  }

  async function cancel(id: string) {
    const t = tasks.value.find((x) => x.id === id);
    if (t) {
      t.status = "cancelled";
      t.etaSeconds = null;
    }
    try {
      await cancelDownload(id);
    } catch {
      /* 忽略已结束 */
    }
  }

  function remove(id: string) {
    tasks.value = tasks.value.filter((x) => x.id !== id);
  }

  function clearFinished() {
    tasks.value = tasks.value.filter(
      (t) => t.status === "running" || t.status === "queued",
    );
  }

  return {
    tasks,
    ready,
    activeCount,
    overallProgress,
    overallSpeedBps,
    overallBytes,
    init,
    dispose,
    enqueue,
    cancel,
    remove,
    clearFinished,
  };
});

/** 字节数格式化为人类可读字符串 */
export function formatBytes(bytes: number | undefined): string {
  if (!bytes || bytes <= 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let v = bytes;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u++;
  }
  return `${v.toFixed(v >= 100 || u === 0 ? 0 : 1)} ${units[u]}`;
}

/** 秒数格式化为「剩余时间」文案 */
export function formatEta(seconds: number | null | undefined): string {
  if (seconds == null || !isFinite(seconds) || seconds <= 0) return "—";
  const s = Math.round(seconds);
  if (s < 60) return `${s} 秒`;
  const m = Math.floor(s / 60);
  if (m < 60) {
    const rs = s % 60;
    return rs ? `${m} 分 ${rs} 秒` : `${m} 分`;
  }
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm ? `${h} 小时 ${rm} 分` : `${h} 小时`;
}
