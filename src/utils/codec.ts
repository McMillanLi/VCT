import type { TargetCodec } from "@/types";

/**
 * 将 ffprobe 返回的 codec_name（源视频原编码）标准化到 TargetCodec 分类体系
 *
 * ffprobe codec_name 常见取值：
 *   H.264 → "h264" / "avc1"
 *   H.265 → "hevc" / "h265"
 *   AV1   → "av1"
 *   其它  → "mpeg4" / "vp9" / "wmv3" / "mpeg2video" ...
 *
 * 返回 undefined 表示「源编码不属于三种目标中的任何一种」，无法做一致性比较。
 */
export function normalizeSourceCodec(codec: string | undefined | null): TargetCodec | undefined {
  if (!codec) return undefined;
  const s = codec.toLowerCase().trim();
  if (!s) return undefined;
  if (s === "h264" || s === "avc1" || s.startsWith("h264")) return "h264";
  if (s === "hevc" || s === "h265" || s.startsWith("hevc")) return "h265";
  if (s === "av1" || s.startsWith("av1")) return "av1";
  return undefined;
}

/**
 * 判断「视频原编码」与「用户选中的目标编码」是否为同一编码家族
 * 用于在转码前提示用户：这样做不会带来格式转换的收益
 */
export function isSameCodecFamily(source: string | undefined | null, target: TargetCodec): boolean {
  const normalized = normalizeSourceCodec(source);
  if (!normalized) return false;
  return normalized === target;
}
