// 全站可复用的轻量 Toast。
// 用法（任意组件/普通 ts 文件，无需额外依赖或 provide）：
//   import { useToast } from "../components/ui/useToast"; // 或相对路径按需调整
//   const toast = useToast();
//   toast.success("保存成功");
//   toast.error("保存失败");
//   toast.info("正在检测…");
// 注意：使用前需要页面里挂载一次 <AppToastHost />（建议在根布局或各页面顶部放一个）。
import { reactive } from "vue";

export type ToastType = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
  duration: number;
}

const toasts = reactive<ToastItem[]>([]);
let seq = 0;

function push(type: ToastType, message: string, duration = 2800): number {
  const id = ++seq;
  toasts.push({ id, type, message, duration });
  if (duration > 0) {
    window.setTimeout(() => remove(id), duration);
  }
  return id;
}

function remove(id: number): void {
  const idx = toasts.findIndex((t) => t.id === id);
  if (idx !== -1) toasts.splice(idx, 1);
}

export function useToast() {
  return {
    toasts,
    success: (msg: string, duration?: number) => push("success", msg, duration),
    error: (msg: string, duration?: number) => push("error", msg, duration),
    info: (msg: string, duration?: number) => push("info", msg, duration),
    remove,
  };
}
