// 应用级状态：硬件信息、转码配置
import { reactive, computed } from "vue";
import { detectHardware, isTauri } from "@/api/backend";
import type { HardwareInfo, TargetCodec, Preset } from "@/types";

interface AppState {
  /** 后端是否就绪 */
  backendReady: boolean;
  /** 是否处于 Tauri 环境 */
  isDesktop: boolean;
  /** 硬件检测结果 */
  hardware: HardwareInfo | null;
  /** 当前选择的目标编码 */
  targetCodec: TargetCodec;
  /** 当前选择的画质预设 */
  preset: Preset;
  /** 自定义 CRF/CQ 值（preset 为 custom 时生效） */
  customCrf: number;
  /** 输出位置模式 */
  outputMode: "same-dir" | "custom";
  /** 自定义输出目录 */
  customOutputDir: string;
  /** 是否正在初始化 */
  initializing: boolean;
  /** 完成时是否推送系统通知 */
  notificationsEnabled: boolean;
}

const state = reactive<AppState>({
  backendReady: false,
  isDesktop: isTauri,
  hardware: null,
  targetCodec: "h265",
  preset: "balanced",
  customCrf: 24,
  outputMode: "same-dir",
  customOutputDir: "",
  initializing: false,
  notificationsEnabled: true,
});

/** 根据硬件检测结果，返回当前编码目标推荐的编码器 */
export const recommendedEncoder = computed(() => {
  if (!state.hardware) return "libx265";
  switch (state.targetCodec) {
    case "h264":
      return state.hardware.recommended_h264;
    case "av1":
      return state.hardware.recommended_av1;
    default:
      return state.hardware.recommended_h265;
  }
});

/** 当前是否使用硬件加速 */
export const isHardwareAccel = computed(() => {
  const enc = recommendedEncoder.value;
  return /nvenc|amf|qsv|videotoolbox/.test(enc);
});

export function useApp() {
  /** 初始化：探测硬件、确认后端连通 */
  async function init() {
    state.initializing = true;
    try {
      state.hardware = await detectHardware();
      state.backendReady = true;
    } catch (e) {
      console.error("硬件检测失败:", e);
      // 降级为 CPU
      state.hardware = {
        available_encoders: [],
        recommended_h264: "libx264",
        recommended_h265: "libx265",
        recommended_av1: "libsvtav1",
        gpu_vendor: "cpu",
      };
    } finally {
      state.initializing = false;
    }
  }

  function setTargetCodec(codec: TargetCodec) {
    state.targetCodec = codec;
  }

  function setPreset(preset: Preset) {
    state.preset = preset;
  }

  function setOutputMode(mode: "same-dir" | "custom") {
    state.outputMode = mode;
  }

  function setCustomOutputDir(dir: string) {
    state.customOutputDir = dir;
    state.outputMode = "custom";
  }

  function setNotificationsEnabled(enabled: boolean) {
    state.notificationsEnabled = enabled;
  }

  return {
    state,
    recommendedEncoder,
    isHardwareAccel,
    init,
    setTargetCodec,
    setPreset,
    setOutputMode,
    setCustomOutputDir,
    setNotificationsEnabled,
  };
}
