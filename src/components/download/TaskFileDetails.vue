<script setup lang="ts">
import { computed } from "vue";
import type { FileProgressItem } from "../../api";
import { formatBytes } from "../../stores/downloads";

const props = defineProps<{
  files: FileProgressItem[];
  expanded: boolean;
}>();

type FileStatus = NonNullable<FileProgressItem["status"]>;

/** 单行展示所需信息，统一归一下状态与百分比 */
interface RowView {
  item: FileProgressItem;
  basename: string;
  status: FileStatus;
  percent: number;
  doneText: string;
  sizeText: string;
}

const STATUS_ORDER: FileStatus[] = [
  "pending",
  "running",
  "completed",
  "failed",
  "skipped",
];

function normalizeStatus(s: FileProgressItem["status"]): FileStatus {
  return s && STATUS_ORDER.includes(s) ? s : "pending";
}

function percentOf(item: FileProgressItem): number {
  if (typeof item.percent === "number" && isFinite(item.percent)) {
    return Math.max(0, Math.min(100, Math.round(item.percent)));
  }
  if (item.size && item.size > 0 && typeof item.done === "number") {
    return Math.max(0, Math.min(100, Math.round((item.done / item.size) * 100)));
  }
  return 0;
}

function basenameOf(p: string): string {
  const parts = p.split(/[/\\]/);
  return parts[parts.length - 1] || p;
}

const rows = computed<RowView[]>(() =>
  props.files.map((item) => {
    const status = normalizeStatus(item.status);
    const percent = status === "completed" ? 100 : percentOf(item);
    return {
      item,
      basename: basenameOf(item.path),
      status,
      percent,
      doneText: formatBytes(item.done),
      sizeText: formatBytes(item.size),
    };
  }),
);

/** 汇总：已完成文件数 / 总数 */
const summary = computed(() => {
  let completed = 0;
  for (const r of rows.value) if (r.status === "completed") completed++;
  return { completed, total: rows.value.length };
});
</script>

<template>
  <div v-if="expanded" class="file-details">
    <div class="fd-head">
      <span class="fd-title">文件明细</span>
      <span class="fd-summary">{{ summary.completed }} / {{ summary.total }}</span>
    </div>

    <div class="fd-list" role="list">
      <div
        v-for="(row, idx) in rows"
        :key="row.item.path + '#' + idx"
        class="fd-row"
        role="listitem"
      >
        <!-- 状态图标 -->
        <span class="fd-icon" :class="'st-' + row.status" aria-hidden="true">
          <svg
            v-if="row.status === 'running'"
            class="fd-spin"
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2.4"
            stroke-linecap="round"
          >
            <path d="M12 3a9 9 0 108.94 10" />
          </svg>
          <svg
            v-else-if="row.status === 'completed'"
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M5 12.5l4.5 4.5L19 7" />
          </svg>
          <svg
            v-else-if="row.status === 'failed'"
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2.6"
            stroke-linecap="round"
          >
            <path d="M6 6l12 12M18 6L6 18" />
          </svg>
          <svg
            v-else-if="row.status === 'skipped'"
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          >
            <path d="M5 12h14" />
          </svg>
          <span v-else class="fd-dot" />
        </span>

        <!-- 文件名 -->
        <span class="fd-name" :title="row.item.path">{{ row.basename }}</span>

        <!-- 细进度条 -->
        <span class="fd-track">
          <span
            class="fd-fill"
            :class="'st-' + row.status"
            :style="{ width: row.percent + '%' }"
          />
        </span>

        <!-- 右侧 done/size 与百分比 -->
        <span class="fd-meta">
          <span class="fd-bytes">{{ row.doneText }}<template v-if="row.item.size"> / {{ row.sizeText }}</template></span>
          <span class="fd-percent" :class="'st-' + row.status">{{ row.percent }}%</span>
        </span>
      </div>

      <div v-if="!rows.length" class="fd-empty">暂无文件明细</div>
    </div>
  </div>
</template>

<style scoped>
.file-details {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}
.fd-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.fd-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.3px;
}
.fd-summary {
  font-size: 11.5px;
  font-family: var(--mono);
  color: var(--text-muted);
}
.fd-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 240px;
  overflow-y: auto;
  padding-right: 4px;
}
.fd-row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 5px 6px;
  border-radius: var(--radius-sm);
  transition: background 0.12s;
}
.fd-row:hover {
  background: var(--bg-hover);
}

/* 状态图标 */
.fd-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  color: var(--text-muted);
}
.fd-icon.st-running {
  color: var(--accent);
}
.fd-icon.st-completed {
  color: var(--success);
}
.fd-icon.st-failed {
  color: var(--danger);
}
.fd-icon.st-skipped {
  color: var(--text-muted);
}
.fd-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-muted);
  opacity: 0.7;
}
.fd-spin {
  animation: fd-rotate 0.9s linear infinite;
}
@keyframes fd-rotate {
  to {
    transform: rotate(360deg);
  }
}

/* 文件名 */
.fd-name {
  flex: 1 1 auto;
  min-width: 0;
  font-family: var(--mono);
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 细进度条 */
.fd-track {
  flex: 0 0 84px;
  height: 4px;
  border-radius: 999px;
  background: var(--bg-base);
  overflow: hidden;
}
.fd-fill {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
  transition: width 0.3s ease;
}
.fd-fill.st-pending {
  background: var(--text-muted);
  opacity: 0.4;
}
.fd-fill.st-completed {
  background: var(--success);
}
.fd-fill.st-failed {
  background: var(--danger);
}
.fd-fill.st-skipped {
  background: var(--text-muted);
  opacity: 0.5;
}

/* 右侧信息 */
.fd-meta {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.fd-bytes {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
}
.fd-percent {
  font-family: var(--mono);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 34px;
  text-align: right;
}
.fd-percent.st-completed {
  color: var(--success);
}
.fd-percent.st-failed {
  color: var(--danger);
}
.fd-percent.st-running {
  color: var(--accent);
}

.fd-empty {
  padding: 10px 6px;
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
}
</style>
