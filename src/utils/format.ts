// 格式化工具函数

/** 字节数 -> 人类可读（如 1.5 GB） */
export function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(value >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

/** 秒 -> 时长（如 01:23:45） */
export function formatDuration(seconds: number): string {
  if (!seconds || seconds <= 0 || !isFinite(seconds)) return "00:00";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${String(h).padStart(2, "0")}:${mm}:${ss}` : `${mm}:${ss}`;
}

/** 秒 -> 简短剩余时间（如 1m20s / 2h） */
export function formatEta(seconds: number): string {
  if (!seconds || seconds <= 0 || !isFinite(seconds)) return "—";
  if (seconds < 60) return `${Math.ceil(seconds)}s`;
  const m = Math.floor(seconds / 60);
  const s = Math.round(seconds % 60);
  if (m < 60) return s > 0 ? `${m}m${s}s` : `${m}m`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm > 0 ? `${h}h${rm}m` : `${h}h`;
}

/** 速度倍率格式化（如 2.5x） */
export function formatSpeed(speed: number): string {
  if (!speed || speed <= 0 || !isFinite(speed)) return "—";
  return `${speed.toFixed(2)}x`;
}

/** 分辨率 -> 简称（如 1920x1080 -> 1080p） */
export function resolutionLabel(width: number, height: number): string {
  if (!height) return "—";
  const h = height;
  if (h >= 2160) return "4K";
  if (h >= 1440) return "1440p";
  if (h >= 1080) return "1080p";
  if (h >= 720) return "720p";
  if (h >= 480) return "480p";
  return `${width}×${height}`;
}

/** 从完整路径提取目录部分 */
export function dirOf(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx >= 0 ? path.slice(0, idx) : path;
}

/** 根据目标编码生成输出文件名后缀 */
export function outputSuffix(codec: TargetCodecLike): string {
  return codec === "av1" ? "_AV1" : "_H265";
}

type TargetCodecLike = "h265" | "av1";

/** 根据输入路径与目标编码生成默认输出路径 */
export function defaultOutputPath(
  inputPath: string,
  codec: TargetCodecLike,
): string {
  const dot = inputPath.lastIndexOf(".");
  const base = dot > 0 ? inputPath.slice(0, dot) : inputPath;
  return `${base}${outputSuffix(codec)}.mp4`;
}

/** 生成唯一任务 ID */
export function genTaskId(): string {
  return `task_${Date.now().toString(36)}_${Math.random()
    .toString(36)
    .slice(2, 8)}`;
}
