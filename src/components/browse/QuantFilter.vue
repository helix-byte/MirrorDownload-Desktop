<script setup lang="ts">
export interface QuantChip {
  key: string;
  label: string;
  count: number;
}

defineProps<{
  chips: QuantChip[];
  selected: string[];
}>();

const emit = defineEmits<{
  (e: "toggle", key: string): void;
  (e: "clear"): void;
}>();
</script>

<template>
  <div v-if="chips.length" class="quant-filter">
    <span class="qf-label">量化筛选</span>
    <div class="chips">
      <button
        v-for="c in chips"
        :key="c.key"
        class="chip"
        :class="{ active: selected.includes(c.key) }"
        @click="emit('toggle', c.key)"
      >
        {{ c.label }}
        <span class="chip-count">{{ c.count }}</span>
      </button>
    </div>
    <button v-if="selected.length" class="btn btn-ghost clear-btn" @click="emit('clear')">
      清除筛选
    </button>
  </div>
</template>

<style scoped>
.quant-filter {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  padding: 10px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}
.qf-label {
  font-size: 11.5px;
  color: var(--text-muted);
  letter-spacing: 0.3px;
  flex-shrink: 0;
}
.chips {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 11px;
  font-size: 12px;
  border-radius: 999px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  transition: all 0.15s;
  text-transform: uppercase;
  font-family: var(--mono);
}
.chip:hover {
  border-color: var(--border-strong);
  color: var(--text-primary);
}
.chip.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}
.chip-count {
  font-size: 10.5px;
  opacity: 0.75;
}
.clear-btn {
  font-size: 12px;
  padding: 4px 10px;
  color: var(--text-muted);
  margin-left: auto;
}
</style>
