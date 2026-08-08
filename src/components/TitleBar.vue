<script setup lang="ts">
import { computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useApp } from "@/stores/app";

const { state, isHardwareAccel } = useApp();

// GPU 厂商中文标签
const gpuLabel = computed(() => {
  const map: Record<string, string> = {
    nvidia: "NVIDIA NVENC",
    amd: "AMD AMF",
    intel: "Intel QSV",
    apple: "VideoToolbox",
    cpu: "CPU 软解",
  };
  return map[state.hardware?.gpu_vendor ?? "cpu"] ?? "CPU 软解";
});

async function minimize() {
  try {
    await getCurrentWindow().minimize();
  } catch {
    /* web 预览下无窗口操作 */
  }
}
async function toggleMaximize() {
  try {
    await getCurrentWindow().toggleMaximize();
  } catch {
    /* web 预览下无窗口操作 */
  }
}
async function close() {
  try {
    await getCurrentWindow().close();
  } catch {
    /* web 预览下无窗口操作 */
  }
}
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left" data-tauri-drag-region>
      <div class="logo">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none">
          <path
            d="M4 6.5C4 5.12 5.12 4 6.5 4h11A2.5 2.5 0 0 1 20 6.5v7a2.5 2.5 0 0 1-2.5 2.5H9l-4 4a.6.6 0 0 1-1-.46V6.5Z"
            fill="url(#g)"
          />
          <path d="M9 10.5l2 2 4-4" stroke="#fff" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          <defs>
            <linearGradient id="g" x1="4" y1="4" x2="20" y2="20" gradientUnits="userSpaceOnUse">
              <stop stop-color="#8275ff" />
              <stop offset="1" stop-color="#6d5efc" />
            </linearGradient>
          </defs>
        </svg>
      </div>
      <span class="app-name">VCT</span>
      <span class="app-sub">视频转码</span>
    </div>

    <div class="titlebar-center" data-tauri-drag-region>
      <div
        v-if="state.hardware"
        class="gpu-badge"
        :class="{ hw: isHardwareAccel }"
        :title="`当前编码器: ${state.hardware.recommended_h265} / ${state.hardware.recommended_av1}`"
      >
        <span class="gpu-dot" />
        {{ gpuLabel }}
      </div>
    </div>

    <div class="titlebar-right">
      <button class="win-btn" title="最小化" @click="minimize">
        <svg width="11" height="11" viewBox="0 0 11 11"><rect y="5" width="11" height="1" fill="currentColor" /></svg>
      </button>
      <button class="win-btn" title="最大化" @click="toggleMaximize">
        <svg width="11" height="11" viewBox="0 0 11 11"><rect x="0.5" y="0.5" width="10" height="10" rx="1" fill="none" stroke="currentColor" /></svg>
      </button>
      <button class="win-btn close" title="关闭" @click="close">
        <svg width="11" height="11" viewBox="0 0 11 11"><path d="M1 1l9 9M10 1l-9 9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" /></svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 6px 0 14px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.logo {
  display: flex;
  filter: drop-shadow(0 2px 6px rgba(109, 94, 252, 0.4));
}
.app-name {
  font-weight: 700;
  font-size: 14px;
  letter-spacing: 0.5px;
}
.app-sub {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-left: 2px;
}
.titlebar-center {
  display: flex;
  align-items: center;
  flex: 1;
  justify-content: center;
}
.gpu-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-tertiary);
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
}
.gpu-badge.hw {
  color: var(--status-success);
  background: rgba(52, 211, 153, 0.1);
  border-color: rgba(52, 211, 153, 0.25);
}
.gpu-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-tertiary);
}
.gpu-badge.hw .gpu-dot {
  background: var(--status-success);
  box-shadow: 0 0 6px var(--status-success);
}
.titlebar-right {
  display: flex;
  align-items: center;
  gap: 2px;
}
.win-btn {
  width: 34px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease), color var(--duration-fast) var(--ease);
}
.win-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.win-btn.close:hover {
  background: #e84855;
  color: #fff;
}
</style>
