<script setup lang="ts">
import type { SourceKind } from "../../api";

const props = defineProps<{
  repoId: string;
  repoType: string;
  source: SourceKind | "";
  revision: string;
  loading: boolean;
  sourceLabel: string;
}>();

const emit = defineEmits<{
  (e: "update:repoId", v: string): void;
  (e: "update:repoType", v: string): void;
  (e: "update:source", v: SourceKind | ""): void;
  (e: "update:revision", v: string): void;
  (e: "submit"): void;
}>();

const repoTypes = [
  { value: "model", label: "模型 (model)" },
  { value: "dataset", label: "数据集 (dataset)" },
];

const sources: { value: SourceKind | ""; label: string }[] = [
  { value: "", label: "跟随设置" },
  { value: "hf", label: "Hugging Face" },
  { value: "hf-mirror", label: "HF Mirror" },
  { value: "modelscope", label: "ModelScope" },
];

function onRepoIdInput(e: Event) {
  emit("update:repoId", (e.target as HTMLInputElement).value);
}
function onRevisionInput(e: Event) {
  emit("update:revision", (e.target as HTMLInputElement).value);
}
function onRepoTypeChange(e: Event) {
  emit("update:repoType", (e.target as HTMLSelectElement).value);
}
function onSourceChange(e: Event) {
  emit("update:source", (e.target as HTMLSelectElement).value as SourceKind | "");
}
function onKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !props.loading) emit("submit");
}
</script>

<template>
  <div class="search-bar card">
    <div class="repo-input-row">
      <div class="repo-input-wrap">
        <svg class="search-icon" viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4a7 7 0 100 14 7 7 0 000-14zM21 21l-4.3-4.3" stroke-linecap="round" />
        </svg>
        <input
          :value="repoId"
          class="repo-input"
          type="text"
          placeholder="输入仓库 ID，如 Qwen/Qwen2.5-7B-Instruct"
          spellcheck="false"
          @input="onRepoIdInput"
          @keydown="onKeydown"
        />
      </div>
      <button class="btn btn-primary query-btn" :disabled="loading || !repoId.trim()" @click="emit('submit')">
        <span v-if="loading" class="spinner small" />
        <span>{{ loading ? "查询中…" : "查询" }}</span>
      </button>
    </div>

    <div class="filters">
      <label class="field">
        <span class="field-label">类型</span>
        <select :value="repoType" @change="onRepoTypeChange">
          <option v-for="t in repoTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
        </select>
      </label>

      <label class="field">
        <span class="field-label">数据源</span>
        <select :value="source" @change="onSourceChange">
          <option v-for="s in sources" :key="s.value" :value="s.value">{{ s.label }}</option>
        </select>
      </label>

      <label class="field">
        <span class="field-label">版本 (revision)</span>
        <input
          :value="revision"
          type="text"
          placeholder="留空使用默认分支"
          spellcheck="false"
          @input="onRevisionInput"
          @keydown="onKeydown"
        />
      </label>

      <div class="source-hint">
        当前数据源：<span class="tag tag-best">{{ sourceLabel }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search-bar {
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex-shrink: 0;
}
.repo-input-row {
  display: flex;
  gap: 10px;
}
.repo-input-wrap {
  position: relative;
  flex: 1;
}
.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  pointer-events: none;
}
.repo-input {
  padding-left: 38px;
  height: 40px;
  font-size: 13.5px;
}
.query-btn {
  height: 40px;
  min-width: 96px;
  justify-content: center;
}
.spinner.small {
  width: 13px;
  height: 13px;
  border-width: 2px;
}
.filters {
  display: flex;
  align-items: flex-end;
  gap: 14px;
  flex-wrap: wrap;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 150px;
}
.field-label {
  font-size: 11.5px;
  color: var(--text-muted);
  letter-spacing: 0.3px;
}
.source-hint {
  margin-left: auto;
  font-size: 12px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 6px;
  padding-bottom: 6px;
}
</style>
