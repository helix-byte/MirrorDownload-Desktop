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
  downloaded: string;
  total: string;
  line: string;
  createdAt: number;
  error?: string;
}

let unlisten: (() => void) | null = null;

export const useDownloadsStore = defineStore("downloads", () => {
  const tasks = ref<DownloadTask[]>([]);
  const ready = ref(false);

  const activeCount = computed(
    () => tasks.value.filter((t) => t.status === "running" || t.status === "queued").length,
  );

  function applyEvent(e: ProgressEvent) {
    const t = tasks.value.find((x) => x.id === e.taskId);
    if (!t) return;
    t.status = e.status;
    t.fileIndex = e.fileIndex;
    t.fileTotal = e.fileTotal;
    t.currentFile = e.currentFile;
    t.filePercent = e.filePercent;
    t.speed = e.speed;
    t.downloaded = e.downloaded;
    t.total = e.total;
    t.line = e.line;
    if (e.status === "failed") t.error = e.line;
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
    };
    tasks.value.unshift(task);
    await startDownload(req);
  }

  async function cancel(id: string) {
    const t = tasks.value.find((x) => x.id === id);
    if (t) t.status = "cancelled";
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
    init,
    dispose,
    enqueue,
    cancel,
    remove,
    clearFinished,
  };
});
