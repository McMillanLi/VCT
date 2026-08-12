<script setup lang="ts">
import { computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useApp } from "@/stores/app";
import { isTauri } from "@/api/backend";
import type { TargetCodec, Preset } from "@/types";

const { state, recommendedEncoder, isHardwareAccel, setNotificationsEnabled } = useApp();

const codecs: { value: TargetCodec; label: string; sub: string }[] = [
  { value: "h264", label: "H.264", sub: "AVC · 兼容性最好" },
  { value: "h265", label: "H.265", sub: "HEVC · 高压缩" },
  { value: "av1", label: "AV1", sub: "次世代 · 更高压缩" },
];

const presets: { value: Preset; label: string; desc: string }[] = [
  { value: "fast", label: "极速", desc: "体积略大 · 速度最快" },
  { value: "balanced", label: "均衡", desc: "推荐 · 画质体积兼顾" },
  { value: "quality", label: "高质量", desc: "体积最小 · 速度较慢" },
  { value: "custom", label: "自定义", desc: "手动指定 CRF / CQ" },
];

/** 当前编码格式对应的 CRF/CQ 范围提示 */
const crfRange = computed(() => {
  if (state.targetCodec === "av1") return { min: 0, max: 63, hint: "0-63 · 越小质量越高" };
  return { min: 0, max: 51, hint: "0-51 · 越小质量越高" };
});

const encoderLabel = computed(() => {
  const enc = recommendedEncoder.value;
  const map: Record<string, string> = {
    h264_nvenc: "NVIDIA NVENC · h264_nvenc",
    hevc_nvenc: "NVIDIA NVENC · hevc_nvenc",
    av1_nvenc: "NVIDIA NVENC · av1_nvenc",
    h264_amf: "AMD AMF · h264_amf",
    hevc_amf: "AMD AMF · hevc_amf",
    av1_amf: "AMD AMF · av1_amf",
    h264_qsv: "Intel QSV · h264_qsv",
    hevc_qsv: "Intel QSV · hevc_qsv",
    av1_qsv: "Intel QSV · av1_qsv",
    h264_videotoolbox: "Apple VideoToolbox · h264",
    hevc_videotoolbox: "Apple VideoToolbox · hevc",
    av1_videotoolbox: "Apple VideoToolbox · av1",
    libx264: "CPU 软解 · libx264",
    libx265: "CPU 软解 · libx265",
    libsvtav1: "CPU 软解 · libsvtav1",
  };
  return map[enc] ?? enc;
});

async function pickOutputDir() {
  if (!isTauri) return;
  const dir = await open({ directory: true, multiple: false });
  if (dir) state.customOutputDir = dir as string;
}
</script>

<template>
  <section class="config panel">
    <!-- 目标编码 -->
    <div class="config-row">
      <div class="config-label">
        <span class="label-text">目标编码</span>
        <span class="label-hint">{{ encoderLabel }}<span v-if="isHardwareAccel" class="hw-tag">硬件加速</span></span>
      </div>
      <div class="codec-group">
        <button
          v-for="c in codecs"
          :key="c.value"
          class="codec-btn"
          :class="{ active: state.targetCodec === c.value, [c.value]: state.targetCodec === c.value }"
          @click="state.targetCodec = c.value"
        >
          <span class="codec-label">{{ c.label }}</span>
          <span class="codec-sub">{{ c.sub }}</span>
        </button>
      </div>
    </div>

    <div class="divider" />

    <!-- 画质预设 -->
    <div class="config-row">
      <div class="config-label">
        <span class="label-text">画质预设</span>
        <span class="label-hint">CRF / CQ 与编码速度</span>
      </div>
      <div class="preset-group">
        <button
          v-for="p in presets"
          :key="p.value"
          class="preset-btn"
          :class="{ active: state.preset === p.value }"
          @click="state.preset = p.value"
        >
          <span class="preset-label">{{ p.label }}</span>
          <span class="preset-desc">{{ p.desc }}</span>
        </button>
      </div>
      <!-- 自定义 CRF 输入 -->
      <div v-if="state.preset === 'custom'" class="crf-input-row">
        <label class="crf-label">CRF / CQ</label>
        <input
          :value="state.customCrf"
          class="crf-input"
          type="number"
          :min="crfRange.min"
          :max="crfRange.max"
          @input="state.customCrf = Math.max(crfRange.min, Math.min(crfRange.max, Number(($event.target as HTMLInputElement).value) || 0))"
        />
        <span class="crf-hint">{{ crfRange.hint }}</span>
      </div>
    </div>

    <div class="divider" />

    <!-- 输出路径 -->
    <div class="config-row">
      <div class="config-label">
        <span class="label-text">输出位置</span>
        <span class="label-hint">默认在源文件同目录生成 _{{ state.targetCodec === "av1" ? "AV1" : state.targetCodec === "h264" ? "H264" : "H265" }}.mp4</span>
      </div>
      <div class="output-group">
        <button
          class="output-toggle"
          :class="{ active: state.outputMode === 'same-dir' }"
          @click="state.outputMode = 'same-dir'"
        >
          同目录
        </button>
        <button
          class="output-toggle"
          :class="{ active: state.outputMode === 'custom' }"
          @click="state.outputMode = 'custom'"
        >
          自定义目录
        </button>
        <div v-if="state.outputMode === 'custom'" class="output-path">
          <input
            v-model="state.customOutputDir"
            class="path-input"
            type="text"
            placeholder="选择或输入输出目录…"
            readonly
          />
          <button class="btn btn-ghost path-btn" @click="pickOutputDir">浏览…</button>
        </div>
      </div>
    </div>

    <div class="divider" />

    <!-- 完成通知 -->
    <div class="config-row">
      <div class="config-label">
        <span class="label-text">完成通知</span>
        <span class="label-hint">转码完成后推送系统桌面通知</span>
      </div>
      <button
        class="toggle-switch"
        :class="{ on: state.notificationsEnabled }"
        :title="state.notificationsEnabled ? '点击关闭' : '点击开启'"
        @click="setNotificationsEnabled(!state.notificationsEnabled)"
      >
        <span class="toggle-knob" />
      </button>
    </div>
  </section>
