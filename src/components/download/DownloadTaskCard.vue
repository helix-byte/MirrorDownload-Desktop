<script setup lang="ts">
import { computed } from "vue";
import type { DownloadTask } from "../../stores/downloads";
import type { TaskStatus } from "../../api";

const props = defineProps<{
  task: DownloadTask;
}>();

const emit = defineEmits<{
  (e: "cancel", id: string): void;
  (e: "remove", id: string): void;
}>();

const isActive = computed(
  () => props.task.status === "running" || props.task.status === "queued",
);

const statusMeta: Record<TaskStatus, { label: string; cls: string }> = {
  queued: { label: "排队中", cls: "st-queued" },
  running: { label: "下载中", cls: "st-running" },
  completed: { label: "已完成", cls: "st-completed" },
  failed: { label: "失败", cls: "st-failed" },
  cancelled: { label: "已取消", cls: "st-cancelled" },
};

const meta = computed(() => statusMeta[props.task.status]);

const percent = computed(() => {
  if (props.task.status === "completed") return 100;
  return Math.max(0, Math.min(100, Math.round(props.task.filePercent || 0)));
});

const fileLabel = computed(() => {
  const t = props.task;
  if (t.fileTotal > 0) {
    return `${Math.min(t.fileIndex, t.fileTotal)} / ${t.fileTotal}`;
  }
  return t.files.length ? `0 / ${t.files.length}` : "—";
});

const currentFile = computed(() => {
  const t = props.task;
  if (t.currentFile) return t.currentFile;
  if (t.status === "completed") return "全部文件已下载";
  if (t.status === "cancelled") return "下载已取消";
  if (t.status === "failed") return t.error || "下载失败";
  return "等待开始…";
});
</script>

<template>
  <div class="task-card" :class="meta.cls">
    <div class="task-head">
      <div class="task-repo">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M4 7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H6a2 2 0 01-2-2V7z" stroke-linejoin="round" />
        </svg>
        <span class="repo-id" :title="task.repoId">{{ task.repoId }}</span>
      </div>
      <span class="tag status-tag" :class="meta.cls">{{ meta.label }}</span>
    </div>

    <div class="task-file-row">
      <span class="file-name" :title="currentFile">{{ currentFile }}</span>
      <span class="file-count">{{ fileLabel }}</span>
    </div>

    <div class="progress-track">
      <div class="progress-fill" :class="meta.cls" :style="{ width: percent + '%' }" />
    </div>

    <div class="task-foot">
      <div class="metrics">
        <span class="percent">{{ percent }}%</span>
        <template v-if="task.status === 'running'">
          <span v-if="task.speed" class="metric">{{ task.speed }}</span>
          <span v-if="task.downloaded || task.total" class="metric">
            {{ task.downloaded }}<template v-if="task.total"> / {{ task.total }}</template>
          </span>
        </template>
        <span v-else-if="task.status === 'queued'" class="metric">排队中…</span>
      </div>
      <div class="actions">
        <button v-if="isActive" class="btn btn-danger btn-sm" @click="emit('cancel', task.id)">
          取消
        </button>
        <button v-else class="btn btn-ghost btn-sm" @click="emit('remove', task.id)">
          移除
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-card {
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 14px 16px;
  transition: border-color 0.15s;
}
.task-card:hover {
  border-color: var(--border-strong);
}
.task-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 9px;
}
.task-repo {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-secondary);
  min-width: 0;
}
.repo-id {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.status-tag.st-queued {
  color: var(--warning);
  background: rgba(245, 185, 75, 0.12);
}
.status-tag.st-running {
  color: var(--accent);
  background: var(--accent-soft);
}
.status-tag.st-completed {
  color: var(--success);
  background: rgba(61, 220, 151, 0.12);
}
.status-tag.st-failed {
  color: var(--danger);
  background: rgba(242, 109, 109, 0.12);
}
.status-tag.st-cancelled {
  color: var(--text-muted);
  background: var(--bg-hover);
}
.task-file-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}
.file-name {
  font-size: 12.5px;
  font-family: var(--mono);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}
.file-count {
  font-size: 11.5px;
  font-family: var(--mono);
  color: var(--text-muted);
  flex-shrink: 0;
}
.progress-track {
  height: 7px;
  border-radius: 999px;
  background: var(--bg-base);
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
  transition: width 0.3s ease;
}
.progress-fill.st-queued {
  background: var(--warning);
  opacity: 0.7;
}
.progress-fill.st-completed {
  background: var(--success);
}
.progress-fill.st-failed {
  background: var(--danger);
}
.progress-fill.st-cancelled {
  background: var(--text-muted);
}
.task-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 9px;
  gap: 12px;
}
.metrics {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
  overflow: hidden;
}
.percent {
  font-size: 12.5px;
  font-weight: 600;
  font-family: var(--mono);
  color: var(--text-primary);
}
.metric {
  font-size: 12px;
  font-family: var(--mono);
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
}
</style>
