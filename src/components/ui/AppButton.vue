<script setup lang="ts">
// 统一按钮封装：支持 primary / danger / ghost / default 变体，loading 与 disabled 态。
import { computed } from "vue";

interface Props {
  variant?: "default" | "primary" | "danger" | "ghost";
  loading?: boolean;
  disabled?: boolean;
  type?: "button" | "submit" | "reset";
  block?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "default",
  loading: false,
  disabled: false,
  type: "button",
  block: false,
});

const emit = defineEmits<{ (e: "click", ev: MouseEvent): void }>();

const variantClass = computed(() => {
  switch (props.variant) {
    case "primary":
      return "btn-primary";
    case "danger":
      return "btn-danger";
    case "ghost":
      return "btn-ghost";
    default:
      return "";
  }
});

const isDisabled = computed(() => props.disabled || props.loading);

function onClick(ev: MouseEvent) {
  if (isDisabled.value) return;
  emit("click", ev);
}
</script>

<template>
  <button
    :type="type"
    class="btn app-btn"
    :class="[variantClass, { 'app-btn-block': block }]"
    :disabled="isDisabled"
    @click="onClick"
  >
    <span v-if="loading" class="spinner app-btn-spinner" aria-hidden="true"></span>
    <slot />
  </button>
</template>

<style scoped>
.app-btn {
  justify-content: center;
  min-height: 34px;
}
.app-btn-block {
  width: 100%;
}
.app-btn-spinner {
  width: 13px;
  height: 13px;
  border-width: 2px;
  border-top-color: currentColor;
  border-color: currentColor transparent transparent transparent;
}
.btn-primary .app-btn-spinner {
  border-color: rgba(255, 255, 255, 0.45) transparent transparent transparent;
}
</style>
