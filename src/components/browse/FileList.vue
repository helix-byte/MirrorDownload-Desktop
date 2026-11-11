<script setup lang="ts">
import { computed } from "vue";
import type { RepoFile } from "../../api";

const props = defineProps<{
  files: RepoFile[];
  selected: string[];
  keyword: string;
}>();

const emit = defineEmits<{
  (e: "update:keyword", v: string): void;
  (e: "toggle", path: string): void;
  (e: "toggle-all", checked: boolean): void;
}>();

const filtered = computed(() => {
  const kw = props.keyword.trim().toLowerCase();
  if (!kw) return props.files;
  return props.files.filter((f) => f.path.toLowerCase().includes(kw));
});

const selectedSet = computed(() => new Set(props.selected));

const allChecked = computed(
  () =>
    filtered.value.length > 0 &&
    filtered.value.every((f) => selectedSet.value.has(f.path)),
);
const someChecked = computed(
  () =>
    filtered.value.some((f) => selectedSet.value.has(f.path)) && !allChecked.value,
);

function onKeyword(e: Event) {
  emit("update:keyword", (e.target as HTMLInputElement).value);
}
function onToggleAll(e: Event) {
  emit("toggle-all", (e.target as HTMLInputElement).checked);
}

function formatSize(size?: number): string {
  if (size === undefined || size === null) return "";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let n = size;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(n >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

function baseName(path: string): string {
  const idx = path.lastIndexOf("/");
  return idx >= 0 ? path.slice(idx + 1) : path;
}
function dirName(path: string): string {
  const idx = path.lastIndexOf("/");
  return idx >= 0 ? path.slice(0, idx + 1) : "";
}
</script>

<template>
  <div class="file-list-wrap">
    <div class="file-list-toolbar">
      <label class="check-all">
        <input
          type="checkbox"
          :checked="allChecked"
          :indeterminate.prop="someChecked"
          @change="onToggleAll"
        />
        <span>全选</span>
      </label>
      <div class="kw-wrap">
        <input
          :value="keyword"
          type="text"
          placeholder="按文件名过滤…"
          spellcheck="false"
          @input="onKeyword"
        />
      </div>
      <span class="shown-count">{{ filtered.length }} / {{ files.length }}</span>
    </div>

    <div class="file-scroll">
      <div v-if="!filtered.length" class="no-match">没有匹配的文件</div>
      <label
        v-for="f in filtered"
        :key="f.path"
        class="file-row"
        :class="{ checked: selectedSet.has(f.path) }"
      >
        <input
          type="checkbox"
          :checked="selectedSet.has(f.path)"
          @change="emit('toggle', f.path)"
        />
        <span class="file-path" :title="f.path">
          <span v-if="dirName(f.path)" class="dir">{{ dirName(f.path) }}</span>
          <span class="base">{{ baseName(f.path) }}</span>
        </span>
        <span v-if="f.lfs" class="tag lfs-tag">LFS</span>
        <span v-if="formatSize(f.size)" class="file-size">{{ formatSize(f.size) }}</span>
      </label>
    </div>
  </div>
</template>

<style scoped>
.file-list-wrap {
  display: flex;
  flex-direction: column;
  min-height: 0;
  flex: 1;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--bg-base);
}
.file-list-toolbar {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 9px 12px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.check-all {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12.5px;
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
  user-select: none;
}
.kw-wrap {
  flex: 1;
  min-width: 0;
}
.kw-wrap input {
  height: 30px;
  font-size: 12.5px;
}
.shown-count {
  font-size: 11.5px;
  font-family: var(--mono);
  color: var(--text-muted);
  flex-shrink: 0;
}
.file-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
.no-match {
  padding: 28px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
.file-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--bg-elevated);
  cursor: pointer;
  transition: background 0.12s;
}
.file-row:hover {
  background: var(--bg-hover);
}
.file-row.checked {
  background: var(--accent-soft);
}
input[type="checkbox"] {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
  accent-color: var(--accent);
  cursor: pointer;
  padding: 0;
  margin: 0;
}
.file-path {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  font-family: var(--mono);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.dir {
  color: var(--text-muted);
}
.base {
  color: var(--text-primary);
}
.lfs-tag {
  font-size: 9.5px;
  padding: 1px 6px;
  color: var(--warning);
  background: rgba(245, 185, 75, 0.12);
  flex-shrink: 0;
  letter-spacing: 0.5px;
}
.file-size {
  font-size: 11.5px;
  font-family: var(--mono);
  color: var(--text-muted);
  flex-shrink: 0;
  min-width: 58px;
  text-align: right;
}
</style>
