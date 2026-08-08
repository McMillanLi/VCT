<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import { probeFile, isTauri } from "@/api/backend";
import { useTasks } from "@/stores/tasks";
import type { UnlistenFn } from "@tauri-apps/api/event";

const { addFiles } = useTasks();

const dragging = ref(false);
const loading = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);
let unlistenDrag: UnlistenFn | null = null;

/** 处理路径列表：逐个探测元信息后加入任务队列 */
async function handlePaths(paths: string[]) {
  if (!paths.length) return;
  loading.value = true;
  try {
    const infos = await Promise.all(paths.map((p) => probeFile(p)));
    addFiles(infos);
  } catch (e) {
    console.error("文件探测失败:", e);
  } finally {
    loading.value = false;
  }
}

/** 点击选择文件 */
async function pickFiles() {
  if (isTauri) {
    const selected = await open({
      multiple: true,
      filters: [{ name: "视频文件", extensions: ["mp4", "mov", "mkv", "avi", "flv", "webm"] }],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await handlePaths(paths);
  } else {
    // web 预览：触发隐藏 input
    fileInput.value?.click();
  }
}

// ---- web 预览下的 HTML5 拖拽 ----
function onHtmlDragOver(e: DragEvent) {
  e.preventDefault();
  dragging.value = true;
}
function onHtmlDragLeave(e: DragEvent) {
  e.preventDefault();
  dragging.value = false;
}
function onHtmlDrop(e: DragEvent) {
  e.preventDefault();
  dragging.value = false;
  if (isTauri) return; // Tauri 下由原生事件处理
  const files = Array.from(e.dataTransfer?.files ?? []);
  if (!files.length) return;
  // web 下无真实路径，用文件名构造伪路径供 mock 探测
  handlePaths(files.map((f) => `mock://${f.name}`));
}

function onInputChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const files = Array.from(input.files ?? []);
  if (files.length) handlePaths(files.map((f) => `mock://${f.name}`));
  input.value = ""; // 允许重复选择同一文件
}

onMounted(async () => {
  if (!isTauri) return;
  // Tauri 原生拖拽：webview 拦截文件拖入并提供路径
  try {
    unlistenDrag = await getCurrentWebviewWindow().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "enter" || p.type === "over") {
        dragging.value = true;
      } else if (p.type === "leave") {
        dragging.value = false;
      } else if (p.type === "drop") {
        dragging.value = false;
        handlePaths(p.paths);
      }
    });
  } catch (e) {
    console.warn("拖拽事件订阅失败:", e);
  }
});

onUnmounted(() => {
  unlistenDrag?.();
});
</script>

<template>
  <div
    class="dropzone"
    :class="{ dragging, loading }"
    @dragenter.prevent="onHtmlDragOver"
    @dragover.prevent="onHtmlDragOver"
    @dragleave.prevent="onHtmlDragLeave"
    @drop.prevent="onHtmlDrop"
    @click="pickFiles"
  >
    <input
      ref="fileInput"
      type="file"
      accept="video/*"
      multiple
      hidden
      @change="onInputChange"
    />

    <div class="dropzone-inner">
      <div class="drop-icon">
        <svg viewBox="0 0 48 48" width="44" height="44" fill="none">
          <rect x="6" y="6" width="36" height="36" rx="8" stroke="currentColor" stroke-width="2" opacity="0.5" />
          <path d="M24 14v16M17 23l7 7 7-7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </div>
      <div class="drop-text">
        <span class="drop-title">{{ dragging ? "松开即可添加" : "拖拽 MP4 文件到此处" }}</span>
        <span class="drop-hint">或 <em>点击选择</em> · 支持 MP4 / MOV / MKV / AVI 等批量添加</span>
      </div>
      <div v-if="loading" class="drop-loading">
        <span class="spinner" /> 正在读取文件信息…
      </div>
    </div>
  </div>
</template>

<style scoped>
.dropzone {
  position: relative;
  border: 1.5px dashed var(--border-strong);
  border-radius: var(--radius-lg);
  background: var(--bg-surface);
  padding: 28px 20px;
  cursor: pointer;
  transition: border-color var(--duration) var(--ease),
    background var(--duration) var(--ease), transform var(--duration) var(--ease);
}
.dropzone:hover {
  border-color: var(--accent);
  background: var(--bg-elevated);
}
.dropzone.dragging {
  border-color: var(--accent);
  background: var(--accent-soft);
  transform: scale(1.01);
  box-shadow: var(--shadow-glow);
}
.dropzone-inner {
  display: flex;
  align-items: center;
  gap: 18px;
}
.drop-icon {
  color: var(--text-tertiary);
  display: flex;
  transition: color var(--duration) var(--ease);
  flex-shrink: 0;
}
.dropzone:hover .drop-icon,
.dropzone.dragging .drop-icon {
  color: var(--accent);
}
.drop-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.drop-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}
.drop-hint {
  font-size: 12px;
  color: var(--text-tertiary);
}
.drop-hint em {
  color: var(--accent);
  font-style: normal;
  font-weight: 600;
}
.drop-loading {
  position: absolute;
  bottom: 10px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}
.spinner {
  width: 13px;
  height: 13px;
  border: 2px solid var(--border-strong);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
