// 后端通信封装层
//
// 自动检测运行环境：
//   - Tauri 环境下：通过 invoke 调用 Rust 命令，通过 listen 订阅事件
//   - 纯 Web 开发（npm run dev:web）下：回退到 mock 实现，模拟进度推送
//
// 这样 Step 1 的 UI 可在未安装 Rust 时独立预览验证。

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  HardwareInfo,
  FileInfo,
  BuildCommandInput,
  BuiltCommand,
  TranscodeRequest,
  ProgressEvent,
  StatusEvent,
} from "@/types";

export const isTauri = typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;

// ---- 事件订阅 ----

const progressListeners = new Set<(e: ProgressEvent) => void>();
const statusListeners = new Set<(e: StatusEvent) => void>();

/** 订阅转码进度事件 */
export async function onProgress(cb: (e: ProgressEvent) => void): Promise<UnlistenFn> {
  if (isTauri) {
    return listen<ProgressEvent>("transcode://progress", (event) => cb(event.payload));
  }
  // mock: 注册到内存监听器
  progressListeners.add(cb);
  return () => progressListeners.delete(cb);
}

/** 订阅任务状态变更事件 */
export async function onStatus(cb: (e: StatusEvent) => void): Promise<UnlistenFn> {
  if (isTauri) {
    return listen<StatusEvent>("transcode://status", (event) => cb(event.payload));
  }
  statusListeners.add(cb);
  return () => statusListeners.delete(cb);
}

function emitProgress(e: ProgressEvent) {
  progressListeners.forEach((cb) => cb(e));
}
function emitStatus(e: StatusEvent) {
  statusListeners.forEach((cb) => cb(e));
}

// ---- 命令封装 ----

export async function ping(): Promise<string> {
  if (isTauri) return invoke<string>("ping");
  return "VCT mock backend ready";
}

export async function detectHardware(): Promise<HardwareInfo> {
  if (isTauri) return invoke<HardwareInfo>("detect_hardware");
  // mock: 模拟检测到 NVIDIA
  return {
    available_encoders: ["h264_nvenc", "hevc_nvenc", "av1_nvenc"],
    recommended_h264: "h264_nvenc",
    recommended_h265: "hevc_nvenc",
    recommended_av1: "av1_nvenc",
    gpu_vendor: "nvidia",
  };
}

export async function probeFile(path: string): Promise<FileInfo> {
  if (isTauri) return invoke<FileInfo>("probe_file", { path });
  // mock: 生成随机但合理的视频元信息，用于 UI 预览
  const name = path.split(/[\\/]/).pop() || path;
  const presets = [
    { w: 1920, h: 1080, fps: 30, codec: "h264" },
    { w: 3840, h: 2160, fps: 60, codec: "h264" },
    { w: 1280, h: 720, fps: 30, codec: "h264" },
  ];
  const p = presets[Math.floor(Math.random() * presets.length)];
  return {
    path,
    name,
    size: Math.floor(Math.random() * 2_000_000_000) + 50_000_000,
    duration: Math.floor(Math.random() * 3600) + 60,
    width: p.w,
    height: p.h,
    codec: p.codec,
    fps: p.fps,
  };
}

export async function buildCommand(input: BuildCommandInput): Promise<BuiltCommand> {
  if (isTauri) return invoke<BuiltCommand>("build_command", { input });
  return {
    args: ["-i", input.input_path, "-c:v", input.encoder, input.output_path],
    encoder: input.encoder,
    estimated_quality: input.preset,
  };
}

export async function startTranscode(req: TranscodeRequest): Promise<{ success: boolean; message: string }> {
  if (isTauri) {
    const r = await invoke<{ task_id: string; success: boolean; output_path: string; message: string }>(
      "start_transcode",
      { req },
    );
    return { success: r.success, message: r.message };
  }
  // mock: 模拟一段渐进式进度推送
  mockTranscode(req);
  return { success: true, message: "mock 已启动" };
}

export async function cancelTranscode(taskId: string): Promise<boolean> {
  if (isTauri) return invoke<boolean>("cancel_transcode", { taskId });
  mockCancel(taskId);
  return true;
}

// ---- mock 转码模拟器 ----

const mockTimers = new Map<string, ReturnType<typeof setInterval>>();

function mockTranscode(req: TranscodeRequest) {
  emitStatus({ task_id: req.task_id, status: "running" });
  let processed = 0;
  const total = req.duration || 120;
  const interval = setInterval(() => {
    // 每帧推进 1~3 秒，模拟真实转码速度波动
    const step = 1 + Math.random() * 2.5;
    processed += step;
    const progress = Math.min(100, (processed / total) * 100);
    const speed = step; // 简化为每 tick 推进的秒数
    const remaining = Math.max(0, total - processed);
    const eta = speed > 0 ? remaining / speed : 0;
    emitProgress({
      task_id: req.task_id,
      progress,
      speed,
      fps: speed * 30,
      processed_time: processed,
      eta,
    });
    if (progress >= 100) {
      clearInterval(interval);
      mockTimers.delete(req.task_id);
      emitStatus({
        task_id: req.task_id,
        status: "completed",
        output_path: req.output_path,
        message: "转码完成（mock）",
      });
    }
  }, 500);
  mockTimers.set(req.task_id, interval);
}

function mockCancel(taskId: string) {
  const t = mockTimers.get(taskId);
  if (t) {
    clearInterval(t);
    mockTimers.delete(taskId);
  }
  emitStatus({ task_id: taskId, status: "canceled", message: "已取消" });
}
