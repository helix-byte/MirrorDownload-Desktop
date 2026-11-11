<script setup lang="ts">
import { computed, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useDownloadsStore } from "../stores/downloads";
import QueueStats from "../components/download/QueueStats.vue";
import DownloadTaskCard from "../components/download/DownloadTaskCard.vue";
import DownloadEmpty from "../components/download/DownloadEmpty.vue";

const store = useDownloadsStore();
const { tasks, activeCount } = storeToRefs(store);

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
});
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
      @clear-finished="handleClearFinished"
    />

    <div v-if="tasks.length" class="task-list">
      <DownloadTaskCard
        v-for="t in tasks"
        :key="t.id"
        :task="t"
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