</template>

<style scoped>
.config {
  padding: 4px 18px;
}
.config-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 14px 0;
}
.config-label {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex-shrink: 0;
  min-width: 110px;
}
.label-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}
.label-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.hw-tag {
  color: var(--status-success);
  background: rgba(52, 211, 153, 0.12);
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
}
.divider {
  height: 1px;
  background: var(--border-subtle);
}

/* 编码选择 */
.codec-group {
  display: flex;
  gap: 10px;
}
.codec-btn {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  padding: 10px 18px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-elevated);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease);
  min-width: 120px;
}
.codec-btn:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
}
.codec-btn.active.h264 {
  border-color: var(--accent-h264);
  background: var(--accent-h264-soft);
  box-shadow: 0 0 0 1px var(--accent-h264);
}
.codec-btn.active.h265 {
  border-color: var(--accent-h265);
  background: var(--accent-h265-soft);
  box-shadow: 0 0 0 1px var(--accent-h265);
}
.codec-btn.active.av1 {
  border-color: var(--accent-av1);
  background: var(--accent-av1-soft);
  box-shadow: 0 0 0 1px var(--accent-av1);
}
.codec-label {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
}
.codec-btn.active.h264 .codec-label {
  color: var(--accent-h264);
}
.codec-btn.active.h265 .codec-label {
  color: var(--accent-h265);
}
.codec-btn.active.av1 .codec-label {
  color: var(--accent-av1);
}
.codec-sub {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* 预设选择 */
.preset-group {
  display: flex;
  gap: 8px;
}
.preset-btn {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  padding: 9px 14px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-elevated);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease);
}
.preset-btn:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
}
.preset-btn.active {
  border-color: var(--accent);
  background: var(--accent-soft);
  box-shadow: 0 0 0 1px var(--accent);
}
.preset-label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}
.preset-btn.active .preset-label {
  color: var(--accent);
}
.preset-desc {
  font-size: 10px;
  color: var(--text-tertiary);
  white-space: nowrap;
}

/* 自定义 CRF 输入 */
.crf-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  padding-left: 0;
}
.crf-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  white-space: nowrap;
}
.crf-input {
  width: 64px;
  padding: 6px 8px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-base);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 13px;
  text-align: center;
  outline: none;
  transition: border-color var(--duration-fast) var(--ease);
}
.crf-input:focus {
  border-color: var(--accent);
}
.crf-hint {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* 输出位置 */
.output-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
.output-toggle {
  padding: 8px 14px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease);
}
.output-toggle:hover {
  background: var(--bg-hover);
}
.output-toggle.active {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
}
.output-path {
  display: flex;
  gap: 6px;
  align-items: center;
}
.path-input {
  width: 220px;
  padding: 7px 10px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-base);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
  outline: none;
  cursor: pointer;
}
.path-input:hover {
  border-color: var(--border-strong);
}
.path-btn {
  padding: 7px 12px;
  font-size: 12px;
}

/* 通知开关 */
.toggle-switch {
  width: 42px;
  height: 24px;
  border-radius: 12px;
  border: none;
  background: var(--track);
  position: relative;
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease);
  flex-shrink: 0;
}
.toggle-switch.on {
  background: var(--accent);
}
.toggle-knob {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  transition: transform var(--duration-fast) var(--ease);
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}
.toggle-switch.on .toggle-knob {
  transform: translateX(18px);
}
</style>
