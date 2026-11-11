<script setup lang="ts">
// Toast 宿主：读取 useToast 的全局队列并渲染浮层。在页面中挂载一次即可。
import { useToast, type ToastItem } from "./useToast";

const toast = useToast();

const iconPaths: Record<ToastItem["type"], string> = {
  success: "M5 13l4 4L19 7",
  error: "M6 6l12 12M18 6L6 18",
  info: "M12 8h.01M11 12h1v4h1",
};
</script>

<template>
  <Teleport to="body">
    <div class="toast-host" aria-live="polite">
      <TransitionGroup name="toast">
        <div
          v-for="t in toast.toasts"
          :key="t.id"
          class="toast-item"
          :class="`toast-${t.type}`"
          @click="toast.remove(t.id)"
        >
          <span class="toast-icon">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.4">
              <path :d="iconPaths[t.type]" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </span>
          <span class="toast-msg">{{ t.message }}</span>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-host {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: flex-end;
  pointer-events: none;
}
.toast-item {
  pointer-events: auto;
  display: inline-flex;
  align-items: center;
  gap: 9px;
  min-width: 200px;
  max-width: 380px;
  padding: 11px 15px;
  border-radius: var(--radius-md);
  background: var(--bg-elevated);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow);
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
}
.toast-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  flex-shrink: 0;
}
.toast-msg {
  line-height: 1.4;
}
.toast-success {
  border-color: rgba(61, 220, 151, 0.4);
}
.toast-success .toast-icon {
  background: rgba(61, 220, 151, 0.16);
  color: var(--success);
}
.toast-error {
  border-color: rgba(242, 109, 109, 0.42);
}
.toast-error .toast-icon {
  background: rgba(242, 109, 109, 0.16);
  color: var(--danger);
}
.toast-info {
  border-color: rgba(79, 140, 255, 0.42);
}
.toast-info .toast-icon {
  background: var(--accent-soft);
  color: var(--accent);
}
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(24px);
}
</style>
