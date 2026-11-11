<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  activeCount: number;
  totalCount: number;
  finishedCount: number;
}>();

const emit = defineEmits<{
  (e: "clear-finished"): void;
}>();

const canClear = computed(() => props.finishedCount > 0);
</script>

<template>
  <div class="queue-stats">
    <div class="stat">
      <div class="stat-label">活跃任务</div>
      <div class="stat-value accent">{{ activeCount }}</div>
    </div>
    <div class="divider" />
    <div class="stat">
      <div class="stat-label">总任务数</div>
      <div class="stat-value">{{ totalCount }}</div>
    </div>
    <div class="divider" />
    <div class="stat">
      <div class="stat-label">已完成</div>
      <div class="stat-value">{{ finishedCount }}</div>
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
</template>

<style scoped>
.queue-stats {
  display: flex;
  align-items: center;
  gap: 22px;
  padding: 16px 20px;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
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
}
.stat-value.accent {
  color: var(--accent);
}
.divider {
  width: 1px;
  height: 34px;
  background: var(--border);
}
.spacer {
  flex: 1;
}
</style>
