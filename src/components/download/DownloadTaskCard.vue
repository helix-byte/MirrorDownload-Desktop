<script setup lang="ts">
import { computed, ref } from "vue";
import type { DownloadTask } from "../../stores/downloads";
import { formatBytes, formatEta } from "../../stores/downloads";
import type { TaskStatus } from "../../api";
import TaskFileDetails from "./TaskFileDetails.vue";

const props = defineProps<{
  task: DownloadTask;
  initialExpanded?: boolean;
}>();

const emit = defineEmits<{
  (e: "cancel", id: string): void;
  (e: "remove", id: string): void;
}>();

const expanded = ref(!!props.initialExpanded);

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

// 已下载/总大小（优先字节，回退到后端字符串）
const sizeText = computed(() => {
  const t = props.task;
  if (t.totalBytes && t.totalBytes > 0) {
    return `${formatBytes(t.downloadedBytes)} / ${formatBytes(t.totalBytes)}`;
  }
  if (t.downloaded || t.total) {
    return t.total ? `${t.downloaded} / ${t.total}` : t.downloaded;
  }
  return "";
});

const etaText = computed(() => {
  const t = props.task;
  if (t.status !== "running") return "";
  return formatEta(t.etaSeconds);
});

// 速度迷你趋势图（单位字节/秒）
const SPARK_W = 120;
const SPARK_H = 30;
const sparkPoints = computed(() => {
  const hist = props.task.speedHistory;
  if (hist.length < 2) return "";
  const max = Math.max(...hist, 1);
  const stepX = SPARK_W / (hist.length - 1);
  return hist
    .map((v, i) => {
      const x = i * stepX;
      const y = SPARK_H - (v / max) * (SPARK_H - 3) - 1.5;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});
const sparkArea = computed(() => {
  if (!sparkPoints.value) return "";
  return `0,${SPARK_H} ${sparkPoints.value} ${SPARK_W},${SPARK_H}`;
});

const canExpand = computed(() => props.task.filesDetail.length > 0);
</script>

<template>
  <div class="task-card" :class="[meta.cls, { expanded }]">
    <div class="task-head">
      <div class="task-repo">
        <span class="status-icon" :class="meta.cls">
          <!-- running: 旋转箭头 -->
          <svg v-if="task.status === 'running'" class="spin" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 3a9 9 0 109 9" stroke-linecap="round" />
          </svg>
          <!-- completed: 勾 -->
          <svg v-else-if="task.status === 'completed'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M5 12.5l4.2 4.2L19 6.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          <!-- failed: 叉 -->
          <svg v-else-if="task.status === 'failed'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M6 6l12 12M18 6L6 18" stroke-linecap="round" />
          </svg>
          <!-- cancelled: 横杠 -->
          <svg v-else-if="task.status === 'cancelled'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M6 12h12" stroke-linecap="round" />
          </svg>
          <!-- queued: 时钟 -->
          <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.9">
            <circle cx="12" cy="12" r="8.5" />
            <path d="M12 7.5V12l3 2" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
        <span class="repo-id" :title="task.repoId">{{ task.repoId }}</span>
        <span v-if="task.quant" class="quant-tag">{{ task.quant }}</span>
      </div>
      <span class="tag status-tag" :class="[meta.cls, { pulse: task.status === 'running' }]">
        {{ meta.label }}
      </span>
    </div>

    <div class="task-file-row">
      <span class="file-name" :title="currentFile">{{ currentFile }}</span>
      <span class="file-count">
        <template v-if="task.fileTotal > 0">
          <b>{{ Math.min(task.fileIndex, task.fileTotal) }}</b> / {{ task.fileTotal }} 个文件
        </template>
        <template v-else>{{ fileLabel }}</template>
      </span>
    </div>

    <div class="progress-track">
      <div class="progress-fill" :class="[meta.cls, { live: task.status === 'running' }]" :style="{ width: percent + '%' }" />
    </div>

    <div class="task-foot">
      <div class="metrics">
        <span class="percent" :class="meta.cls">{{ percent }}%</span>
        <template v-if="task.status === 'running'">
          <span v-if="task.speed" class="metric speed-metric">
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 20V5m0 0l-5 5m5-5l5 5" stroke-linecap="round" stroke-linejoin="round" transform="rotate(180 12 12)" />
            </svg>
            {{ task.speed }}
          </span>
          <span v-if="sizeText" class="metric">{{ sizeText }}</span>
          <span v-if="etaText" class="metric eta">
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.9">
              <circle cx="12" cy="12" r="8.5" />
              <path d="M12 7.5V12l3 2" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            剩余 {{ etaText }}
          </span>
        </template>
        <template v-else-if="task.status === 'queued'">
          <span class="metric">排队中…</span>
        </template>
        <template v-else-if="task.status === 'completed'">
          <span v-if="sizeText" class="metric">{{ sizeText }}</span>
        </template>
      </div>

      <div class="foot-right">
        <!-- 速度迷你趋势图 -->
        <svg
          v-if="task.status === 'running' && sparkPoints"
          class="spark"
          :viewBox="`0 0 ${SPARK_W} ${SPARK_H}`"
          preserveAspectRatio="none"
        >
          <polygon :points="sparkArea" class="spark-area" />
          <polyline :points="sparkPoints" class="spark-line" />
        </svg>
        <div class="actions">
          <button
            v-if="canExpand"
            class="btn btn-ghost btn-sm detail-toggle"
            :class="{ open: expanded }"
            @click="expanded = !expanded"
          >
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2">
              <path d="M9 6l6 6-6 6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            详情
          </button>
          <button v-if="isActive" class="btn btn-danger btn-sm" @click="emit('cancel', task.id)">
            取消
          </button>
          <button v-else class="btn btn-ghost btn-sm" @click="emit('remove', task.id)">
            移除
          </button>
        </div>
      </div>
    </div>

    <TaskFileDetails :files="task.filesDetail" :expanded="expanded" />
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
.task-card.st-running {
  border-color: rgba(79, 140, 255, 0.35);
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
  gap: 8px;
  color: var(--text-secondary);
  min-width: 0;
}
.status-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.status-icon.st-running {
  color: var(--accent);
}
.status-icon.st-completed {
  color: var(--success);
}
.status-icon.st-failed {
  color: var(--danger);
}
.status-icon.st-cancelled {
  color: var(--text-muted);
}
.status-icon.st-queued {
  color: var(--warning);
}
.spin {
  animation: spin 1.1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.repo-id {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.quant-tag {
  font-size: 10.5px;
  font-family: var(--mono);
  color: var(--accent);
  background: var(--accent-soft);
  border-radius: 4px;
  padding: 1px 6px;
  flex-shrink: 0;
}
.status-tag.st-queued {
  color: var(--warning);
  background: rgba(245, 185, 75, 0.12);
}
.status-tag.st-running {
  color: var(--accent);
  background: var(--accent-soft);
}
.status-tag.st-running.pulse {
  animation: tagPulse 1.6s ease-in-out infinite;
}
@keyframes tagPulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
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
.file-count b {
  color: var(--text-primary);
  font-weight: 600;
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
.progress-fill.live {
  background-image: linear-gradient(
    90deg,
    var(--accent) 0%,
    var(--accent-hover) 50%,
    var(--accent) 100%
  );
  background-size: 200% 100%;
  animation: shimmer 1.6s linear infinite;
}
@keyframes shimmer {
  from {
    background-position: 0% 0;
  }
  to {
    background-position: -200% 0;
  }
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
  flex-wrap: wrap;
}
.percent {
  font-size: 12.5px;
  font-weight: 600;
  font-family: var(--mono);
  color: var(--text-primary);
}
.percent.st-running {
  color: var(--accent);
}
.percent.st-completed {
  color: var(--success);
}
.percent.st-failed {
  color: var(--danger);
}
.metric {
  font-size: 12px;
  font-family: var(--mono);
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.speed-metric {
  color: var(--accent);
}
.eta {
  color: var(--text-secondary);
}
.foot-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}
.spark {
  width: 92px;
  height: 26px;
  opacity: 0.9;
}
.spark-line {
  fill: none;
  stroke: var(--accent);
  stroke-width: 1.4;
  stroke-linejoin: round;
  stroke-linecap: round;
  vector-effect: non-scaling-stroke;
}
.spark-area {
  fill: var(--accent-soft);
  stroke: none;
}
.actions {
  display: flex;
  gap: 8px;
}
.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
}
.detail-toggle {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}
.detail-toggle svg {
  transition: transform 0.18s ease;
}
.detail-toggle.open svg {
  transform: rotate(90deg);
}
</style>
