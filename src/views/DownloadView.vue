<script setup lang="ts">
import { computed, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useDownloadsStore } from "../stores/downloads";
import { isTauri } from "../api";
import QueueStats from "../components/download/QueueStats.vue";
import DownloadTaskCard from "../components/download/DownloadTaskCard.vue";
import DownloadEmpty from "../components/download/DownloadEmpty.vue";

const store = useDownloadsStore();
const { tasks, activeCount, overallProgress, overallSpeedBps, overallBytes } =
  storeToRefs(store);

const demoExpandFirst = !isTauri && window.location.href.includes("expandFirst");

const totalCount = computed(() => tasks.value.length);
const finishedCount = computed(
  () =>
    tasks.value.filter(
      (t) =>
        t.status === "completed" ||
        t.status === "failed" ||
        t.status === "cancelled",
    ).length,
);

function handleCancel(id: string) {
  void store.cancel(id);
}
function handleRemove(id: string) {
  store.remove(id);
}
function handleClearFinished() {
  store.clearFinished();
}

onMounted(() => {
  void store.init();
  // 仅浏览器调试：?demo=1 自动注入若干 mock 任务，便于纯前端预览各种下载状态
  if (!isTauri && window.location.href.includes("demo=1")) {
    void seedDemo();
  }
});

async function seedDemo() {
  const mk = (taskId: string, repoId: string, files: string[], quant?: string) => ({
    taskId,
    repoId,
    files,
    include: files,
    exclude: [] as string[],
    quant,
    targetMode: "none",
    comfyuiRoot: "",
    lmstudioRoot: "",
    hfToken: "",
    modelscopeToken: "",
    token: "",
    proxy: "",
  });
  await store.enqueue(
    mk("demo-1", "Qwen/Qwen2.5-7B-Instruct", [
      "config.json",
      "model-00001-of-00004.safetensors",
      "model-00002-of-00004.safetensors",
      "model-00003-of-00004.safetensors",
      "model-00004-of-00004.safetensors",
      "tokenizer.json",
      "vocab.json",
    ], "Q4_K_M"),
  );
  await store.enqueue(
    mk("demo-2", "stabilityai/stable-diffusion-xl-base-1.0", [
      "README.md",
      "unet/diffusion_pytorch_model.safetensors",
      "vae/diffusion_pytorch_model.safetensors",
    ]),
  );
  setTimeout(() => {
    void store.enqueue(
      mk("demo-3", "google/gemma-2-2b-it", ["config.json", "model.safetensors", "tokenizer.model"]),
    );
  }, 900);
}
</script>

<template>
  <div class="download-view">
    <header class="view-header">
      <div>
        <h1 class="view-title">下载队列</h1>
        <p class="view-sub">实时查看文件级下载进度，可取消运行中的任务。</p>
      </div>
    </header>

    <QueueStats
      :active-count="activeCount"
      :total-count="totalCount"
      :finished-count="finishedCount"
      :overall-progress="overallProgress"
      :overall-speed="overallSpeedBps"
      :bytes-done="overallBytes.done"
      :bytes-total="overallBytes.total"
      @clear-finished="handleClearFinished"
    />

    <div v-if="tasks.length" class="task-list">
      <DownloadTaskCard
        v-for="t in tasks"
        :key="t.id"
        :task="t"
        :initial-expanded="demoExpandFirst && t.id === 'demo-1'"
        @cancel="handleCancel"
        @remove="handleRemove"
      />
    </div>
    <DownloadEmpty v-else />
  </div>
</template>

<style scoped>
.download-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 24px 28px;
  gap: 18px;
  overflow: hidden;
  min-height: 0;
}
.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}
.view-title {
  font-size: 20px;
  font-weight: 650;
  letter-spacing: 0.2px;
}
.view-sub {
  font-size: 12.5px;
  color: var(--text-muted);
  margin-top: 3px;
}
.task-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 11px;
  padding-right: 4px;
  padding-bottom: 8px;
  min-height: 0;
}
</style>
