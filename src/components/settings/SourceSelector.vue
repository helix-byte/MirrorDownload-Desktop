<script setup lang="ts">
// 下载源选择：分段卡片，附探测结果展示（状态 / 延迟 / 推荐徽标）。
import type { SourceKind, ProbeReport } from "../../api";

interface SourceOption {
  value: SourceKind;
  label: string;
  desc: string;
}

const options: SourceOption[] = [
  { value: "hf", label: "Hugging Face", desc: "官方源，海外网络更稳定" },
  { value: "hf-mirror", label: "HF Mirror", desc: "国内镜像，国内网络推荐" },
  { value: "modelscope", label: "ModelScope", desc: "魔搭社区源，部分模型独有" },
];

const props = defineProps<{
  modelValue: SourceKind;
  probeReport: ProbeReport | null;
  probing: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: SourceKind): void;
  (e: "probe"): void;
}>();

function statusTagClass(status: string): string {
  const s = status.toUpperCase();
  if (s === "OK") return "tag-ok";
  return "tag-fail";
}
</script>

<template>
  <div class="source-wrap">
    <div class="source-options" role="radiogroup" aria-label="下载源">
      <button
        v-for="opt in options"
        :key="opt.value"
        type="button"
        class="source-card"
        :class="{ active: props.modelValue === opt.value }"
        role="radio"
        :aria-checked="props.modelValue === opt.value"
        @click="emit('update:modelValue', opt.value)"
      >
        <span class="radio-dot" :class="{ on: props.modelValue === opt.value }"></span>
        <span class="source-text">
          <span class="source-name">
            {{ opt.label }}
            <span
              v-if="probeReport?.recommended === opt.label"
              class="tag tag-best source-best"
            >推荐</span>
          </span>
          <span class="source-desc">{{ opt.desc }}</span>
        </span>
      </button>
    </div>

    <div class="probe-bar">
      <button class="btn probe-btn" :disabled="probing" @click="emit('probe')">
        <span v-if="probing" class="spinner" aria-hidden="true"></span>
        <span>{{ probing ? "探测中…" : "探测延迟" }}</span>
      </button>
      <span class="probe-tip">测试各下载源连通性与延迟，帮助你选择最优源。</span>
    </div>

    <Transition name="fade">
      <div v-if="probeReport" class="probe-results">
        <div v-for="r in probeReport.results" :key="r.source" class="probe-row">
          <span class="probe-source">{{ r.source }}</span>
          <span class="tag" :class="statusTagClass(r.status)">{{ r.status }}</span>
          <span class="probe-latency">
            {{ r.latencyMs == null ? "—" : `${r.latencyMs} ms` }}
          </span>
          <span v-if="r.best" class="tag tag-best">最佳</span>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.source-wrap {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.source-options {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}
.source-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  text-align: left;
  padding: 13px 14px;
  border-radius: var(--radius-md);
  background: var(--bg-base);
  border: 1px solid var(--border);
  transition: border-color 0.15s, background 0.15s;
}
.source-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
}
.source-card.active {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.radio-dot {
  flex-shrink: 0;
  margin-top: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--border-strong);
  position: relative;
  transition: border-color 0.15s;
}
.radio-dot.on {
  border-color: var(--accent);
}
.radio-dot.on::after {
  content: "";
  position: absolute;
  inset: 2px;
  border-radius: 50%;
  background: var(--accent);
}
.source-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}
.source-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}
.source-best {
  font-size: 10px;
  padding: 1px 6px;
}
.source-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.45;
}
.probe-bar {
  display: flex;
  align-items: center;
  gap: 12px;
}
.probe-btn {
  flex-shrink: 0;
}
.probe-tip {
  font-size: 12px;
  color: var(--text-muted);
}
.probe-results {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 10px 14px;
}
.probe-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12.5px;
  padding: 3px 0;
}
.probe-source {
  flex: 1;
  color: var(--text-secondary);
}
.probe-latency {
  font-family: var(--mono);
  color: var(--text-secondary);
  min-width: 64px;
  text-align: right;
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.18s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
@media (max-width: 900px) {
  .source-options {
    grid-template-columns: 1fr;
  }
}
</style>
