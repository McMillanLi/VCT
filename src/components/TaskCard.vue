<script setup lang="ts">
import { computed } from "vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useTasks } from "@/stores/tasks";
import { isTauri } from "@/api/backend";
import {
  formatSize,
  formatDuration,
  formatEta,
  formatSpeed,
  resolutionLabel,
} from "@/utils/format";
import type { TranscodeTask } from "@/types";

const props = defineProps<{ task: TranscodeTask }>();
const { removeTask, cancelTask } = useTasks();

const statusMeta = computed(() => {
  const map: Record<string, { label: string; color: string }> = {
    pending: { label: "排队中", color: "var(--text-tertiary)" },
    running: { label: "转码中", color: "var(--accent)" },
    completed: { label: "已完成", color: "var(--status-success)" },
    failed: { label: "失败", color: "var(--status-error)" },
    canceled: { label: "已取消", color: "var(--text-tertiary)" },
    paused: { label: "已暂停", color: "var(--status-warning)" },
  };
  return map[props.task.status] ?? map.pending;
});

const isRunning = computed(() => props.task.status === "running");
const codecBadge = computed(() => (props.task.target_codec === "av1" ? "AV1" : "H.265"));

async function openFolder() {
  if (!isTauri) return;
  try {
    await revealItemInDir(props.task.output_path);
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}
</script>

<template>
  <div class="task-card" :class="task.status">
    <div class="task-head">
      <div class="task-file">
        <div class="file-icon">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none">
            <path d="M7 3h7l5 5v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.6" />
            <path d="M14 3v5h5" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" />
            <path d="M10 13l3 3 0-5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" opacity="0.6" />
          </svg>
        </div>
        <div class="file-info">
          <div class="file-name" :title="task.file.name">{{ task.file.name }}</div>
          <div class="file-meta">
            <span>{{ resolutionLabel(task.file.width, task.file.height) }}</span>
            <span class="dot" />
            <span>{{ formatDuration(task.file.duration) }}</span>
            <span class="dot" />
            <span class="text-mono">{{ task.file.codec }}</span>
            <span class="dot" />
            <span>{{ formatSize(task.file.size) }}</span>
          </div>
        </div>
      </div>
      <div class="task-actions">
        <span class="codec-badge" :class="task.target_codec">{{ codecBadge }}</span>
        <span class="status-tag" :style="{ color: statusMeta.color }">
          <span v-if="isRunning" class="status-pulse" :style="{ background: statusMeta.color }" />
          {{ statusMeta.label }}
        </span>
        <button v-if="isRunning" class="icon-btn" title="取消" @click="cancelTask(task.id)">
          <svg width="14" height="14" viewBox="0 0 14 14"><rect x="2" y="2" width="10" height="10" rx="1.5" fill="currentColor" /></svg>
        </button>
        <button v-else-if="task.status === 'completed'" class="icon-btn" title="打开所在文件夹" @click="openFolder">
          <svg width="15" height="15" viewBox="0 0 16 16" fill="none"><path d="M2 5a1 1 0 0 1 1-1h3l1.5 1.5H13a1 1 0 0 1 1 1V12a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V5Z" stroke="currentColor" stroke-width="1.3" /></svg>
        </button>
        <button class="icon-btn" title="移除" @click="removeTask(task.id)">
          <svg width="14" height="14" viewBox="0 0 14 14"><path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
        </button>
      </div>
    </div>

    <!-- 进度条 -->
    <div class="progress-track">
      <div
        class="progress-fill"
        :class="{ indeterminate: isRunning && task.progress <= 0 }"
        :style="{ width: task.progress + '%', background: statusMeta.color }"
      />
    </div>

    <!-- 进度统计 -->
    <div class="task-stats">
      <span class="stat main">{{ task.progress.toFixed(0) }}%</span>
      <template v-if="isRunning">
        <span class="stat">{{ formatSpeed(task.speed) }}</span>
        <span class="stat">{{ Math.round(task.fps) }} fps</span>
        <span class="stat">已转 {{ formatDuration(task.processed_time) }}</span>
        <span class="stat">剩余 {{ formatEta(task.eta) }}</span>
      </template>
      <span v-else-if="task.status === 'completed'" class="stat done">输出 {{ task.output_path.split(/[\\/]/).pop() }}</span>
      <span v-else-if="task.status === 'failed'" class="stat err" :title="task.error">{{ task.error || "转码失败" }}</span>
    </div>
  </div>
</template>

<style scoped>
.task-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 12px 14px;
  transition: border-color var(--duration) var(--ease), opacity var(--duration) var(--ease);
}
.task-card.running {
  border-color: rgba(109, 94, 252, 0.4);
  box-shadow: 0 0 0 1px rgba(109, 94, 252, 0.15);
}
.task-card.completed {
  opacity: 0.82;
}
.task-card.failed {
  border-color: rgba(248, 113, 113, 0.3);
}

.task-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}
.task-file {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.file-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.file-info {
  min-width: 0;
}
.file-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 340px;
}
.file-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 2px;
}
.file-meta .dot {
  width: 2px;
  height: 2px;
  border-radius: 50%;
  background: var(--text-tertiary);
}

.task-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.codec-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 7px;
  border-radius: 4px;
  background: var(--accent-h265-soft);
  color: var(--accent-h265);
}
.codec-badge.av1 {
  background: var(--accent-av1-soft);
  color: var(--accent-av1);
}
.status-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 600;
}
.status-pulse {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  animation: pulse 1.2s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.7); }
}
.icon-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease);
}
.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.progress-track {
  height: 6px;
  background: var(--track);
  border-radius: 3px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.4s var(--ease-out);
  position: relative;
}
.progress-fill.indeterminate::after {
  content: "";
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.4), transparent);
  animation: shimmer 1.4s infinite;
}
@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.task-stats {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 8px;
  font-size: 11px;
}
.stat {
  color: var(--text-tertiary);
}
.stat.main {
  color: var(--text-primary);
  font-weight: 700;
  font-size: 12px;
  min-width: 36px;
}
.stat.done {
  color: var(--status-success);
}
.stat.err {
  color: var(--status-error);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}
</style>
