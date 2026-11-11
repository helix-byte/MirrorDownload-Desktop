<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  activeCount: number;
  totalCount: number;
  finishedCount: number;
  /** 队列总体进度 0-100 */
  overallProgress: number;
  /** 队列总速度（字节/秒） */
  overallSpeed: number;
  /** 队列已下载/总字节 */
  bytesDone: number;
  bytesTotal: number;
}>();

const emit = defineEmits<{
  (e: "clear-finished"): void;
}>();

const canClear = computed(() => props.finishedCount > 0);
const running = computed(() => props.activeCount > 0);

const pct = computed(() => Math.max(0, Math.min(100, Math.round(props.overallProgress))));

function fmtBytes(bytes: number): string {
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

const speedLabel = computed(() => {
  if (!running.value || props.overallSpeed <= 0) return "";
  return fmtBytes(props.overallSpeed) + "/s";
});
</script>

<template>
  <div class="queue-stats">
    <div class="stats-row">
      <div class="stat">
        <div class="stat-label">活跃任务</div>
        <div class="stat-value accent">
          <span v-if="running" class="live-dot" />
          {{ activeCount }}
        </div>
      </div>
      <div class="divider" />
      <div class="stat">
        <div class="stat-label">总任务数</div>
        <div class="stat-value">{{ totalCount }}</div>
      </div>
      <div class="divider" />
      <div class="stat">
        <div class="stat-label">已完成</div>
        <div class="stat-value success">{{ finishedCount }}</div>
      </div>
      <div v-if="speedLabel" class="divider" />
      <div v-if="speedLabel" class="stat">
        <div class="stat-label">总速度</div>
        <div class="stat-value speed">{{ speedLabel }}</div>
      </div>
      <div class="spacer" />
      <button
        class="btn"
        :disabled="!canClear"
        title="移除所有已完成/失败/已取消的任务"
        @click="emit('clear-finished')"
      >
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 7h16M9 7V5h6v2M7 7l1 13h8l1-13" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        清空已完成
      </button>
    </div>

    <div class="overall">
      <div class="overall-head">
        <span class="overall-label">队列总体进度</span>
        <span class="overall-bytes">
          <template v-if="bytesTotal > 0">
            {{ fmtBytes(bytesDone) }} / {{ fmtBytes(bytesTotal) }}
          </template>
          <template v-else>尚无数据</template>
          <span class="overall-pct">{{ pct }}%</span>
        </span>
      </div>
      <div class="overall-track">
        <div
          class="overall-fill"
          :class="{ live: running }"
          :style="{ width: pct + '%' }"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.queue-stats {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 20px;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.stats-row {
  display: flex;
  align-items: center;
  gap: 22px;
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.stat-label {
  font-size: 11.5px;
  color: var(--text-muted);
  letter-spacing: 0.3px;
}
.stat-value {
  font-size: 22px;
  font-weight: 650;
  font-family: var(--mono);
  line-height: 1.15;
  display: flex;
  align-items: center;
  gap: 7px;
}
.stat-value.accent {
  color: var(--accent);
}
.stat-value.success {
  color: var(--success);
}
.stat-value.speed {
  color: var(--accent);
  font-size: 17px;
}
.live-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 0 0 var(--accent-soft);
  animation: pulse 1.6s ease-out infinite;
}
.divider {
  width: 1px;
  height: 34px;
  background: var(--border);
}
.spacer {
  flex: 1;
}

.overall {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.overall-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-muted);
}
.overall-bytes {
  font-family: var(--mono);
  display: flex;
  align-items: center;
  gap: 10px;
}
.overall-pct {
  color: var(--text-primary);
  font-weight: 600;
}
.overall-track {
  height: 8px;
  border-radius: 999px;
  background: var(--bg-base);
  overflow: hidden;
}
.overall-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--accent), var(--accent-hover));
  transition: width 0.4s cubic-bezier(0.22, 1, 0.36, 1);
}
.overall-fill.live {
  background-image: linear-gradient(
    90deg,
    var(--accent) 0%,
    var(--accent-hover) 50%,
    var(--accent) 100%
  );
  background-size: 200% 100%;
  animation: shimmer 1.8s linear infinite;
}
@keyframes shimmer {
  from {
    background-position: 0% 0;
  }
  to {
    background-position: -200% 0;
  }
}
@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(79, 140, 255, 0.5);
  }
  70% {
    box-shadow: 0 0 0 7px rgba(79, 140, 255, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(79, 140, 255, 0);
  }
}
</style>
