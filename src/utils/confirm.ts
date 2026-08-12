/**
 * 系统确认弹窗
 *
 * Tauri 桌面环境：调用 @tauri-apps/plugin-dialog 的原生确认框（系统样式，OK/Cancel 按钮）
 * Web 预览模式：降级为浏览器 window.confirm
 */
import { isTauri } from "@/api/backend";

interface ConfirmOptions {
  title?: string;
  okLabel?: string;
  cancelLabel?: string;
  kind?: "info" | "warning" | "error";
}

export async function confirmDialog(message: string, opts: ConfirmOptions = {}): Promise<boolean> {
  if (isTauri) {
    try {
      // 动态 import 避免 Web 环境加载 Tauri 插件报错
      const dialog = await import("@tauri-apps/plugin-dialog");
      return await dialog.confirm(message, {
        title: opts.title ?? "提示",
        okLabel: opts.okLabel ?? "确定",
        cancelLabel: opts.cancelLabel ?? "取消",
        kind: opts.kind ?? "warning",
      });
    } catch {
      // 插件未加载或调用失败 → 回退
      return window.confirm(message);
    }
  }
  return window.confirm(message);
}
