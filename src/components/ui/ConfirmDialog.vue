<script setup lang="ts">
// 确认弹窗：受控组件，通过 visible 控制显示，确认/取消均为事件。
import AppButton from "./AppButton.vue";

interface Props {
  visible: boolean;
  title?: string;
  message?: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
  loading?: boolean;
}

withDefaults(defineProps<Props>(), {
  title: "确认操作",
  message: "",
  confirmText: "确认",
  cancelText: "取消",
  danger: false,
  loading: false,
});

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <Teleport to="body">
    <Transition name="dlg">
      <div v-if="visible" class="dialog-mask" @click.self="emit('cancel')">
        <div class="dialog-box" role="dialog" aria-modal="true">
          <div class="dialog-head">
            <h3 class="dialog-title">{{ title }}</h3>
            <button class="dialog-close" aria-label="关闭" @click="emit('cancel')">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M6 6l12 12M18 6L6 18" stroke-linecap="round" />
              </svg>
            </button>
          </div>
          <div class="dialog-body">
            <slot>{{ message }}</slot>
          </div>
          <div class="dialog-foot">
            <AppButton variant="ghost" @click="emit('cancel')">{{ cancelText }}</AppButton>
            <AppButton :variant="danger ? 'danger' : 'primary'" :loading="loading" @click="emit('confirm')">
              {{ confirmText }}
            </AppButton>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dialog-mask {
  position: fixed;
  inset: 0;
  z-index: 9998;
  background: rgba(6, 8, 12, 0.62);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
.dialog-box {
  width: 100%;
  max-width: 420px;
  background: var(--bg-panel);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow);
  padding: 18px 20px;
}
.dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.dialog-title {
  font-size: 15px;
  font-weight: 600;
}
.dialog-close {
  display: inline-flex;
  color: var(--text-muted);
  padding: 4px;
  border-radius: var(--radius-sm);
}
.dialog-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.dialog-body {
  color: var(--text-secondary);
  font-size: 13.5px;
  line-height: 1.6;
  white-space: pre-line;
}
.dialog-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
}
.dlg-enter-active,
.dlg-leave-active {
  transition: opacity 0.16s ease;
}
.dlg-enter-active .dialog-box,
.dlg-leave-active .dialog-box {
  transition: transform 0.16s ease;
}
.dlg-enter-from,
.dlg-leave-to {
  opacity: 0;
}
.dlg-enter-from .dialog-box,
.dlg-leave-to .dialog-box {
  transform: translateY(8px) scale(0.98);
}
</style>
