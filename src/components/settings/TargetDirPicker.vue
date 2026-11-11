<script setup lang="ts">
// 目录选择器：Tauri 下调用 @tauri-apps/plugin-dialog 的 open 选择目录；
// 浏览器 mock 模式降级为可编辑文本输入（方便纯前端调试）。
import { isTauri } from "../../api";

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
}>();

async function pick() {
  if (props.disabled) return;
  if (!isTauri) return;
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: props.modelValue || undefined,
      title: "选择目录",
    });
    if (typeof selected === "string") {
      emit("update:modelValue", selected);
    }
  } catch (err) {
    // 选择器不可用时保持静默，用户仍可手动输入路径
    console.error("选择目录失败", err);
  }
}

function onInput(ev: Event) {
  emit("update:modelValue", (ev.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="dir-picker">
    <input
      type="text"
      :value="props.modelValue"
      :placeholder="placeholder ?? ''"
      :disabled="props.disabled"
      spellcheck="false"
      class="dir-input"
      @input="onInput"
    />
    <button
      v-if="isTauri"
      type="button"
      class="btn dir-btn"
      :disabled="props.disabled"
      @click="pick"
    >
      <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" stroke-linejoin="round" />
      </svg>
      浏览…
    </button>
  </div>
</template>

<style scoped>
.dir-picker {
  display: flex;
  gap: 8px;
  align-items: center;
}
.dir-input {
  flex: 1;
  min-width: 0;
  font-family: var(--mono);
}
.dir-btn {
  flex-shrink: 0;
}
</style>
