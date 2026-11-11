<script setup lang="ts">
// 表单行：左侧标签 + 说明，右侧控件插槽，纵向或横向布局。
interface Props {
  label: string;
  hint?: string;
  forId?: string;
  horizontal?: boolean;
}

withDefaults(defineProps<Props>(), {
  hint: "",
  forId: undefined,
  horizontal: true,
});
</script>

<template>
  <div class="field-row" :class="horizontal ? 'is-horizontal' : 'is-vertical'">
    <div class="field-label-wrap">
      <label class="field-label" :for="forId">{{ label }}</label>
      <p v-if="hint" class="field-hint">{{ hint }}</p>
    </div>
    <div class="field-control">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.field-row {
  display: flex;
  gap: 18px;
  padding: 14px 0;
  border-bottom: 1px solid var(--border);
}
.field-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}
.field-row:first-child {
  padding-top: 0;
}
.is-horizontal {
  align-items: flex-start;
}
.is-vertical {
  flex-direction: column;
  gap: 8px;
}
.field-label-wrap {
  flex: 0 0 168px;
  padding-top: 4px;
}
.is-vertical .field-label-wrap {
  padding-top: 0;
}
.field-label {
  display: block;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--text-primary);
}
.field-hint {
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
.field-control {
  flex: 1;
  min-width: 0;
}
</style>
