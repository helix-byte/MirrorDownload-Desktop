<script setup lang="ts">
// 设置分区卡片：标题 + 可选描述 + 内容插槽，统一设置页视觉。
interface Props {
  title: string;
  description?: string;
  icon?: string;
}

withDefaults(defineProps<Props>(), {
  description: "",
  icon: "",
});
</script>

<template>
  <section class="card settings-section">
    <header class="section-head">
      <div class="section-title-wrap">
        <span v-if="icon" class="section-icon">
          <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2">
            <path :d="icon" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
        <div>
          <h2 class="section-title">{{ title }}</h2>
          <p v-if="description" class="section-desc">{{ description }}</p>
        </div>
      </div>
      <slot name="action" />
    </header>
    <div class="section-body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.settings-section {
  padding: 20px 22px;
}
.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 4px;
}
.section-title-wrap {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.section-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  color: var(--accent);
  flex-shrink: 0;
}
.section-title {
  font-size: 15px;
  font-weight: 600;
  line-height: 1.5;
}
.section-desc {
  margin-top: 2px;
  font-size: 12.5px;
  color: var(--text-muted);
}
.section-body {
  margin-top: 14px;
}
</style>
