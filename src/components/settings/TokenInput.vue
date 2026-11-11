<script setup lang="ts">
// Token 输入框：password 类型，可切换明文显示。
import { ref } from "vue";

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
}>();

const visible = ref(false);

function onInput(ev: Event) {
  emit("update:modelValue", (ev.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="token-input">
    <input
      :type="visible ? 'text' : 'password'"
      :value="props.modelValue"
      :placeholder="placeholder ?? ''"
      autocomplete="off"
      spellcheck="false"
      @input="onInput"
    />
    <button
      type="button"
      class="toggle-btn"
      :aria-label="visible ? '隐藏' : '显示'"
      @click="visible = !visible"
    >
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path
          v-if="!visible"
          d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z M12 15a3 3 0 100-6 3 3 0 000 6z"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          v-else
          d="M3 3l18 18M10.6 10.6a2 2 0 002.8 2.8M9.9 5.2A10.9 10.9 0 0112 5c6.5 0 10 7 10 7a17.8 17.8 0 01-3.6 4.3M6.2 6.2A17.5 17.5 0 002 12s3.5 7 10 7a10.6 10.6 0 004.2-.9"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.token-input {
  position: relative;
  display: flex;
  align-items: center;
}
.token-input input {
  padding-right: 38px;
  font-family: var(--mono);
  letter-spacing: 0.5px;
}
.toggle-btn {
  position: absolute;
  right: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  padding: 4px;
  border-radius: var(--radius-sm);
}
.toggle-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}
</style>
