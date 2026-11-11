<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function goBrowse() {
  router.push("/browse");
}
</script>

<template>
  <div class="download-empty">
    <div class="empty-icon">
      <svg viewBox="0 0 24 24" width="46" height="46" fill="none" stroke="currentColor" stroke-width="1.4">
        <path d="M12 3v12m0 0l-4-4m4 4l4-4M5 21h14" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </div>
    <h2 class="empty-title">暂无下载任务</h2>
    <p class="empty-desc">去「模型浏览」搜索仓库，勾选需要的文件即可开始下载。</p>
    <button class="btn btn-primary" @click="goBrowse">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M11 4a7 7 0 100 14 7 7 0 000-14zM21 21l-4.3-4.3" stroke-linecap="round" />
      </svg>
      前往模型浏览
    </button>
  </div>
</template>

<style scoped>
.download-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 24px;
  text-align: center;
  color: var(--text-secondary);
}
.empty-icon {
  width: 92px;
  height: 92px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  background: var(--accent-soft);
  border: 1px solid var(--border);
  margin-bottom: 6px;
}
.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}
.empty-desc {
  font-size: 13px;
  color: var(--text-muted);
  max-width: 320px;
  line-height: 1.6;
  margin-bottom: 6px;
}
</style>
