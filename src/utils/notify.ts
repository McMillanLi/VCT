// 系统通知工具
//
// 在 Tauri 环境下通过 tauri-plugin-notification 推送桌面通知；
// 纯 Web 预览模式下静默跳过（不影响功能）。
//
// 使用动态 import 避免在非 Tauri 环境下加载插件模块。

import { isTauri } from "@/api/backend";

/** 权限缓存：null=未检查, true/false=已检查结果 */
let permissionGranted: boolean | null = null;

/** 确保已获取通知权限（首次调用时请求） */
async function ensurePermission(): Promise<boolean> {
  if (!isTauri) return false;
  if (permissionGranted !== null) return permissionGranted;

  try {
    const { isPermissionGranted, requestPermission } = await import(
      "@tauri-apps/plugin-notification"
    );
    let granted = await isPermissionGranted();
    if (!granted) {
      const permission = await requestPermission();
      granted = permission === "granted";
    }
    permissionGranted = granted;
    return granted;
  } catch {
    permissionGranted = false;
    return false;
  }
}

/** 发送一条桌面通知 */
export async function notify(title: string, body?: string): Promise<void> {
  if (!isTauri) return;
  const granted = await ensurePermission();
  if (!granted) return;
  try {
    const { sendNotification } = await import("@tauri-apps/plugin-notification");
    await sendNotification({ title, body });
  } catch (e) {
    console.warn("发送通知失败:", e);
  }
}

/** 单个任务转码完成 */
export async function notifyTaskCompleted(filename: string): Promise<void> {
  await notify("✅ 转码完成", filename);
}

/** 单个任务转码失败 */
export async function notifyTaskFailed(filename: string, error: string): Promise<void> {
  // 取错误第一行，截断到 120 字符
  const shortError = error.split("\n")[0]?.slice(0, 120) || "未知错误";
  await notify("❌ 转码失败", `${filename}\n${shortError}`);
}

/** 全部任务完成（含成功/失败统计） */
export async function notifyAllDone(completed: number, failed: number): Promise<void> {
  if (failed > 0) {
    await notify("📋 批量转码结束", `成功 ${completed} 个，失败 ${failed} 个`);
  } else {
    await notify("🎉 全部转码完成", `共完成 ${completed} 个任务`);
  }
}
