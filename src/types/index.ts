// ===== 前后端共享类型定义（与 Rust 端结构一一对应） =====

/** 硬件加速检测结果 */
export interface HardwareInfo {
  available_encoders: string[];
  recommended_h265: string;
  recommended_av1: string;
  gpu_vendor: GpuVendor;
}

export type GpuVendor = "nvidia" | "amd" | "intel" | "apple" | "cpu";

/** 视频文件元信息 */
export interface FileInfo {
  path: string;
  name: string;
  size: number;
  duration: number; // 秒
  width: number;
  height: number;
  codec: string;
  fps: number;
}

/** 目标编码格式 */
export type TargetCodec = "h265" | "av1";

/** 画质预设 */
export type Preset = "fast" | "balanced" | "quality";

/** 构建命令输入 */
export interface BuildCommandInput {
  input_path: string;
  output_path: string;
  target_codec: TargetCodec;
  preset: Preset;
  encoder: string;
}

/** 构建命令输出 */
export interface BuiltCommand {
  args: string[];
  encoder: string;
  estimated_quality: string;
}

// ===== 任务状态（前端维护，后端通过事件更新） =====

export type TaskStatus =
  | "pending" // 排队中
  | "running" // 转码中
  | "paused" // 暂停
  | "completed" // 完成
  | "failed" // 失败
  | "canceled"; // 已取消

/** 转码任务（前端领域模型） */
export interface TranscodeTask {
  id: string;
  file: FileInfo;
  output_path: string;
  target_codec: TargetCodec;
  preset: Preset;
  encoder: string;
  status: TaskStatus;
  /** 进度百分比 0-100 */
  progress: number;
  /** 当前转码速度倍率，如 2.5 */
  speed: number;
  /** 当前 fps */
  fps: number;
  /** 已转码时长（秒） */
  processed_time: number;
  /** 预计剩余时间（秒） */
  eta: number;
  /** 错误信息 */
  error: string;
  /** 创建时间戳 */
  created_at: number;
  /** 开始时间戳 */
  started_at: number | null;
  /** 完成时间戳 */
  finished_at: number | null;
}

/** 后端推送的进度事件 payload */
export interface ProgressEvent {
  task_id: string;
  progress: number;
  speed: number;
  fps: number;
  processed_time: number;
  eta: number;
}

/** 后端推送的任务状态变更事件 payload */
export interface StatusEvent {
  task_id: string;
  status: TaskStatus;
  message?: string;
  output_path?: string;
}

/** 转码任务请求（发给后端） */
export interface TranscodeRequest {
  task_id: string;
  input_path: string;
  output_path: string;
  target_codec: TargetCodec;
  preset: Preset;
  encoder: string;
  duration: number;
}
